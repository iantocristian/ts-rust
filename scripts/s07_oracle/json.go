package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
)

// Decode recursively before the behavioral recovery boundary. encoding/json's
// ordinary map decoder accepts duplicate keys and repairs invalid UTF-8.
func strictValue(decoder *json.Decoder) (any, error) {
	token, err := decoder.Token()
	if err != nil {
		return nil, err
	}
	if delimiter, ok := token.(json.Delim); ok {
		switch delimiter {
		case '{':
			result := map[string]any{}
			for decoder.More() {
				keyToken, err := decoder.Token()
				if err != nil {
					return nil, err
				}
				key, ok := keyToken.(string)
				if !ok {
					return nil, fmt.Errorf("nonstring object key")
				}
				if _, found := result[key]; found {
					return nil, fmt.Errorf("duplicate key %q", key)
				}
				value, err := strictValue(decoder)
				if err != nil {
					return nil, err
				}
				result[key] = value
			}
			closing, err := decoder.Token()
			if err != nil || closing != json.Delim('}') {
				return nil, fmt.Errorf("unclosed object")
			}
			return result, nil
		case '[':
			result := []any{}
			for decoder.More() {
				value, err := strictValue(decoder)
				if err != nil {
					return nil, err
				}
				result = append(result, value)
			}
			closing, err := decoder.Token()
			if err != nil || closing != json.Delim(']') {
				return nil, fmt.Errorf("unclosed array")
			}
			return result, nil
		default:
			return nil, fmt.Errorf("unexpected delimiter")
		}
	}
	if number, ok := token.(json.Number); ok {
		value, err := number.Int64()
		if err != nil {
			return nil, fmt.Errorf("noninteger/out-of-range number")
		}
		return value, nil
	}

	return token, nil
}

func fields(value map[string]any, required string) error {
	names := strings.Fields(required)
	if len(value) != len(names) {
		return fmt.Errorf("wrong field count: expected %s", required)
	}
	for _, name := range names {
		if _, ok := value[name]; !ok {
			return fmt.Errorf("missing field %s", name)
		}
	}
	return nil
}
func integer(value any) (int64, error) {
	result, ok := value.(int64)
	if !ok {
		return 0, fmt.Errorf("expected integer")
	}
	return result, nil
}
func boolean(value any) (bool, error) {
	result, ok := value.(bool)
	if !ok {
		return false, fmt.Errorf("expected boolean")
	}
	return result, nil
}
func text(value any) (string, error) {
	result, ok := value.(string)
	if !ok {
		return "", fmt.Errorf("expected string")
	}
	return result, nil
}
func byteString(value any) ([]byte, error) {
	s, err := text(value)
	if err != nil {
		return nil, err
	}
	if strings.ToLower(s) != s {
		return nil, fmt.Errorf("noncanonical hex")
	}
	result, err := hex.DecodeString(s)
	if err != nil || len(result) > 8*1024*1024 {
		return nil, fmt.Errorf("invalid/oversized hex")
	}
	return result, nil
}

// Go's JSON decoder repairs lone UTF-16 escape surrogates. Request strings are
// Unicode scalar sequences, while arbitrary Go strings use byte hex instead.
func scalarEscapes(raw []byte) error {
	inString := false
	for index := 0; index < len(raw); index++ {
		if raw[index] == '"' {
			inString = !inString
			continue
		}
		if !inString || raw[index] != '\\' {
			continue
		}
		index++
		if index >= len(raw) {
			return fmt.Errorf("truncated JSON string escape")
		}
		if raw[index] != 'u' {
			continue
		}
		if index+4 >= len(raw) {
			return fmt.Errorf("truncated Unicode escape")
		}
		word, err := strconv.ParseUint(string(raw[index+1:index+5]), 16, 16)
		if err != nil {
			return err
		}
		index += 4
		if word >= 0xdc00 && word <= 0xdfff {
			return fmt.Errorf("unpaired low surrogate")
		}
		if word < 0xd800 || word > 0xdbff {
			continue
		}
		if index+6 >= len(raw) || raw[index+1] != '\\' || raw[index+2] != 'u' {
			return fmt.Errorf("unpaired high surrogate")
		}
		low, err := strconv.ParseUint(string(raw[index+3:index+7]), 16, 16)
		if err != nil || low < 0xdc00 || low > 0xdfff {
			return fmt.Errorf("invalid surrogate pair")
		}
		index += 6
	}
	return nil
}
