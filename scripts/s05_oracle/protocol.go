package main

import (
	"bufio"
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"math"
	"regexp"
	"slices"
	"strconv"
	"strings"
	"unicode/utf8"

	"github.com/microsoft/TypeScript/tsc/internal/ast"
	"github.com/microsoft/TypeScript/tsc/internal/core"
	"github.com/microsoft/TypeScript/tsc/internal/diagnostics"
	"github.com/microsoft/TypeScript/tsc/internal/jsnum"
	"github.com/microsoft/TypeScript/tsc/internal/scanner"
	filedecoder "github.com/microsoft/TypeScript/tsc/internal/vfs/internal"
)

const maxRecord = 32 * 1024 * 1024

type request struct {
	ID      string
	Source  []byte
	Decode  bool
	Target  int
	Variant int
	Skip    bool
	Actions []map[string]any
}

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
	if token == nil {
		return nil, fmt.Errorf("null request value")
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
	if err != nil || len(result) > 4*1024*1024 {
		return nil, fmt.Errorf("invalid/oversized hex")
	}
	return result, nil
}
func validTarget(value int64) bool { return value >= 0 && value <= 12 || value == 99 || value == 100 }

var actionFields = func() map[string]string {
	result := map[string]string{}
	for _, op := range strings.Fields("scan scan_all snapshot rescan_less_than rescan_greater_than rescan_asterisk_equals rescan_hash rescan_question scan_jsx scan_jsx_identifier scan_jsx_attribute rescan_jsx_attribute scan_jsdoc reset mark rewind commit can_follow_jsdoc_at identifier_token valid_identifier intrinsic_jsx_name string_to_token keyword_suggestions shebang normalize_jsdoc number_from_string pseudo_bigint") {
		result[op] = "op"
	}
	for _, op := range strings.Fields("rescan_template rescan_jsx scan_jsx_ex scan_jsdoc_text set_skip_trivia set_skip_jsdoc_asterisks set_on_error") {
		result[op] = "op flag"
	}
	for op, args := range map[string]string{"rescan_slash": "report_errors", "set_text": "text_hex", "reset_pos": "pos", "reset_token_state": "pos", "set_variant": "value", "set_target": "value", "observe": "getter", "identifier_block": "first count", "identifier_point": "point", "identifier_text": "variant", "equal_fold": "other_hex", "token_to_string": "kind", "number_format": "bits", "skip_trivia": "pos options stop_after_line_break stop_at_comments in_jsdoc", "comment_ranges": "pos trailing"} {
		result[op] = "op " + args
	}
	return result
}()

