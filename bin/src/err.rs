use std::{io, path::PathBuf};

use globset::ErrorKind;
use rnix::parser::ParseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigErr {
    #[error("error parsing glob `{0:?}`: {1}")]
    InvalidGlob(Option<String>, ErrorKind),

    #[error("path error: {0}")]
    InvalidPath(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum LintErr {
    #[error("[{0}] syntax error: {1}")]
    Parse(PathBuf, ParseError),
}

#[derive(Error, Debug)]
pub enum StatixErr {
    #[error("linter error: {0}")]
    Lint(#[from] LintErr),
    #[error("config error: {0}")]
    Config(#[from] ConfigErr),
}
