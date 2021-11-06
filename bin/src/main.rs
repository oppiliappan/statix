mod config;
mod dirs;
mod err;
mod explain;
mod fix;
mod lint;
mod traits;

use crate::err::StatixErr;

use clap::Clap;
use config::{Opts, SubCommand};

fn _main() -> Result<(), StatixErr> {
    let opts = Opts::parse();
    match opts.cmd {
        SubCommand::Check(config) => lint::main::main(config),
        SubCommand::Fix(config) => fix::main::all(config),
        SubCommand::Single(config) => fix::main::single(config),
        SubCommand::Explain(config) => explain::main::main(config),
    }
}

fn main() {
    if let Err(e) = _main() {
        eprintln!("{}", e);
    }
}