func decodeRequest(input []byte) (request, error) {
	r := request{}
	if !utf8.Valid(input) {
		return r, fmt.Errorf("invalid UTF-8 JSON")
	}
	decoder := json.NewDecoder(bytes.NewReader(input))
	decoder.UseNumber()
	value, err := strictValue(decoder)
	if err != nil {
		return r, err
	}
	if _, err := decoder.Token(); err != io.EOF {
		return r, fmt.Errorf("trailing JSON")
	}
	object, ok := value.(map[string]any)
	if !ok {
		return r, fmt.Errorf("request must be an object")
	}
	if err := fields(object, "version id source_hex decode_source target variant skip_trivia actions"); err != nil {
		return r, err
	}
	version, err := integer(object["version"])
	if err != nil || version != 1 {
		return r, fmt.Errorf("unknown version")
	}
	r.ID, err = text(object["id"])
	if err != nil || r.ID == "" || utf8.RuneCountInString(r.ID) > 1024 {
		return r, fmt.Errorf("invalid id")
	}
	r.Source, err = byteString(object["source_hex"])
	if err != nil {
		return r, err
	}
	r.Decode, err = boolean(object["decode_source"])
	if err != nil {
		return r, err
	}
	r.Skip, err = boolean(object["skip_trivia"])
	if err != nil {
		return r, err
	}
	target, err := integer(object["target"])
	if err != nil || !validTarget(target) {
		return r, fmt.Errorf("invalid target")
	}
	r.Target = int(target)
	variant, err := integer(object["variant"])
	if err != nil || variant < 0 || variant > 1 {
		return r, fmt.Errorf("invalid variant")
	}
	r.Variant = int(variant)
	actions, ok := object["actions"].([]any)
	if !ok || len(actions) < 1 || len(actions) > 4096 {
		return r, fmt.Errorf("invalid action count")
	}
	marks := 0
	for _, value := range actions {
		action, ok := value.(map[string]any)
		if !ok {
			return r, fmt.Errorf("action must be object")
		}
		op, err := text(action["op"])
		if err != nil {
			return r, err
		}
		required, ok := actionFields[op]
		if !ok {
			return r, fmt.Errorf("unknown action %q", op)
		}
		if err := fields(action, required); err != nil {
			return r, err
		}
		for key, value := range action {
			switch key {
			case "op", "getter", "report_errors":
				if _, err := text(value); err != nil {
					return r, err
				}
			case "flag", "options", "stop_after_line_break", "stop_at_comments", "in_jsdoc", "trailing":
				if _, err := boolean(value); err != nil {
					return r, err
				}
			case "text_hex", "other_hex", "bits":
				if _, err := byteString(value); err != nil {
					return r, err
				}
			default:
				if _, err := integer(value); err != nil {
					return r, err
				}
			}
		}
		if op == "rescan_slash" && !slices.Contains([]string{"omitted", "false", "true"}, action["report_errors"].(string)) {
			return r, fmt.Errorf("invalid reporting mode")
		}
		if op == "observe" && !slices.Contains(strings.Fields("text token flags full_start start end token_text value range directives predicates"), action["getter"].(string)) {
			return r, fmt.Errorf("invalid getter")
		}
		if op == "set_target" && !validTarget(action["value"].(int64)) {
			return r, fmt.Errorf("invalid action target")
		}
		for _, key := range []string{"value", "variant"} {
			if op == "set_variant" || op == "identifier_text" {
				if n, ok := action[key].(int64); ok && (n < 0 || n > 1) {
					return r, fmt.Errorf("invalid action variant")
				}
			}
		}
		if op == "identifier_block" {
			first, count := action["first"].(int64), action["count"].(int64)
			if first < 0 || first > 0x10ffff || count < 1 || count > 4096 || first+count > 0x110000 {
				return r, fmt.Errorf("invalid point block")
			}
		}
		if op == "identifier_point" {
			n := action["point"].(int64)
			if n < math.MinInt32 || n > math.MaxInt32 {
				return r, fmt.Errorf("invalid rune")
			}
		}
		if op == "token_to_string" {
			n := action["kind"].(int64)
			if n < 0 || n >= 351 {
				return r, fmt.Errorf("invalid kind")
			}
		}
		if op == "number_format" && len(action["bits"].(string)) != 16 {
			return r, fmt.Errorf("invalid float bits")
		}
		if op == "mark" {
			marks++
		}
		if op == "rewind" || op == "commit" {
			marks--
			if marks < 0 {
				return r, fmt.Errorf("checkpoint underflow")
			}
		}
		r.Actions = append(r.Actions, action)
	}
	if marks != 0 {
		return r, fmt.Errorf("unclosed checkpoint")
	}
	return r, nil
}

func bytesHex(text string) string { return hex.EncodeToString([]byte(text)) }
func directives(s *scanner.Scanner) []map[string]any {
	result := []map[string]any{}
	for _, directive := range s.CommentDirectives() {
		result = append(result, map[string]any{"start": directive.Loc.Pos(), "end": directive.Loc.End(), "kind": int32(directive.Kind)})
	}
	return result
}
func predicates(s *scanner.Scanner) []bool {
	return []bool{s.HasUnicodeEscape(), s.HasExtendedUnicodeEscape(), s.HasPrecedingLineBreak(), s.HasPrecedingJSDocComment(), s.HasPrecedingJSDocLeadingAsterisks(), s.HasPrecedingJSDocWithDeprecatedTag(), s.HasPrecedingJSDocWithSeeOrLink()}
}
func snapshot(s *scanner.Scanner, kind ast.Kind) any {
	return map[string]any{"kind": int(kind), "token": int(s.Token()), "full_start": s.TokenFullStart(), "start": s.TokenStart(), "end": s.TokenEnd(), "text_hex": bytesHex(s.TokenText()), "value_hex": bytesHex(s.TokenValue()), "flags": int32(s.TokenFlags()), "range": []int{s.TokenRange().Pos(), s.TokenRange().End()}, "predicates": predicates(s), "directives": directives(s)}
}
func numberValue(number jsnum.Number) any {
	n := float64(number)
	class := "finite"
	var bits any = fmt.Sprintf("%016x", math.Float64bits(n))
	if math.IsNaN(n) {
		class = "nan"
		bits = nil
	} else if math.IsInf(n, 1) {
		class = "positive_infinity"
	} else if math.IsInf(n, -1) {
		class = "negative_infinity"
	}
	return map[string]any{"number_class": class, "bits": bits, "text_hex": bytesHex(number.String())}
}

