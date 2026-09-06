package internal

// OracleDecodeBytes exposes the unexported file decoder (E4 fixtures).
func OracleDecodeBytes(s string) (string, bool) {
	return decodeBytes(s)
}
