use std::{
    default::Default,
    fmt, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{LintMap, dirs, err::ConfigErr, utils};

use clap::Parser;
use lib::LINTS;
use serde::{Deserialize, Serialize};
use vfs::ReadOnlyVfs;

#[derive(Parser, Debug)]
#[clap(version, author, about)]
pub struct Opts {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    /// Lints and suggestions for the nix programming language
    Check(Check),
    /// Find and fix issues raised by statix-check
    Fix(Fix),
    /// Fix exactly one issue at provided position
    Single(Single),
    /// Print detailed explanation for a lint warning
    Explain(Explain),
    /// Dump a sample config to stdout
    Dump(Dump),
    /// List all available lints
    List(List),
}

#[derive(Parser, Debug)]
pub struct Check {
    /// File or directory to run check on
    #[clap(default_value = ".", parse(from_os_str))]
    target: Vec<PathBuf>,

    /// Globs of file patterns to skip
    #[clap(short, long)]
    ignore: Vec<String>,

    /// Don't respect .gitignore files
    #[clap(short, long)]
    unrestricted: bool,

    /// Output format.
    #[cfg_attr(feature = "json", doc = "Supported values: stderr, errfmt, json")]
    #[cfg_attr(not(feature = "json"), doc = "Supported values: stderr, errfmt")]
    #[clap(short = 'o', long, default_value_t, parse(try_from_str))]
    pub format: OutFormat,

    /// Path to statix.toml or its parent directory
    #[clap(short = 'c', long = "config", default_value = ".")]
    pub conf_path: PathBuf,

    /// Enable "streaming" mode, accept file on stdin, output diagnostics on stdout
    #[clap(short, long = "stdin")]
    pub streaming: bool,
}

impl Check {
    pub fn vfs(&self, extra_ignores: &[String]) -> Result<ReadOnlyVfs, ConfigErr> {
        if self.streaming {
            use std::io::{self, BufRead};
            let src = io::stdin()
                .lock()
                .lines()
                .map(|l| l.unwrap())
                .collect::<Vec<String>>()
                .join("\n");
            Ok(ReadOnlyVfs::singleton("<stdin>", src.as_bytes()))
        } else {
            let all_ignores = [self.ignore.as_slice(), extra_ignores].concat();
            let files = dirs::walk_nix_files(&all_ignores, &self.target, self.unrestricted)?;
            Ok(vfs(&files.collect::<Vec<_>>()))
        }
    }
}

#[derive(Parser, Debug)]
pub struct Fix {
    /// File or directory to run fix on
    #[clap(default_value = ".", parse(from_os_str))]
    target: Vec<PathBuf>,

    /// Globs of file patterns to skip
    #[clap(short, long)]
    ignore: Vec<String>,

    /// Don't respect .gitignore files
    #[clap(short, long)]
    unrestricted: bool,

    /// Do not fix files in place, display a diff instead
    #[clap(short, long = "dry-run")]
    pub diff_only: bool,

    /// Path to statix.toml or its parent directory
    #[clap(short = 'c', long = "config", default_value = ".")]
    pub conf_path: PathBuf,

    /// Enable "streaming" mode, accept file on stdin, output diagnostics on stdout
    #[clap(short, long = "stdin")]
    pub streaming: bool,
}

pub enum FixOut {
    Diff,
    Stream,
    Write,
}

impl Fix {
    pub fn vfs(&self, extra_ignores: &[String]) -> Result<ReadOnlyVfs, ConfigErr> {
        if self.streaming {
            use std::io::{self, BufRead};
            let src = io::stdin()
                .lock()
                .lines()
                .map(|l| l.unwrap())
                .collect::<Vec<String>>()
                .join("\n");
            Ok(ReadOnlyVfs::singleton("<stdin>", src.as_bytes()))
        } else {
            let all_ignores = [self.ignore.as_slice(), extra_ignores].concat();
            let files = dirs::walk_nix_files(&all_ignores, &self.target, self.unrestricted)?;
            Ok(vfs(&files.collect::<Vec<_>>()))
        }
    }

    // i need this ugly helper because clap's data model
    // does not reflect what i have in mind
    #[must_use]
    pub fn out(&self) -> FixOut {
        if self.diff_only {
            FixOut::Diff
        } else if self.streaming {
            FixOut::Stream
        } else {
            FixOut::Write
        }
    }
}

#[derive(Parser, Debug)]
pub struct Single {
    /// File to run single-fix on
    #[clap(parse(from_os_str))]
    pub target: Option<PathBuf>,

    /// Position to attempt a fix at
    #[clap(short, long, parse(try_from_str = parse_line_col))]
    pub position: (usize, usize),

    /// Do not fix files in place, display a diff instead
    #[clap(short, long = "dry-run")]
    pub diff_only: bool,

    /// Enable "streaming" mode, accept file on stdin, output diagnostics on stdout
    #[clap(short, long = "stdin")]
    pub streaming: bool,

    /// Path to statix.toml or its parent directory
    #[clap(short = 'c', long = "config", default_value = ".")]
    pub conf_path: PathBuf,
}

impl Single {
    pub fn vfs(&self) -> Result<ReadOnlyVfs, ConfigErr> {
        if self.streaming {
            use std::io::{self, BufRead};
            let src = io::stdin()
                .lock()
                .lines()
                .map(|l| l.unwrap())
                .collect::<Vec<String>>()
                .join("\n");
            Ok(ReadOnlyVfs::singleton("<stdin>", src.as_bytes()))
        } else {
            let src = std::fs::read_to_string(self.target.as_ref().unwrap())
                .map_err(ConfigErr::InvalidPath)?;
            Ok(ReadOnlyVfs::singleton("<stdin>", src.as_bytes()))
        }
    }
    #[must_use]
    pub fn out(&self) -> FixOut {
        if self.diff_only {
            FixOut::Diff
        } else if self.streaming {
            FixOut::Stream
        } else {
            FixOut::Write
        }
    }
}

#[derive(Parser, Debug)]
pub struct Explain {
    /// Warning code to explain
    #[clap(parse(try_from_str = parse_warning_code))]
    pub target: u32,
}

#[derive(Parser, Debug)]
pub struct Dump {}

#[derive(Parser, Debug)]
pub struct List {}

#[derive(Debug, Copy, Clone, Default)]
pub enum OutFormat {
    #[cfg(feature = "json")]
    Json,
    Errfmt,
    #[default]
    StdErr,
}

impl fmt::Display for OutFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                #[cfg(feature = "json")]
                Self::Json => "json",
                Self::Errfmt => "errfmt",
                Self::StdErr => "stderr",
            }
        )
    }
}

