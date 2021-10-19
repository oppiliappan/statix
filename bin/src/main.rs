#![feature(path_try_exists)]

mod config;
mod err;
mod traits;

use std::io;

use crate::{
    err::{LintErr, StatixErr},
    traits::{LintResult, WriteDiagnostic},
};

use clap::Clap;
use config::{LintConfig, Opts, SubCommand};
use lib::LINTS;
use rnix::WalkEvent;
use vfs::VfsEntry;

fn analyze<'ρ>(vfs_entry: VfsEntry<'ρ>) -> Result<LintResult, LintErr> {
    let source = vfs_entry.contents;
    let parsed = rnix::parse(source)
        .as_result()
        .map_err(|e| LintErr::Parse(vfs_entry.file_path.to_path_buf(), e))?;
    let reports = parsed
        .node()
        .preorder_with_tokens()
        .filter_map(|event| match event {
            WalkEvent::Enter(child) => LINTS.get(&child.kind()).map(|rules| {
                rules
                    .iter()
                    .filter_map(|rule| rule.validate(&child))
                    .collect::<Vec<_>>()
            }),
            _ => None,
        })
        .flatten()
        .collect();
    Ok(LintResult {
        file_id: vfs_entry.file_id,
        reports,
    })
}

fn _main() -> Result<(), StatixErr> {
    // TODO: accept cli args, construct a CLI config with a list of files to analyze
    let opts = Opts::parse();
    match opts.subcmd {
        Some(SubCommand::Fix(_)) => {}
        None => {
            let lint_config = LintConfig::from_opts(opts)?;
            let vfs = lint_config.vfs()?;
            let (reports, errors): (Vec<_>, Vec<_>) =
                vfs.iter().map(analyze).partition(Result::is_ok);
            let lint_results: Vec<_> = reports.into_iter().map(Result::unwrap).collect();
            let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

            let mut stderr = io::stderr();
            lint_results.into_iter().for_each(|r| {
                stderr.write(&r, &vfs).unwrap();
            });
            errors.into_iter().for_each(|e| {
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