type runner struct {
	s           *scanner.Scanner
	marks       []scanner.ScannerState
	diagnostics []map[string]any
	callback    scanner.ErrorCallback
}

func newRunner(r request) *runner {
	value := &runner{s: scanner.NewScanner(), diagnostics: []map[string]any{}}
	value.callback = func(message *diagnostics.Message, start, length int, args ...any) {
		arguments := []map[string]any{}
		for _, arg := range args {
			switch arg := arg.(type) {
			case string:
				arguments = append(arguments, map[string]any{"kind": "string", "hex": bytesHex(arg)})
			case int:
				arguments = append(arguments, map[string]any{"kind": "integer", "value": arg})
			case int32:
				arguments = append(arguments, map[string]any{"kind": "integer", "value": arg})
			case int64:
				arguments = append(arguments, map[string]any{"kind": "integer", "value": arg})
			default:
				panic(fmt.Sprintf("unsupported oracle diagnostic argument %T", arg))
			}
		}
		value.diagnostics = append(value.diagnostics, map[string]any{"code": message.Code(), "category": int32(message.Category()), "key": string(message.Key()), "start": start, "length": length, "args": arguments})
	}
	source := string(r.Source)
	if r.Decode {
		source = filedecoder.S04DecodeBytes(source)
	}
	value.s.SetText(source)
	value.s.SetOnError(value.callback)
	value.s.SetScriptTarget(core.ScriptTarget(r.Target))
	value.s.SetLanguageVariant(core.LanguageVariant(r.Variant))
	value.s.SetSkipTrivia(r.Skip)
	return value
}
func (r *runner) execute(a map[string]any) any {
	s := r.s
	op := a["op"].(string)
	integer := func(key string) int { return int(a[key].(int64)) }
	flag := func(key string) bool { return a[key].(bool) }
	raw := func(key string) string { value, _ := hex.DecodeString(a[key].(string)); return string(value) }
	var kind ast.Kind
	switch op {
	case "scan", "scan_all":
		kind = s.Scan()
	case "snapshot":
		kind = s.Token()
	case "rescan_less_than":
		kind = s.ReScanLessThanToken()
	case "rescan_greater_than":
		kind = s.ReScanGreaterThanToken()
	case "rescan_asterisk_equals":
		kind = s.ReScanAsteriskEqualsToken()
	case "rescan_hash":
		kind = s.ReScanHashToken()
	case "rescan_question":
		kind = s.ReScanQuestionToken()
	case "rescan_template":
		kind = s.ReScanTemplateToken(flag("flag"))
	case "rescan_slash":
		if a["report_errors"] == "omitted" {
			kind = s.ReScanSlashToken()
		} else {
			kind = s.ReScanSlashToken(a["report_errors"] == "true")
		}
	case "rescan_jsx":
		kind = s.ReScanJsxToken(flag("flag"))
	case "scan_jsx":
		kind = s.ScanJsxToken()
	case "scan_jsx_ex":
		kind = s.ScanJsxTokenEx(flag("flag"))
	case "scan_jsx_identifier":
		kind = s.ScanJsxIdentifier()
	case "scan_jsx_attribute":
		kind = s.ScanJsxAttributeValue()
	case "rescan_jsx_attribute":
		kind = s.ReScanJsxAttributeValue()
	case "scan_jsdoc":
		kind = s.ScanJSDocToken()
	case "scan_jsdoc_text":
		kind = s.ScanJSDocCommentTextToken(flag("flag"))
	case "can_follow_jsdoc_at":
		return s.CanFollowJSDocAt()
	case "reset":
		s.Reset()
		return nil
	case "set_text":
		s.SetText(raw("text_hex"))
		return nil
	case "reset_pos":
		s.ResetPos(integer("pos"))
		return nil
	case "reset_token_state":
		s.ResetTokenState(integer("pos"))
		return nil
	case "set_skip_trivia":
		s.SetSkipTrivia(flag("flag"))
		return nil
	case "set_skip_jsdoc_asterisks":
		s.SetSkipJSDocLeadingAsterisks(flag("flag"))
		return nil
	case "set_variant":
		s.SetLanguageVariant(core.LanguageVariant(integer("value")))
		return nil
	case "set_target":
		s.SetScriptTarget(core.ScriptTarget(integer("value")))
		return nil
	case "set_on_error":
		if flag("flag") {
			s.SetOnError(r.callback)
		} else {
			s.SetOnError(nil)
		}
		return nil
	case "mark":
		r.marks = append(r.marks, s.Mark())
		return nil
	case "rewind", "commit":
		mark := r.marks[len(r.marks)-1]
		r.marks = r.marks[:len(r.marks)-1]
		if op == "rewind" {
			s.Rewind(mark)
		}
		return nil
	case "observe":
		switch a["getter"].(string) {
		case "text":
			return bytesHex(s.Text())
		case "token":
			return int(s.Token())
		case "flags":
			return int32(s.TokenFlags())
		case "full_start":
			return s.TokenFullStart()
		case "start":
			return s.TokenStart()
		case "end":
			return s.TokenEnd()
		case "token_text":
			return bytesHex(s.TokenText())
		case "value":
			return bytesHex(s.TokenValue())
		case "range":
			return []int{s.TokenRange().Pos(), s.TokenRange().End()}
		case "directives":
			return directives(s)
		case "predicates":
			return predicates(s)
		}
	case "identifier_point":
		ch := rune(integer("point"))
		return []bool{scanner.IsIdentifierStart(ch), scanner.IsIdentifierPart(ch), scanner.IsIdentifierPartEx(ch, core.LanguageVariantJSX)}
	case "identifier_block":
		first, count := integer("first"), integer("count")
		start, part, jsx := make([]byte, (count+7)/8), make([]byte, (count+7)/8), make([]byte, (count+7)/8)
		for i := 0; i < count; i++ {
			ch := rune(first + i)
			mask := byte(1 << (i % 8))
			if scanner.IsIdentifierStart(ch) {
				start[i/8] |= mask
			}
			if scanner.IsIdentifierPart(ch) {
				part[i/8] |= mask
			}
			if scanner.IsIdentifierPartEx(ch, core.LanguageVariantJSX) {
				jsx[i/8] |= mask
			}
		}
		return map[string]any{"start_hex": hex.EncodeToString(start), "part_hex": hex.EncodeToString(part), "jsx_hex": hex.EncodeToString(jsx)}
	case "equal_fold":
		return strings.EqualFold(s.Text(), raw("other_hex"))
	case "identifier_token":
		return int(scanner.GetIdentifierToken(s.Text()))
	case "valid_identifier":
		return scanner.IsValidIdentifier(s.Text())
	case "identifier_text":
		return scanner.IsIdentifierText(s.Text(), core.LanguageVariant(integer("variant")))
	case "intrinsic_jsx_name":
		return scanner.IsIntrinsicJsxName(s.Text())
	case "string_to_token":
		return int(scanner.StringToToken(s.Text()))
	case "token_to_string":
		return bytesHex(scanner.TokenToString(ast.Kind(integer("kind"))))
	case "keyword_suggestions":
		values := scanner.GetViableKeywordSuggestions()
		slices.Sort(values)
		for i, value := range values {
			values[i] = bytesHex(value)
		}
		return values
	case "shebang":
		return bytesHex(scanner.GetShebang(s.Text()))
	case "normalize_jsdoc":
		return bytesHex(scanner.S05NormalizeJSDoc(s.Text()))
	case "skip_trivia":
		var options *scanner.SkipTriviaOptions
		if flag("options") {
			options = &scanner.SkipTriviaOptions{StopAfterLineBreak: flag("stop_after_line_break"), StopAtComments: flag("stop_at_comments"), InJSDoc: flag("in_jsdoc")}
		}
		return scanner.SkipTriviaEx(s.Text(), integer("pos"), options)
	case "comment_ranges":
		values := []map[string]any{}
		sequence := scanner.GetLeadingCommentRanges(nil, s.Text(), integer("pos"))
		if flag("trailing") {
			sequence = scanner.GetTrailingCommentRanges(nil, s.Text(), integer("pos"))
		}
		for value := range sequence {
			values = append(values, map[string]any{"start": value.Pos(), "end": value.End(), "kind": int(value.Kind), "trailing_newline": value.HasTrailingNewLine})
		}
		return values
	case "number_from_string":
		return numberValue(jsnum.FromString(s.Text()))
	case "number_format":
		bits, _ := strconv.ParseUint(a["bits"].(string), 16, 64)
		return numberValue(jsnum.Number(math.Float64frombits(bits)))
	case "pseudo_bigint":
		return bytesHex(jsnum.ParsePseudoBigInt(s.Text()))
	default:
		panic("validated operation missing dispatch")
	}
	return snapshot(s, kind)
}

