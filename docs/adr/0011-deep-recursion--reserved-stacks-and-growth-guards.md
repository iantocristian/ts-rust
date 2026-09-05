# ADR 0011: Deep recursion: reserved stacks and growth guards

Status: Accepted (2026-09-05)
Plan: section 6, decision 6; section 13, item 11

## Context

Go stacks grow on demand up to 1 GB. Corsa removed the trampolines the TypeScript-in-TypeScript compiler used: the binder, `checkBinaryLikeExpression`, the transforms and the printer recurse on the left operand of binary expressions. Rust threads have fixed stacks and a stack overflow aborts the process.

## Decision

Run parsing, checking and emit on threads created with large reserved stacks (256 MB to 1 GB, virtual, committed lazily) and add growth guards in the deepest recursive paths. Stress tests cover the full parse, bind, check, transform and emit path, left- and right-associative chains, and deep nesting of parentheses, JSX and conditional types; native and `wasm32` stack strategies are validated separately because WebAssembly stacks do not grow.

## Consequences

Whether trampolines return at the four known sites is section 13, item 11, still open; the stress tests decide it. A 100,000-term chain is one failure shape among several.

## Evidence

`tsc/internal/core/core.go` (`ApplyDebugStackLimit`), `tsc/internal/binder/binder.go` (binary expression binding), `tsc/internal/checker/checker.go` (`checkBinaryLikeExpression`), `tsc/internal/printer/printer.go` (`emitBinaryExpression`).
