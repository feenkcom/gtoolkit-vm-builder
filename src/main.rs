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

use std::io::Write;

use clap::Parser;

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

fn main() -> Result<()> {
    let build_options: BuilderOptions = BuilderOptions::parse();

    build_synchronously(build_options)?;

    Ok(())
}

fn build_synchronously(build_options: BuilderOptions) -> Result<()> {
    let resolved_options = ResolvedOptions::new(build_options);
    let bundler = bundler(&resolved_options);

    let bundle_options = BundleOptions::new(resolved_options);

    bundler.ensure_compiled_libraries_directory(&bundle_options)?;

    export_build_info(&bundler, &bundle_options)?;

    bundle_options.executables().iter().for_each(|executable| {
        let executable_options = ExecutableOptions::new(&bundle_options, executable.clone());
        bundler.pre_compile(&executable_options);
        bundler.compile_binary(&executable_options);
        bundler.post_compile(&bundle_options, executable, &executable_options)
    });

    bundler.compile_third_party_libraries(&bundle_options)?;
    bundler.bundle(&bundle_options);

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
        .compilation_location(&bundle_options)
        .join("build-info.json");

    std::env::set_var("APP_BUILD_INFO", file_path.as_os_str());

    let mut file = std::fs::File::create(file_path)?;
    writeln!(&mut file, "{}", json).unwrap();

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
