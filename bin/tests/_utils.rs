use std::{io::Write, process::Command};

use tempfile::NamedTempFile;

pub fn test_cli(expression: &str, args: &[&str]) -> anyhow::Result<String> {
    let mut fixture = NamedTempFile::with_suffix(".nix")?;
    fixture.write_all(expression.as_bytes())?;
    fixture.write_all(b"\n")?; // otherwise diff says there's no newline at end of file

    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .args(args)
        .arg(fixture.path())
        .output()?;

    let stdout = strip_ansi_escapes::strip(output.stdout)?;
    let stdout = String::from_utf8(stdout)?;
    let stdout = stdout.replace(fixture.path().to_str().unwrap(), "<temp_file_path>");

    Ok(stdout)
}
