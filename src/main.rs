extern crate clap;
extern crate cmake;
extern crate crossbeam;
extern crate downloader;
extern crate feenk_releaser;
extern crate file_matcher;
extern crate flate2;
extern crate mustache;
extern crate pkg_config;
extern crate serde;
extern crate shared_library_builder;
extern crate tar;
extern crate url;
extern crate user_error;
extern crate which;
extern crate xz2;

mod bundlers;
mod error;
mod libraries;
mod options;

pub use error::*;
pub use options::*;

use clap::Parser;
use std::io::Write;

use crate::bundlers::linux::LinuxBundler;
use crate::bundlers::mac::MacBundler;
use crate::bundlers::windows::WindowsBundler;
use crate::bundlers::Bundler;
use crate::options::{BuilderOptions, BundleOptions, Executable, Target};

fn main() -> Result<()> {
    let build_options: BuilderOptions = BuilderOptions::parse();

    build_synchronously(build_options)?;

    Ok(())
}

fn build_synchronously(build_options: BuilderOptions) -> Result<()> {
    let resolved_options = ResolvedOptions::new(build_options);
    let bundler = bundler(&resolved_options);

    let bundle_options =
        BundleOptions::new(resolved_options, vec![Executable::App, Executable::Cli]);

    bundler.ensure_third_party_requirements(&bundle_options);
    bundler.ensure_compiled_libraries_directory(&bundle_options)?;

    bundle_options.executables().iter().for_each(|executable| {
        let executable_options = ExecutableOptions::new(&bundle_options, executable.clone());
        bundler.pre_compile(&executable_options);
        bundler.compile_binary(&executable_options);
        bundler.post_compile(&bundle_options, executable, &executable_options)
    });

    bundler.compile_third_party_libraries(&bundle_options)?;
    bundler.bundle(&bundle_options);

    export_build_info(&bundler, &bundle_options)?;

    Ok(())
}

fn export_build_info(bundler: &Box<dyn Bundler>, bundle_options: &BundleOptions) -> Result<()> {
    let executables_dir = bundler.bundled_resources_directory(&bundle_options);

    if !executables_dir.exists() {
        std::fs::create_dir_all(&executables_dir)?;
    }

    // export the info about the app and third party libs
    let json = serde_json::to_string_pretty(&bundle_options)?;
    let file_path = bundler
        .bundled_resources_directory(&bundle_options)
        .join("build-info.json");
    let mut file = std::fs::File::create(file_path)?;
    writeln!(&mut file, "{}", json).unwrap();

    // export the info about the vm itself
    std::fs::copy(
        bundle_options.compilation_location().join("vm-info.json"),
        bundler
            .bundled_resources_directory(&bundle_options)
            .join("vm-info.json"),
    )?;

    Ok(())
}

fn bundler(options: &ResolvedOptions) -> Box<dyn Bundler> {
    match options.target() {
        Target::X8664appleDarwin => Box::new(MacBundler::new()),
        Target::AArch64appleDarwin => Box::new(MacBundler::new()),
        Target::X8664pcWindowsMsvc => Box::new(WindowsBundler::new()),
        Target::X8664UnknownlinuxGNU => Box::new(LinuxBundler::new()),
    }
}
