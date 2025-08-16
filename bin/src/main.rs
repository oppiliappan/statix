use clap::Parser;

#[rustfmt::skip]
use statix::{
    config::{Opts, SubCommand},
    err::StatixErr,
    lint, fix, explain, dump, list,
};

fn _main() -> Result<(), StatixErr> {
    let opts = Opts::parse();
    match opts.cmd {
        SubCommand::Check(config) => lint::main::main(&config),
        SubCommand::Fix(config) => fix::main::all(&config),
        SubCommand::Single(config) => fix::main::single(&config),
        SubCommand::Explain(config) => explain::main::main(&config),
        SubCommand::Dump(_) => dump::main::main(),
        SubCommand::List(_) => list::main::main(),
    }
}

fn main() {
    if let Err(e) = _main() {
        eprintln!("{e}");
    }
}
