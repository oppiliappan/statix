use std::{fs, io::Write, process::Command};

use tempfile::NamedTempFile;

fn write_fixture(expression: &str) -> anyhow::Result<NamedTempFile> {
    let mut fixture = NamedTempFile::with_suffix(".nix")?;
    fixture.write_all(expression.as_bytes())?;
    fixture.write_all(b"\n")?; // otherwise diff says there's no newline at end of file

    Ok(fixture)
}

fn run_cli(path: &std::path::Path, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .args(args)
        .arg(path)
        .output()?;

    let stdout = strip_ansi_escapes::strip(output.stdout)?;
    let stdout = String::from_utf8(stdout)?;
    let stdout = stdout.replace(path.to_str().unwrap(), "<temp_file_path>");

    Ok(stdout)
}

pub fn test_cli(expression: &str, args: &[&str]) -> anyhow::Result<String> {
    let fixture = write_fixture(expression)?;
    run_cli(fixture.path(), args)
}

#[allow(dead_code)]
pub fn apply_and_check(
    expression: &str,
    fix_args: &[&str],
    check_args: &[&str],
) -> anyhow::Result<(String, String, String)> {
    let fixture = write_fixture(expression)?;
    let path = fixture.path();

    let fix_stdout = run_cli(path, fix_args)?;
    let contents = fs::read_to_string(path)?;
    let check_stdout = run_cli(path, check_args)?;

    Ok((fix_stdout, contents, check_stdout))
}
