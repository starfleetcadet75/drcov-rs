use drcov_rs::{Module, Trace};

fn main() {
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
}
