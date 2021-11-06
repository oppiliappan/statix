use std::{default::Default, fmt, fs, path::PathBuf, str::FromStr};

use crate::{dirs, err::ConfigErr};

use clap::Clap;
use vfs::ReadOnlyVfs;

#[derive(Clap, Debug)]
#[clap(version, author, about)]
pub struct Opts {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    /// Lints and suggestions for the nix programming language
    Check(Check),
    /// Find and fix issues raised by statix-check
    Fix(Fix),
    /// Fix exactly one issue at provided position
    Single(Single),
    /// Print detailed explanation for a lint warning
    Explain(Explain),
}

#[derive(Clap, Debug)]
pub struct Check {
    /// File or directory to run check on
    #[clap(default_value = ".", parse(from_os_str))]
    target: PathBuf,

    /// Globs of file patterns to skip
    #[clap(short, long)]
    ignore: Vec<String>,

    /// Don't respect .gitignore files
    #[clap(short, long)]
    unrestricted: bool,

    /// Output format.
    /// Supported values: stderr, errfmt, json (on feature flag only)
    #[clap(short = 'o', long, default_value_t, parse(try_from_str))]
    pub format: OutFormat,

    /// Enable "streaming" mode, accept file on stdin, output diagnostics on stdout
    #[clap(short, long = "stdin")]
    pub streaming: bool,
}

impl Check {
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
            let ignore = dirs::build_ignore_set(&self.ignore, &self.target, self.unrestricted)?;
            let files = dirs::walk_nix_files(ignore, &self.target)?;
            vfs(files.collect::<Vec<_>>())
        }
    }
}

#[derive(Clap, Debug)]
pub struct Fix {
    /// File or directory to run fix on
    #[clap(default_value = ".", parse(from_os_str))]
    target: PathBuf,

    /// Globs of file patterns to skip
    #[clap(short, long)]
    ignore: Vec<String>,

    /// Don't respect .gitignore files
    #[clap(short, long)]
    unrestricted: bool,

    /// Do not fix files in place, display a diff instead
    #[clap(short, long = "dry-run")]
    pub diff_only: bool,

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
            let ignore = dirs::build_ignore_set(&self.ignore, &self.target, self.unrestricted)?;
            let files = dirs::walk_nix_files(ignore, &self.target)?;
            vfs(files.collect::<Vec<_>>())
        }
    }

    // i need this ugly helper because clap's data model
    // does not reflect what i have in mind
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

#[derive(Clap, Debug)]
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

#[derive(Clap, Debug)]
pub struct Explain {
    /// Warning code to explain
    #[clap(parse(try_from_str = parse_warning_code))]
    pub target: u32,
}

fn parse_line_col(src: &str) -> Result<(usize, usize), ConfigErr> {
    let parts = src.split(',');
    match parts.collect::<Vec<_>>().as_slice() {
        [line, col] => {
            let l = line
                .parse::<usize>()
                .map_err(|_| ConfigErr::InvalidPosition(src.to_owned()))?;
            let c = col
                .parse::<usize>()
                .map_err(|_| ConfigErr::InvalidPosition(src.to_owned()))?;
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

fn vfs(files: Vec<PathBuf>) -> Result<ReadOnlyVfs, ConfigErr> {
    let mut vfs = ReadOnlyVfs::default();
    for file in files.iter() {
        if let Ok(data) = fs::read_to_string(&file) {
            let _id = vfs.alloc_file_id(&file);
            vfs.set_file_contents(&file, data.as_bytes());
        } else {
            println!("{} contains non-utf8 content", file.display());
        };
    }
    Ok(vfs)
}

#[derive(Debug, Copy, Clone)]
pub enum OutFormat {
    #[cfg(feature = "json")]
    Json,
    Errfmt,
    StdErr,
}

impl Default for OutFormat {
    fn default() -> Self {
        OutFormat::StdErr
    }
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
