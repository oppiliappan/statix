use std::{
    fs,
    io::{self, Error, ErrorKind},
    path::{Path, PathBuf},
};

use crate::dirs;

use ignore::{
    Error as IgnoreError, Match,
    gitignore::{Gitignore, GitignoreBuilder},
};

#[derive(Debug)]
pub struct Walker {
    dirs: Vec<PathBuf>,
    files: Vec<PathBuf>,
    ignore: Gitignore,
}

impl Walker {
    pub fn new<P: AsRef<Path>>(targets: &[P], ignore: Gitignore) -> io::Result<Self> {
        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for target in targets {
            let target_path = target.as_ref().to_path_buf();
            if !target_path.exists() {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("file not found: {}", target_path.display()),
                ));
            } else if target_path.is_dir() {
                dirs.push(target_path);
            } else {
                files.push(target_path);
            }
        }

        Ok(Self {
            dirs,
            files,
            ignore,
        })
    }
}

impl Iterator for Walker {
    type Item = PathBuf;
    fn next(&mut self) -> Option<Self::Item> {
        self.files.pop().or_else(|| {
            while let Some(dir) = self.dirs.pop() {
                if dir.is_dir() {
                    if let Match::None | Match::Whitelist(_) = self.ignore.matched(&dir, true) {
                        let mut found = false;
                        for entry in fs::read_dir(&dir).ok()? {
                            let entry = entry.ok()?;
                            let path = entry.path();
                            if path.is_dir() {
                                self.dirs.push(path);
                            } else if path.is_file() {
                                if let Match::None | Match::Whitelist(_) =
                                    self.ignore.matched(&path, false)
                                {
                                    found = true;
                                    self.files.push(path);
                                }
                            }
                        }
                        if found {
                            break;
                        }
                    }
                }
            }
            self.files.pop()
        })
    }
}

pub fn build_ignore_set<P: AsRef<Path>>(
    ignore: &[String],
    targets: &[P],
    unrestricted: bool,
) -> Result<Gitignore, IgnoreError> {
    let mut gitignore_path = PathBuf::new();

    // Loop through targets and look for .gitignore file if target is a directory
    for target in targets {
        if !target.as_ref().is_dir() {
            continue;
        }

        let path = target.as_ref().join(".gitignore");
        if path.exists() {
            gitignore_path = path;
            break;
        }
    }

    // Looks like GitignoreBuilder::new does not source globs
    // within gitignore_path by default, we have to enforce that
    // using GitignoreBuilder::add. Probably a bug in the ignore
    // crate?
    let mut gitignore = GitignoreBuilder::new(&gitignore_path);

    // if we are to "restrict" aka "respect" .gitignore, then
    // add globs from gitignore path as well
    if !unrestricted {
        gitignore.add(&gitignore_path);

        // ignore .git by default, nobody cares about .git, i'm sure
        gitignore.add_line(None, ".git")?;
    }

    for i in ignore {
        gitignore.add_line(None, i.as_str())?;
    }

    gitignore.build()
}

pub fn walk_nix_files<P: AsRef<Path>>(
    ignore: Gitignore,
    targets: &[P],
) -> Result<impl Iterator<Item = PathBuf>, io::Error> {
    let walker = dirs::Walker::new(targets, ignore)?;
    Ok(walker.filter(|path: &PathBuf| matches!(path.extension(), Some(e) if e == "nix")))
}
