mod config;
mod err;
mod fix;
mod lint;
mod traits;

use std::io;

use crate::{
    err::{FixErr, StatixErr},
    traits::WriteDiagnostic,
};

use clap::Clap;
use config::{FixConfig, LintConfig, Opts, SubCommand};
use similar::TextDiff;

fn _main() -> Result<(), StatixErr> {
    let opts = Opts::parse();
    match opts.subcmd {
        Some(SubCommand::Fix(_)) => {
            let fix_config = FixConfig::from_opts(opts)?;
            let vfs = fix_config.vfs()?;
            for entry in vfs.iter() {
                match fix::fix(entry.contents) {
                    Ok(fix_result) => {
                        let text_diff = TextDiff::from_lines(entry.contents, &fix_result.src);
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
                    Err(e) => eprintln!("{}", FixErr::Parse(entry.file_path.to_path_buf(), e)),
                }
            }
        }
        None => {
            let lint_config = LintConfig::from_opts(opts)?;
            let vfs = lint_config.vfs()?;
            let (lints, errors): (Vec<_>, Vec<_>) =
                vfs.iter().map(lint::lint).partition(Result::is_ok);
            let lint_results = lints.into_iter().map(Result::unwrap);
            let errors = errors.into_iter().map(Result::unwrap_err);

            let mut stderr = io::stderr();
            lint_results.for_each(|r| {
                stderr.write(&r, &vfs).unwrap();
            });
            errors.for_each(|e| {
                eprintln!("{}", e);
            });
        }
    }
    Ok(())
}

fn main() {
    match _main() {
        Err(e) => eprintln!("{}", e),
        _ => (),
    }
}
