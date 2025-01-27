use std::fmt::Debug;
use std::path::{Path, PathBuf};

use shared_library_builder::{Library, LibraryCompilationContext, LibraryTarget};

use crate::options::BundleOptions;
use crate::{Error, Platform, Result};
use crate::{Executable, ExecutableOptions};

pub mod android;
pub mod linux;
pub mod mac;
pub mod windows;

pub trait Bundler: Debug + Send + Sync {
    fn pre_compile(&self, _options: &ExecutableOptions) {}
    fn post_compile(
        &self,
        _bundle_options: &BundleOptions,
        _executable: &Executable,
        _executable_options: &ExecutableOptions,
    ) {
    }

    fn compile_binary(&self, options: &ExecutableOptions) {
        std::env::set_var("CARGO_TARGET_DIR", options.target_dir());
        if !options.target().is_current() {
            std::env::set_var("CARGO_TARGET", options.target().to_string());
        }

        if let Some(vmmaker_vm) = options.vmmaker_vm() {
            std::env::set_var("VM_CLIENT_VMMAKER", vmmaker_vm);
        }

        if let Some(vmmaker_image) = options.vmmaker_image() {
            std::env::set_var("VM_CLIENT_VMMAKER_IMAGE", vmmaker_image);
        }

        std::env::set_var("VM_CLIENT_VERSION", options.version().to_string());

        let mut command = options.cargo_build_command();

        if !options.target().is_current() {
            command.arg("--target").arg(options.target().to_string());
        }

        match options.verbose() {
            0 => {}
            1 => {
                command.arg("-v");
            }
            _ => {
                command.arg("-vv");
            }
        }

        if options.release() {
            command.arg("--release");
        }

        if !options.features().is_empty() {
            command.arg("--features");
            command.args(options.features());
        }

        if !command.status().unwrap().success() {
            panic!("Failed to compile a vm-client")
        }
    }

    fn bundle(&self, options: &BundleOptions);

    fn bundled_executable_directory(&self, options: &BundleOptions) -> PathBuf;
    fn bundled_resources_directory(&self, options: &BundleOptions) -> PathBuf;

    fn compile_third_party_libraries(&self, options: &BundleOptions) -> Result<()> {
        self.ensure_compiled_libraries_directory(options)?;

        for library in options.libraries() {
            self.compile_library(library, options)?;
        }

        Ok(())
    }

    fn ensure_compiled_libraries_directory(&self, options: &BundleOptions) -> Result<()> {
        let compiled_libraries_directory = self.compiled_libraries_directory(&options);

        if !compiled_libraries_directory.exists() {
            std::fs::create_dir_all(&compiled_libraries_directory).map_err(|error| {
                Error::new(format!(
                    "Could not create {}",
                    compiled_libraries_directory.display()
                ))
                .from(error)
            })?;
        }
        Ok(())
    }

    fn new_library_compilation_context(
        &self,
        library: &Box<dyn Library>,
        options: &BundleOptions,
    ) -> LibraryCompilationContext {
        let sources_directory = options
            .third_party_libraries_sources_directory()
            .join(library.name());
        if !sources_directory.exists() {
            std::fs::create_dir_all(&sources_directory)
                .unwrap_or_else(|_| panic!("Failed to create {}", &sources_directory.display()));
        }
        let build_directory = options
            .third_party_libraries_build_directory()
            .join(library.name());
        if !build_directory.exists() {
            std::fs::create_dir_all(&build_directory)
                .unwrap_or_else(|_| panic!("Failed to create {}", &build_directory.display()));
        }

        LibraryCompilationContext::new(
            sources_directory,
            build_directory,
            LibraryTarget::try_from(options.target().to_string().as_str()).unwrap(),
            !options.release(),
        )
    }

    fn compile_library(&self, library: &Box<dyn Library>, options: &BundleOptions) -> Result<()> {
        let context = self.new_library_compilation_context(library, options);
        let compiled_library = library.compile(&context)?;
        let library_target =
            LibraryTarget::try_from(options.target().to_string().as_str()).unwrap();

        let library_path = self.compiled_libraries_directory(options).join(
            library
                .compiled_library_name()
                .file_name(library.name(), &library_target, false),
        );

        std::fs::copy(&compiled_library, &library_path).map_err(|error| {
            Error::new(format!(
                "Could not copy {} to {}",
                &compiled_library.display(),
                &library_path.display(),
            ))
            .from(error)
        })?;

        Ok(())
    }

    fn bundle_location(&self, configuration: &BundleOptions) -> PathBuf {
        configuration.bundle_location()
    }

    fn compilation_location(&self, configuration: &BundleOptions) -> PathBuf {
        configuration.compilation_location()
    }

    fn default_bundle_location(&self, configuration: &BundleOptions) -> PathBuf {
        configuration.default_bundle_location()
    }

    fn compiled_libraries_directory(&self, configuration: &BundleOptions) -> PathBuf {
        self.compilation_location(configuration)
    }

    fn compiled_libraries(&self, options: &BundleOptions) -> Vec<PathBuf> {
        self.compiled_libraries_in(&self.compiled_libraries_directory(options), options)
    }

    fn compiled_libraries_in(&self, directory: &Path, options: &BundleOptions) -> Vec<PathBuf> {
        directory
            .read_dir()
            .unwrap()
            .map(|each| each.unwrap().path())
            .filter(|each| {
                let extension = each.extension().and_then(|ext| ext.to_str());
                match options.platform() {
                    Platform::Mac => extension == Some("dylib"),
                    Platform::Windows => extension == Some("dll"),
                    Platform::Linux => extension == Some("so"),
                    Platform::Android => extension == Some("so"),
                }
            })
            .collect()
    }

    fn clone_bundler(&self) -> Box<dyn Bundler>;
}
