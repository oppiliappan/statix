use std::{default::Default, fs, path::PathBuf, str::FromStr};

use clap::Clap;
use globset::{GlobBuilder, GlobSetBuilder};
use vfs::ReadOnlyVfs;

use crate::err::ConfigErr;

/// Static analysis and linting for the nix programming language
#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "Akshay <nerdy@peppe.rs>")]
pub struct Opts {
    /// File or directory to run statix on
    #[clap(default_value = ".")]
    target: String,

    // /// Path to statix config
    // #[clap(short, long, default_value = ".statix.toml")]
    // config: String,
    /// Regex of file patterns to not lint
    #[clap(short, long)]
    ignore: Vec<String>,

    /// Output format. Supported values: json, errfmt
    #[clap(short = 'o', long)]
    format: Option<OutFormat>,

    #[clap(subcommand)]
    pub subcmd: Option<SubCommand>,
}

#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "Akshay <nerdy@peppe.rs>")]
pub enum SubCommand {
    /// Find and fix issues raised by statix
    Fix(Fix),
}

#[derive(Clap, Debug)]
pub struct Fix {
    /// Do not write to files, display a diff instead
    #[clap(short = 'd', long = "dry-run")]
    diff_only: bool,
}

#[derive(Debug, Copy, Clone)]
pub enum OutFormat {
    Json,
    Errfmt,
    StdErr,
}

impl Default for OutFormat {
    fn default() -> Self {
        OutFormat::StdErr
    }
}

impl FromStr for OutFormat {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "errfmt" => Ok(Self::Errfmt),
            "stderr" => Ok(Self::StdErr),
            _ => Err("unknown output format, try: json, errfmt"),
        }
    }
}

#[derive(Debug)]
pub struct LintConfig {
    pub files: Vec<PathBuf>,
    pub format: OutFormat,
}

impl LintConfig {
    pub fn from_opts(opts: Opts) -> Result<Self, ConfigErr> {
        let ignores = {
            let mut set = GlobSetBuilder::new();
            for pattern in opts.ignore {
                let glob = GlobBuilder::new(&pattern).build().map_err(|err| {
                    ConfigErr::InvalidGlob(err.glob().map(|i| i.to_owned()), err.kind().clone())
                })?;
                set.add(glob);
            }
            set.build().map_err(|err| {
                ConfigErr::InvalidGlob(err.glob().map(|i| i.to_owned()), err.kind().clone())
            })
        }?;

        let walker = dirs::Walker::new(opts.target).map_err(ConfigErr::InvalidPath)?;

        let files = walker
            .filter(|path| matches!(path.extension(), Some(e) if e == "nix"))
            .filter(|path| !ignores.is_match(path))
            .collect();
        Ok(Self {
            files,
            format: opts.format.unwrap_or_default(),
        })
    }

    pub fn vfs(&self) -> Result<ReadOnlyVfs, ConfigErr> {
        let mut vfs = ReadOnlyVfs::default();
        for file in self.files.iter() {
            let _id = vfs.alloc_file_id(&file);
            let data = fs::read_to_string(&file).map_err(ConfigErr::InvalidPath)?;
            vfs.set_file_contents(&file, data.as_bytes());
        }
        Ok(vfs)
    }
}

mod dirs {
    use std::{
        fs,
        io::{self, Error, ErrorKind},
        path::{Path, PathBuf},
    };

    #[derive(Default, Debug)]
    pub struct Walker {
        dirs: Vec<PathBuf>,
        files: Vec<PathBuf>,
    }

    impl Walker {
        pub fn new<P: AsRef<Path>>(target: P) -> io::Result<Self> {
            let target = target.as_ref().to_path_buf();
            if !target.exists() {
                Err(Error::new(
                    ErrorKind::NotFound,
                    format!("file not found: {}", target.display()),
                ))
            } else if target.is_dir() {
                Ok(Self {
                    dirs: vec![target],
                    ..Default::default()
                })
            } else {
                Ok(Self {
                    files: vec![target],
                    ..Default::default()
                })
            }
        }
    }

    impl Iterator for Walker {
        type Item = PathBuf;
        fn next(&mut self) -> Option<Self::Item> {
            if let Some(dir) = self.dirs.pop() {
                if dir.is_dir() {
                    for entry in fs::read_dir(dir).ok()? {
                        let entry = entry.ok()?;
                        let path = entry.path();
                        if path.is_dir() {
                            self.dirs.push(path);
                        } else if path.is_file() {
                            self.files.push(path);
                        }
                    }
                }
            }
            self.files.pop()
        }
    }
}
