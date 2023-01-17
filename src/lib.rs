#![warn(missing_docs)]

//! Record code coverage traces in the [drcov format](https://www.ayrx.me/drcov-file-format/).
//!
//! ```rust
//! use drcov_rs::{Module, Trace};
//!
//! // Create a new trace with the given module information.
//! let modules = [
//!     Module::new("abcd", 0x1000, 0x2000),
//!     Module::new("libc.so", 0x555000, 0x556000),
//! ];
//! let mut trace = Trace::new(&modules);
//!
//! // Add coverage events from your emulator, debugger, etc.
//! trace.add(0x1204, 3);
//! trace.add(0x1207, 12);
//!
//! // Save the code coverage to a file in drcov format.
//! trace.save("trace.log").unwrap();
//! ```

use std::io::{Error, Write};

/// The version of the drcov format to use.
#[derive(Clone, Copy, Debug)]
pub enum Version {
    /// Drcov version 2.
    V2,
    /// Drcov version 3.
    V3,
}

impl Default for Version {
    fn default() -> Self {
        Version::V2
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Version::V2 => write!(f, "2"),
            Version::V3 => write!(f, "3"),
        }
    }
}

/// Represents a collection of code coverage information.
#[derive(Clone, Debug)]
pub struct Trace {
    /// Collection of all modules added to the trace.
    modules: Vec<Module>,
    /// Collection of all basic block entries recorded in the trace.
    entries: Vec<BasicBlockEntry>,
    /// A string used to describe the tool that generated the coverage information.
    flavor: &'static str,
    /// The drcov file format version to use.
    version: Version,
}

impl Trace {
    /// Create a new drcov trace using the given modules.
    ///
    /// # Arguments
    ///
    /// * `modules` - An array of modules to add to the trace.
    pub fn new(modules: &[Module]) -> Trace {
        Trace {
            modules: modules.to_vec(),
            entries: Vec::new(),
            flavor: "drcov",
            version: Version::default(),
        }
    }

    /// Returns a reference to the [`Module`] containing the given address or None if an unknown address.
    pub fn get_module(&self, address: u64) -> Option<&Module> {
        self.modules.iter().find(|m| m.contains(address))
    }

    /// Add a new coverage entry to the [`Trace`].
    ///
    /// # Arguments
    ///
    /// * `address` - The start address of the basic block to record.
    /// * `size` - The size of the basic block in bytes.
    pub fn add(&mut self, address: u64, size: usize) {
        let entry = self
            .modules
            .iter()
            .enumerate()
            .find(|(_, m)| m.contains(address))
            .map(|(id, module)| BasicBlockEntry {
                start: (address - module.base).try_into().unwrap(),
                size: size
                    .try_into()
                    .expect("Entry size is too large (u16::MAX < entry)"),
                mod_id: id as u16,
            })
            .expect("No module found that contains that address");

        self.entries.push(entry);
    }

    /// Output the coverage information in the appropriate drcov format.
    pub fn write(&self, writer: &mut impl Write) -> Result<(), Error> {
        // Write the drcov header.
        writeln!(writer, "DRCOV VERSION: {}", self.version)?;
        writeln!(writer, "DRCOV FLAVOR: {}", self.flavor)?;

        // Write the module table.
        writeln!(
            writer,
            "Module Table: version 4, count {}",
            self.modules.len()
        )?;
        writeln!(
            writer,
            "Columns: id, containing_id, start, end, entry, offset, path"
        )?;

        for (id, Module { name, base, end }) in self.modules.iter().enumerate() {
            writeln!(writer, "{id}, 0, {base:#x}, {end:#x}, 0, 0, {name}")?;
        }

        // Write the basic block entries.
        writeln!(writer, "BB Table: {} bbs", self.entries.len())?;
        for entry in &self.entries {
            entry.write(writer)?;
        }

        Ok(())
    }

    /// Save the coverage trace to a file at the given path.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        let mut file = std::fs::File::create(path)?;
        self.write(&mut file)
    }
}

/// Contains information about a single module in the program's address space.
#[derive(Clone, Copy, Debug)]
pub struct Module {
    name: &'static str,
    base: u64,
    end: u64,
}

impl Module {
    /// Create a new [`Module`].
    ///
    /// # Panics
    ///
    /// This function will panic if the end address is smaller than the base address.
    pub fn new(name: &'static str, base: u64, end: u64) -> Module {
        assert!(base < end, "`base` must be before `end`");
        assert!(
            (end - base) <= u32::MAX as u64,
            "Module sizes > u32::MAX are not representable"
        );
        Module { name, base, end }
    }

    /// Returns true if the given address is within this `Module`.
    pub fn contains(&self, address: u64) -> bool {
        self.base <= address && address < self.end
    }
}

/// Represents a single executed basic block in a module.
#[derive(Clone, Copy, Debug, Default)]
struct BasicBlockEntry {
    /// Offset of the basic block start from the image base.
    start: u32,
    /// Size of the basic block.
    size: u16,
    /// Id of the module where the basic block is located.
    mod_id: u16,
}

impl BasicBlockEntry {
    fn write(&self, writer: &mut impl Write) -> Result<(), Error> {
        let mut buf = [0; 8];

        buf[0..4].copy_from_slice(&self.start.to_ne_bytes());
        buf[4..6].copy_from_slice(&self.size.to_ne_bytes());
        buf[6..8].copy_from_slice(&self.mod_id.to_ne_bytes());

        writer.write_all(&buf)
    }
}

// Ensure at compile-time that entry structs are 8 bytes in size.
static_assertions::assert_eq_size!(u64, BasicBlockEntry);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        let modules = [Module::new("abcd.so", 0x1000, 0x2000)];
        let mut trace = Trace::new(&modules);

        trace.add(0x1234, 1);

        let mut out = Vec::new();
        trace.write(&mut out).unwrap();

        assert!(!out.is_empty());
    }

    #[test]
    #[should_panic]
    fn add_out_of_bounds() {
        let modules = [Module::new("abcd.so", 0x1000, 0x2000)];
        let mut trace = Trace::new(&modules);

        trace.add(0xdead, 10);
    }
}
