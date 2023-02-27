use crate::{Platform, ResolvedOptions, Target};
use clap::ArgEnum;
use feenk_releaser::Version;
use serde::{Deserialize, Serialize};
use shared_library_builder::Library;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(ArgEnum, Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum Executable {
    App,
    Cli,
    Android,
}

impl Executable {
    pub fn cargo_build_command(&self) -> Command {
        let mut command = Command::new("cargo");
        if self == &Self::Android {
            command.arg("apk").arg("--");
        };

        command
            .arg("build")
            .arg("--package")
            .arg(self.cargo_package_name());

        command
    }

    /// Return the name of a package to be built depending on the executable type
    pub fn cargo_package_name(&self) -> &str {
        match self {
            Executable::App => "vm-client-desktop",
            Executable::Cli => "vm-client-desktop-cli",
            Executable::Android => "vm-client-android",
        }
    }

    pub fn cargo_bin_name(&self) -> &str {
        match self {
            Executable::App => "vm_client",
            Executable::Cli => "vm_client-cli",
            Executable::Android => "libvm_client_android",
        }
    }

    /// Return the name of the main compiled binary as it appears in the release/debug folder
    pub fn compiled_name(&self, options: &ResolvedOptions) -> String {
        let mut executable_name = self.cargo_bin_name().to_owned();

        if let Some(extension) = options.executable_artefact_extension() {
            executable_name = format!("{}.{}", &executable_name, &extension);
        };
        executable_name
    }

    pub fn bundled_name(&self, options: &ResolvedOptions) -> String {
        let mut executable_name = match self {
            Executable::App => options.executable_name().to_owned(),
            Executable::Cli => {
                format!("{}-cli", options.executable_name())
            }
            Executable::Android => {
                format!("lib{}", options.executable_name())
            }
        };

        if let Some(extension) = options.executable_artefact_extension() {
            executable_name = format!("{}.{}", &executable_name, &extension);
        };
        executable_name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleOptions {
    #[serde(flatten)]
    options: ResolvedOptions,
}

impl BundleOptions {
    pub fn new(options: ResolvedOptions) -> Self {
        Self { options }
    }

    pub fn executables(&self) -> &Vec<Executable> {
        self.options.executables()
    }

    pub fn target(&self) -> &Target {
        self.options.target()
    }

    pub fn platform(&self) -> Platform {
        self.options.platform()
    }

    pub fn target_dir(&self) -> &PathBuf {
        self.options.target_dir()
    }

    pub fn verbose(&self) -> i32 {
        self.options.verbose()
    }

    pub fn release(&self) -> bool {
        self.options.release()
    }

    pub fn icons(&self) -> &Vec<PathBuf> {
        self.options.icons()
    }

    pub fn identifier(&self) -> &str {
        self.options.identifier()
    }

    pub fn profile(&self) -> String {
        if self.release() {
            "release".to_string()
        } else {
            "debug".to_string()
        }
    }

    pub fn version(&self) -> &Version {
        self.options.version()
    }

    pub fn vmmaker_vm(&self) -> Option<&Path> {
        self.options.vmmaker_vm()
    }

    pub fn vmmaker_image(&self) -> Option<&Path> {
        self.options.vmmaker_image()
    }

    pub fn libraries(&self) -> &Vec<Box<dyn Library>> {
        self.options.libraries()
    }

    pub fn app_name(&self) -> &str {
        self.options.app_name()
    }

    pub fn compilation_location(&self) -> PathBuf {
        let mut location = self.target_dir().clone();
        if !self.target().is_current() {
            location = location.join(self.target().to_string());
        }
        location.join(self.profile())
    }

    pub fn bundle_location(&self) -> PathBuf {
        self.options.bundle_dir().map_or_else(
            || self.default_bundle_location(),
            |bundle_dir| bundle_dir.to_path_buf(),
        )
    }

    /// A name of the corresponding executable in the bundle. The name either depends on the app name
    /// or on the executable name specified by the user
    pub fn bundled_executable_name(&self, executable: &Executable) -> String {
        executable.bundled_name(&self.options)
    }

    /// A name of the corresponding executable as compiled by cargo. The name either depends on the Cargo.toml of the vm-client
    pub fn compiled_executable_name(&self, executable: &Executable) -> String {
        executable.compiled_name(&self.options)
    }

    /// A path to the compiled binary after running cargo command.
    /// It is the same as defined in the [[bin]] section of the Cargo.toml
    pub fn compiled_executable_path(&self, executable: &Executable) -> PathBuf {
        self.compilation_location()
            .join(executable.compiled_name(&self.options))
    }

    pub fn default_bundle_location(&self) -> PathBuf {
        self.target_dir()
            .join(self.target().to_string())
            .join(self.profile())
            .join("bundle")
    }

    pub fn third_party_libraries_sources_directory(&self) -> PathBuf {
        self.options
            .workspace_directory()
            .unwrap_or(std::env::current_dir().unwrap())
            .join("libs")
    }

    pub fn third_party_libraries_build_directory(&self) -> PathBuf {
        self.options.target_dir().to_path_buf()
    }
}
