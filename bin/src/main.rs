mod config;
mod err;
mod lint;
mod traits;

use std::io;

use crate::{err::StatixErr, traits::WriteDiagnostic};

use clap::Clap;
use config::{LintConfig, Opts, SubCommand};

fn _main() -> Result<(), StatixErr> {
    let opts = Opts::parse();
    match opts.subcmd {
        Some(SubCommand::Fix(_)) => {
            eprintln!("`fix` not yet supported");
        }
        None => {
            let lint_config = LintConfig::from_opts(opts)?;
            let vfs = lint_config.vfs()?;
            let (reports, errors): (Vec<_>, Vec<_>) =
                vfs.iter().map(lint::lint).partition(Result::is_ok);
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
