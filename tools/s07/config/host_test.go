package testrunner

import "github.com/microsoft/TypeScript/tsc/internal/vfs"

type s07ConfigHost struct {
	fs  vfs.FS
	cwd string
}

func (h *s07ConfigHost) FS() vfs.FS                  { return h.fs }
func (h *s07ConfigHost) GetCurrentDirectory() string { return h.cwd }
