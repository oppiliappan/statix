use std::{
    default::Default,
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Clap;
use globset::{Error as GlobError, GlobBuilder, GlobSet, GlobSetBuilder};
use vfs::ReadOnlyVfs;

use crate::err::ConfigErr;

/// Lints and suggestions for the Nix programming language
#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "Akshay <nerdy@peppe.rs>")]
pub struct Opts {
    /// File or directory to run statix on
    #[clap(default_value = ".")]
    pub target: String,

    /// Globs of file patterns to skip
    #[clap(short, long)]
    pub ignore: Vec<String>,

    /// Output format.
    /// Supported values: errfmt, json (on feature flag only)
    #[clap(short = 'o', long)]
    format: Option<OutFormat>,

    /// Find and fix issues raised by statix
    #[clap(short = 'f', long)]
    pub fix: bool,

    /// Do not fix files in place, display a diff instead
    #[clap(short = 'd', long = "dry-run")]
    diff_only: bool,
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

impl FromStr for OutFormat {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            #[cfg(feature = "json")] "json" => Ok(Self::Json),
            "errfmt" => Ok(Self::Errfmt),
            "stderr" => Ok(Self::StdErr),
            "json" => Err("statix was not compiled with the `json` feature flag"),
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
        let ignores = build_ignore_set(&opts.ignore).map_err(|err| {
            ConfigErr::InvalidGlob(err.glob().map(|i| i.to_owned()), err.kind().clone())
        })?;

        let files = walk_nix_files(&opts.target)?
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

pub struct FixConfig {
    pub files: Vec<PathBuf>,
    pub diff_only: bool,
}

impl FixConfig {
    pub fn from_opts(opts: Opts) -> Result<Self, ConfigErr> {
        let ignores = build_ignore_set(&opts.ignore).map_err(|err| {
            ConfigErr::InvalidGlob(err.glob().map(|i| i.to_owned()), err.kind().clone())
        })?;

        let files = walk_nix_files(&opts.target)?
            .filter(|path| !ignores.is_match(path))
            .collect();

        let diff_only = opts.diff_only;
        Ok(Self { files, diff_only })
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

fn build_ignore_set(ignores: &Vec<String>) -> Result<GlobSet, GlobError> {
    let mut set = GlobSetBuilder::new();
    for pattern in ignores {
        let glob = GlobBuilder::new(&pattern).build()?;
        set.add(glob);
    }
    set.build()
}

fn walk_nix_files<P: AsRef<Path>>(target: P) -> Result<impl Iterator<Item = PathBuf>, io::Error> {
    let walker = dirs::Walker::new(target)?;
    Ok(walker.filter(|path: &PathBuf| matches!(path.extension(), Some(e) if e == "nix")))
}
