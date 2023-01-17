# drcov-rs

Record code coverage traces in the [drcov format](https://www.ayrx.me/drcov-file-format/) (version 2).

Based on [dragondance-rs](https://github.com/evanrichter/dragondance-rs).

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
drcov-rs = "1.0"
```

## Example

```rust
use drcov_rs::{Module, Trace};

// Create a new trace with the given module information.
let modules = [
    Module::new("abcd", 0x1000, 0x2000),
    Module::new("libc.so", 0x555000, 0x556000),
];
let mut trace = Trace::new(&modules);

// Add coverage events from your emulator, debugger, etc.
trace.add(0x1204, 3);
trace.add(0x1207, 12);

// Save the code coverage to a file in drcov format.
trace.save("trace.log").unwrap();
```
