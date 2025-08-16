use std::borrow::Cow;

use crate::LintMap;

use lib::session::SessionInfo;
use rnix::TextRange;

mod all;
use all::all_with;

mod single;
use single::single;

type Source<'a> = Cow<'a, str>;

pub struct FixResult<'a> {
    pub src: Source<'a>,
    pub fixed: Vec<Fixed>,
    pub lints: &'a LintMap,
    pub sess: &'a SessionInfo,
}

#[derive(Debug, Clone)]
pub struct Fixed {
    pub at: TextRange,
    pub code: u32,
}

impl<'a> FixResult<'a> {
    fn empty(src: Source<'a>, lints: &'a LintMap, sess: &'a SessionInfo) -> Self {
        Self {
            src,
            fixed: Vec::new(),
            lints,
            sess,
        }
    }
}

pub mod main {
    use std::borrow::Cow;

    use crate::{
        config::{
            FixOut, Single as SingleConfig, {ConfFile, Fix as FixConfig},
        },
        err::{FixErr, StatixErr},
    };

    use lib::session::SessionInfo;
    use similar::TextDiff;

    pub fn all(fix_config: &FixConfig) -> Result<(), StatixErr> {
        let conf_file = ConfFile::discover(&fix_config.conf_path)?;
        let vfs = fix_config.vfs(conf_file.ignore.as_slice())?;

        let lints = conf_file.lints();
        let version = conf_file.version()?;

        let session = SessionInfo::from_version(version);

        for entry in vfs.iter() {
            match (
                fix_config.out(),
                super::all_with(entry.contents, &lints, &session),
            ) {
                (FixOut::Diff, fix_result) => {
                    let src = fix_result
                        .map(|r| r.src)
                        .unwrap_or(Cow::Borrowed(entry.contents));
                    let text_diff = TextDiff::from_lines(entry.contents, &src);
                    let old_file = format!("{}", entry.file_path.display());
                    let new_file = format!("{} [fixed]", entry.file_path.display());
                    println!(
                        "{}",
                        text_diff
                            .unified_diff()
                            .context_radius(4)
                            .header(&old_file, &new_file)
                    );
                }
                (FixOut::Stream, fix_result) => {
                    let src = fix_result
                        .map(|r| r.src)
                        .unwrap_or(Cow::Borrowed(entry.contents));
                    println!("{}", &src);
                }
                (FixOut::Write, Some(fix_result)) => {
                    let path = entry.file_path;
                    std::fs::write(path, &*fix_result.src).map_err(FixErr::InvalidPath)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    pub fn single(single_config: &SingleConfig) -> Result<(), StatixErr> {
        let vfs = single_config.vfs()?;
        let entry = vfs.iter().next().unwrap();
        let path = entry.file_path.display().to_string();
        let original_src = entry.contents;
        let (line, col) = single_config.position;

        let conf_file = ConfFile::discover(&single_config.conf_path)?;

        let version = conf_file.version()?;

        let session = SessionInfo::from_version(version);

        match (
            single_config.out(),
            super::single(line, col, original_src, &session),
        ) {
            (FixOut::Diff, single_result) => {
                let fixed_src = single_result
                    .map(|r| r.src)
                    .unwrap_or(Cow::Borrowed(original_src));
                let text_diff = TextDiff::from_lines(original_src, &fixed_src);
                let old_file = &path;
                let new_file = format!("{} [fixed]", &path);
                println!(
                    "{}",
                    text_diff
                        .unified_diff()
                        .context_radius(4)
                        .header(old_file, &new_file)
                );
            }
            (FixOut::Stream, single_result) => {
                let src = single_result
                    .map(|r| r.src)
                    .unwrap_or(Cow::Borrowed(original_src));
                println!("{}", &src);
            }
            (FixOut::Write, Ok(single_result)) => {
                let path = entry.file_path;
                std::fs::write(path, &*single_result.src).map_err(FixErr::InvalidPath)?;
            }
            (_, Err(e)) => return Err(e.into()),
        }
        Ok(())
    }
}
