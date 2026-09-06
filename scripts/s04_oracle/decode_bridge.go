// Test-only access to the pinned implementation. Installed only in a disposable copy.
package internal

func S04DecodeBytes(text string) string {
	decoded, ok := decodeBytes(text)
	if !ok {
		panic("decodeBytes rejected input")
	}
	return decoded
}
