use std::borrow::Cow;

use rnix::TextRange;

mod all;
use all::all;

mod single;
use single::single;

type Source<'a> = Cow<'a, str>;

#[derive(Debug)]
pub struct FixResult<'a> {
    pub src: Source<'a>,
    pub fixed: Vec<Fixed>,
}

#[derive(Debug, Clone)]
pub struct Fixed {
    pub at: TextRange,
    pub code: u32,
}

impl<'a> FixResult<'a> {
    fn empty(src: Source<'a>) -> Self {
        Self {
            src,
            fixed: Vec::new(),
        }
    }
}

pub mod main {
    use std::borrow::Cow;

    use crate::{
        config::{Fix as FixConfig, FixOut, Single as SingleConfig},
        err::{FixErr, StatixErr},
    };

    use similar::TextDiff;

    pub fn all(fix_config: FixConfig) -> Result<(), StatixErr> {
        let vfs = fix_config.vfs()?;
        for entry in vfs.iter() {
            match (fix_config.out(), super::all(entry.contents)) {
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
                    println!("{}", &src)
                }
                (FixOut::Write, Some(fix_result)) => {
                    let path = entry.file_path;
                    std::fs::write(path, &*fix_result.src).map_err(FixErr::InvalidPath)?;
                }
                _ => (),
            };
        }
        Ok(())
    }

    pub fn single(single_config: SingleConfig) -> Result<(), StatixErr> {
        let vfs = single_config.vfs()?;
        let entry = vfs.iter().next().unwrap();
        let path = entry.file_path.display().to_string();
        let original_src = entry.contents;
        let (line, col) = single_config.position;

        match (single_config.out(), super::single(line, col, original_src)) {
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
                        .header(&old_file, &new_file)
                );
            }
            (FixOut::Stream, single_result) => {
                let src = single_result
                    .map(|r| r.src)
                    .unwrap_or(Cow::Borrowed(original_src));
                println!("{}", &src)
            }
            (FixOut::Write, Ok(single_result)) => {
                let path = entry.file_path;
                std::fs::write(path, &*single_result.src).map_err(FixErr::InvalidPath)?;
            }
            (_, Err(e)) => return Err(e.into()),
        };
        Ok(())
    }
}