var boundsPanic = regexp.MustCompile(`^runtime error: (?:index out of range \[-?\d+\](?: with length \d+)?|slice bounds out of range \[(?:-?\d*)?(?::-?\d*){1,2}\](?: with (?:length|capacity) \d+)?)$`)

func (r *runner) observation(id string, index, ordinal int, a map[string]any) (value map[string]any) {
	r.diagnostics = []map[string]any{}
	value = map[string]any{"event": "observation", "id": id, "action": index, "ordinal": ordinal, "status": "ok", "value": nil}
	defer func() {
		if payload := recover(); payload != nil {
			message := fmt.Sprint(payload)
			class := "unexpected"
			var input any
			switch message {
			case "Cannot reset token state to negative position", "'ReScanAsteriskEqualsToken' should only be called on a '*='", "'reScanQuestionToken' should only be called on a '??'":
				class = "contract"
			case "Debug failure. False expression.":
				class = "upstream_assertion"
			default:
				if boundsPanic.MatchString(message) {
					class = "bounds"
				}
				if a["op"] == "pseudo_bigint" && strings.HasPrefix(message, "Failed to parse big int: ") {
					class = "invalid_bigint"
					input = bytesHex(strings.TrimSuffix(r.s.Text(), "n"))
				}
			}
			value["status"] = "panic"
			value["value"] = map[string]any{"message": message, "class": class, "input_hex": input}
		}
		value["diagnostics"] = r.diagnostics
	}()
	value["value"] = r.execute(a)
	return value
}
func writeRecord(writer io.Writer, value any) error {
	raw, err := json.Marshal(value)
	if err != nil {
		return err
	}
	if len(raw)+1 > maxRecord {
		return fmt.Errorf("oversized output record")
	}
	raw = append(raw, '\n')
	_, err = writer.Write(raw)
	return err
}
func serve(input io.Reader, output io.Writer) error {
	reader := bufio.NewReaderSize(input, 64*1024)
	for {
		line := []byte{}
		for {
			piece, err := reader.ReadSlice('\n')
			line = append(line, piece...)
			if len(line) > maxRecord {
				return fmt.Errorf("oversized request record")
			}
			if err == bufio.ErrBufferFull {
				continue
			}
			if err == io.EOF && len(line) == 0 {
				return nil
			}
			if err != nil {
				return fmt.Errorf("incomplete request framing: %w", err)
			}
			break
		}
		request, err := decodeRequest(line)
		if err != nil {
			return err
		}
		if err := writeRecord(output, map[string]any{"event": "begin", "id": request.ID, "version": 1}); err != nil {
			return err
		}
		runner := newRunner(request)
		ordinal, completed := 0, 0
		stop := false
		for index, action := range request.Actions {
			count := 0
			for {
				observation := runner.observation(request.ID, index, ordinal, action)
				if err := writeRecord(output, observation); err != nil {
					return err
				}
				ordinal++
				count++
				if observation["status"] == "panic" {
					stop = true
					break
				}
				if action["op"] != "scan_all" || runner.s.Token() == ast.KindEndOfFile {
					break
				}
				if count >= len(runner.s.Text())+2 {
					return fmt.Errorf("%s action %d exceeded scan token bound", request.ID, index)
				}
			}
			completed = index + 1
			if stop {
				break
			}
		}
		if err := writeRecord(output, map[string]any{"event": "end", "id": request.ID, "observations": ordinal, "completed_actions": completed}); err != nil {
			return err
		}
	}
}
