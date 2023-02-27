mod bundlers;
mod error;
mod libraries;
mod options;

pub use error::*;
pub use options::*;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[clap(version = "1.0", author = "feenk gmbh <contact@feenk.com>")]
pub struct Options {
    /// A level of verbosity, and can be used multiple times
    #[clap(long, parse(from_os_str))]
    lib: PathBuf,
    #[clap(long)]
    path: Option<String>,
}

fn main() {
    let options: Options = Options::parse();
    bundlers::mac::MacBundler::set_rpath_to(&options.lib, options.path.unwrap_or("".to_string()))
        .unwrap();
}
