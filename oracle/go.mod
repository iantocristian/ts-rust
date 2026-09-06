module github.com/microsoft/TypeScript/tsc/oracle

go 1.26

require github.com/microsoft/TypeScript/tsc v0.0.0

require (
	github.com/go-json-experiment/json v0.0.0-20260623181947-01eb4420fa68 // indirect
	github.com/zeebo/xxh3 v1.1.0 // indirect
	golang.org/x/sync v0.21.0 // indirect
	golang.org/x/text v0.38.0 // indirect
)

replace github.com/microsoft/TypeScript/tsc => ../upstream/tsc
