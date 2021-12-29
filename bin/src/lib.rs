pub mod config;
pub mod dirs;
pub mod err;
pub mod explain;
pub mod fix;
pub mod lint;
pub mod session;
pub mod traits;

mod utils;

use std::collections::HashMap;

use lib::Lint;
use rnix::SyntaxKind;

pub type LintMap = HashMap<SyntaxKind, Vec<&'static Box<dyn Lint>>>;