impl FromStr for OutFormat {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            #[cfg(feature = "json")]
            "json" => Ok(Self::Json),
            #[cfg(not(feature = "json"))]
            "json" => Err("statix was not compiled with the `json` feature flag"),
            "errfmt" => Ok(Self::Errfmt),
            "stderr" => Ok(Self::StdErr),
            _ => Err("unknown output format, try: json, errfmt"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ConfFile {
    #[serde(default = "Vec::new")]
    disabled: Vec<String>,

    #[serde(default = "Vec::new")]
    pub ignore: Vec<String>,
}

impl ConfFile {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigErr> {
        let path = path.as_ref();
        let config_file = fs::read_to_string(path).map_err(ConfigErr::InvalidPath)?;
        toml::de::from_str(&config_file).map_err(ConfigErr::ConfFileParse)
    }
    pub fn discover<P: AsRef<Path>>(path: P) -> Result<Self, ConfigErr> {
        let cannonical_path = fs::canonicalize(path.as_ref()).map_err(ConfigErr::InvalidPath)?;
        for p in cannonical_path.ancestors() {
            let statix_toml_path = if p.is_dir() {
                p.join("statix.toml")
            } else {
                p.to_path_buf()
            };
            if statix_toml_path.exists() {
                return Self::from_path(statix_toml_path);
            }
        }
        Ok(Self::default())
    }
    #[must_use]
    pub fn dump(&self) -> String {
        let ideal_config = {
            let disabled = vec![];
            let ignore = vec![".direnv".into()];
            Self { disabled, ignore }
        };
        toml::ser::to_string_pretty(&ideal_config).unwrap()
    }
    #[must_use]
    pub fn lints(&self) -> LintMap {
        utils::lint_map_of(
            (*LINTS)
                .iter()
                .filter(|l| !self.disabled.iter().any(|check| check == l.name()))
                .copied()
                .collect::<Vec<_>>()
                .as_slice(),
        )
    }
}

fn parse_line_col(src: &str) -> Result<(usize, usize), ConfigErr> {
    let parts = src.split(',');
    match parts.collect::<Vec<_>>().as_slice() {
        [line, col] => {
            let do_parse = |val: &str| {
                val.parse::<usize>()
                    .map_err(|_| ConfigErr::InvalidPosition(src.to_owned()))
            };
            let l = do_parse(line)?;
            let c = do_parse(col)?;
            Ok((l, c))
        }
        _ => Err(ConfigErr::InvalidPosition(src.to_owned())),
    }
}

fn parse_warning_code(src: &str) -> Result<u32, ConfigErr> {
    let mut char_stream = src.chars();
    let severity = char_stream
        .next()
        .ok_or_else(|| ConfigErr::InvalidWarningCode(src.to_owned()))?
        .to_ascii_lowercase();
    match severity {
        'w' => char_stream
            .collect::<String>()
            .parse::<u32>()
            .map_err(|_| ConfigErr::InvalidWarningCode(src.to_owned())),
        _ => Ok(0),
    }
}

fn vfs(files: &[PathBuf]) -> vfs::ReadOnlyVfs {
    let mut vfs = ReadOnlyVfs::default();
    for file in files {
        if let Ok(data) = fs::read_to_string(file) {
            let _id = vfs.alloc_file_id(file);
            vfs.set_file_contents(file, data.as_bytes());
        } else {
            println!("`{}` contains non-utf8 content", file.display());
        }
    }
    vfs
}
