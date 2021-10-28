use std::{
    default::Default,
    fmt, fs, io,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Clap;
use globset::{Error as GlobError, GlobBuilder, GlobSet, GlobSetBuilder};
use vfs::ReadOnlyVfs;

use crate::err::ConfigErr;

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
}

#[derive(Clap, Debug)]
pub struct Check {
    /// File or directory to run check on
    #[clap(default_value = ".", parse(from_os_str))]
    target: PathBuf,

    /// Globs of file patterns to skip
    #[clap(short, long)]
    ignore: Vec<String>,

    /// Output format.
    /// Supported values: stderr, errfmt, json (on feature flag only)
    #[clap(short = 'o', long, default_value_t, parse(try_from_str))]
    pub format: OutFormat,
}

impl Check {
    pub fn vfs(&self) -> Result<ReadOnlyVfs, ConfigErr> {
        let files = walk_with_ignores(&self.ignore, &self.target)?;
        vfs(files)
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

    /// Do not fix files in place, display a diff instead
    #[clap(short, long = "dry-run")]
    pub diff_only: bool,
}

impl Fix {
    pub fn vfs(&self) -> Result<ReadOnlyVfs, ConfigErr> {
        let files = walk_with_ignores(&self.ignore, &self.target)?;
        vfs(files)
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

fn parse_line_col(src: &str) -> Result<(usize, usize), ConfigErr> {
    let parts = src.split(",");
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

fn build_ignore_set(ignores: &[String]) -> Result<GlobSet, GlobError> {
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

fn walk_with_ignores<P: AsRef<Path>>(
    ignores: &[String],
    target: P,
) -> Result<Vec<PathBuf>, ConfigErr> {
    let ignores = build_ignore_set(ignores).map_err(|err| {
        ConfigErr::InvalidGlob(err.glob().map(|i| i.to_owned()), err.kind().clone())
    })?;

    Ok(walk_nix_files(&target)?
        .filter(|path| !ignores.is_match(path))
        .collect())
}

fn vfs(files: Vec<PathBuf>) -> Result<ReadOnlyVfs, ConfigErr> {
    let mut vfs = ReadOnlyVfs::default();
    for file in files.iter() {
        let _id = vfs.alloc_file_id(&file);
        let data = fs::read_to_string(&file).map_err(ConfigErr::InvalidPath)?;
        vfs.set_file_contents(&file, data.as_bytes());
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
