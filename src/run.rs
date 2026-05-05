use std::io::Write;
use std::{io, process::Command};

use tempfile::NamedTempFile;

use crate::{emit::Emitter, parse::Program};

pub fn run(program: Program) -> io::Result<()> {
    let mut python_file = NamedTempFile::new()?;

    let mut emitter = Emitter::new();
    let python_source = emitter.emit(&program);

    writeln!(python_file, "{}", python_source)?;
    println!("{}", python_source);
    // writeln!(python_file, "main()")?;

    let output = Command::new("python3").arg(python_file.path()).output()?;

    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;

    Ok(())
}
