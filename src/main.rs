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

use std::fs;
use std::io::Write;

use clap::{Parser, Subcommand};

pub use error::*;
pub use options::*;

use crate::bundlers::android::AndroidBundler;
use crate::bundlers::linux::LinuxBundler;
use crate::bundlers::mac::MacBundler;
use crate::bundlers::windows::WindowsBundler;
use crate::bundlers::Bundler;

mod bundlers;
mod error;
mod libraries;
mod options;

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "feenk gmbh <contact@feenk.com>")]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Compile executables and third-party libraries
    Compile(BuilderOptions),
    /// Bundle previously compiled artifacts
    Bundle(BuilderOptions),
    /// Compile and bundle in one go
    Build(BuilderOptions),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Compile(build_options) => compile(build_options)?,
        Command::Bundle(build_options) => bundle(build_options)?,
        Command::Build(build_options) => build(build_options)?,
    }

    Ok(())
}

fn build(build_options: BuilderOptions) -> Result<()> {
    let (bundler, bundle_options) = prepare(build_options);
    compile_components(&*bundler, &bundle_options)?;
    bundler.bundle(&bundle_options);

    Ok(())
}

fn compile(build_options: BuilderOptions) -> Result<()> {
    let (bundler, bundle_options) = prepare(build_options);
    compile_components(&*bundler, &bundle_options)
}

fn bundle(build_options: BuilderOptions) -> Result<()> {
    let (bundler, bundle_options) = prepare(build_options);
    bundler.ensure_compiled_libraries_directory(&bundle_options)?;
    bundler.bundle(&bundle_options);

    Ok(())
}

fn prepare(build_options: BuilderOptions) -> (Box<dyn Bundler>, BundleOptions) {
    let resolved_options = ResolvedOptions::new(build_options);
    let bundler = bundler(&resolved_options);

    let bundle_options = BundleOptions::new(resolved_options);

    (bundler, bundle_options)
}

fn compile_components(bundler: &dyn Bundler, bundle_options: &BundleOptions) -> Result<()> {
    bundler.ensure_compiled_libraries_directory(bundle_options)?;

    export_build_info(bundler, bundle_options)?;

    bundle_options.executables().iter().for_each(|executable| {
        let executable_options = ExecutableOptions::new(bundle_options, executable.clone());
        bundler.pre_compile(&executable_options);
        bundler.compile_binary(&executable_options);
        bundler.post_compile(bundle_options, executable, &executable_options)
    });

    bundler.compile_third_party_libraries(bundle_options)?;

    Ok(())
}

fn export_build_info(bundler: &dyn Bundler, bundle_options: &BundleOptions) -> Result<()> {
    let executables_dir = bundler.bundled_resources_directory(bundle_options);

    if !executables_dir.exists() {
        fs::create_dir_all(&executables_dir)?;
    }

    // export the info about the app and third party libs
    let json = serde_json::to_string_pretty(&bundle_options)?;
    let file_path = bundler
        .compilation_location(bundle_options)
        .join("build-info.json");

    let existing_content = if file_path.exists() {
        fs::read_to_string(&file_path).ok()
    } else {
        None
    };
    
    if existing_content.as_ref() != Some(&json) {
        let mut file = fs::File::create(&file_path)?;
        write!(&mut file, "{}", json).unwrap();
    }

    std::env::set_var("APP_BUILD_INFO", file_path.as_os_str());

    Ok(())
}

fn bundler(options: &ResolvedOptions) -> Box<dyn Bundler> {
    match options.platform() {
        Platform::Mac => Box::new(MacBundler::new()),
        Platform::Windows => Box::new(WindowsBundler::new()),
        Platform::Linux => Box::new(LinuxBundler::new()),
        Platform::Android => Box::new(AndroidBundler::new()),
    }
}
