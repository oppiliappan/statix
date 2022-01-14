use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigErr {
    #[error("error parsing ignore list `{0}`")]
    InvalidGlob(#[from] ignore::Error),
    #[error("path error: {0}")]
    InvalidPath(#[from] io::Error),
    #[error("unable to parse `{0}` as line and column")]
    InvalidPosition(String),
    #[error("unable to parse `{0}` as warning code")]
    InvalidWarningCode(String),
    #[error("unable to parse config file: {0}")]
    ConfFileParse(toml::de::Error),
    #[error("unable to parse nix version: `{0}`")]
    ConfFileVersionParse(String),
}

// #[derive(Error, Debug)]
// pub enum LintErr {
//     #[error("[{0}] syntax error: {1}")]
//     Parse(PathBuf, ParseError),
// }

#[derive(Error, Debug)]
pub enum FixErr {
    // #[error("[{0}] syntax error: {1}")]
    // Parse(PathBuf, ParseError),
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
pub enum ExplainErr {
    #[error("lint with code `{0}` not found")]
    LintNotFound(u32),
}

#[derive(Error, Debug)]
pub enum StatixErr {
    // #[error("linter error: {0}")]
    // Lint(#[from] LintErr),
    #[error("fixer error: {0}")]
    Fix(#[from] FixErr),
    #[error("single fix error: {0}")]
    Single(#[from] SingleFixErr),
    #[error("config error: {0}")]
    Config(#[from] ConfigErr),
    #[error("explain error: {0}")]
    Explain(#[from] ExplainErr),
}
