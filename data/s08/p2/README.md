# S08 P2 checkpoint evidence

`capture.tar.xz` contains the original native capture with its source snapshot,
requests, observations, command and logs, plus the Rust observation and exact
comparison. `record.json` identifies the implementation revision and archive
hash. These are named checkpoint results, not E2/E3/E4 or performance evidence.

Replay without running Go again:

```sh
mkdir -p target/s08/p2-replay
tar -xJf data/s08/p2/capture.tar.xz -C target/s08/p2-replay
python3 scripts/s08_p2.py compare --native target/s08/p2-replay/native --actual target/s08/p2-replay/rust/observations.json --output target/s08/p2-replay/verified.json
```

To run the current Rust implementation against those original observations:

```sh
cargo run -p ts_compiler --example p2_checker -- target/s08/p2-replay/native/requests.json target/s08/p2-replay/current-rust.json
python3 scripts/s08_p2.py compare --native target/s08/p2-replay/native --actual target/s08/p2-replay/current-rust.json --output target/s08/p2-replay/current-comparison.json
```

The observer was captured before later comparator validation hardening. Its
original source snapshot remains unchanged and hash-checked by replay; current
comparison additionally requires Rust ownership observations. Native program
and oracle source are unchanged. No native rerun is claimed for validator edits.
