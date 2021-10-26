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
    #[error("unable to parse `{0}` as line and column")]
    InvalidPosition(String),
}

#[derive(Error, Debug)]
pub enum LintErr {
    #[error("[{0}] syntax error: {1}")]
    Parse(PathBuf, ParseError),
}

#[derive(Error, Debug)]
pub enum FixErr {
    #[error("[{0}] syntax error: {1}")]
    Parse(PathBuf, ParseError),
    #[error("path error: {0}")]
    InvalidPath(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum SingleFixErr {
    #[error("path error: {0}")]
    InvalidPath(#[from] io::Error),
    #[error("position out of bounds: line {0}, col {1}")]
    OutOfBounds(usize, usize),
    #[error("{0} is too large")]
    Conversion(usize),
    #[error("nothing to fix")]
    NoOp,
}

#[derive(Error, Debug)]
pub enum StatixErr {
    #[error("linter error: {0}")]
    Lint(#[from] LintErr),
    #[error("fixer error: {0}")]
    Fix(#[from] FixErr),
    #[error("single fix error: {0}")]
    Single(#[from] SingleFixErr),
    #[error("config error: {0}")]
    Config(#[from] ConfigErr),
}
