use std::convert::TryFrom;
use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use clap::{ArgEnum, Parser, ArgAction};
use rustc_version::version_meta;
use serde::{Deserialize, Serialize};

use crate::libraries::{ThirdPartyLibrary, VersionedThirdPartyLibraries};
use crate::Executable;

#[derive(ArgEnum, Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
#[repr(u32)]
pub enum Target {
    #[clap(name = "x86_64-apple-darwin")]
    X8664appleDarwin,
    #[clap(name = "aarch64-apple-darwin")]
    AArch64appleDarwin,
    #[clap(name = "x86_64-pc-windows-msvc")]
    X8664pcWindowsMsvc,
    #[clap(name = "aarch64-pc-windows-msvc")]
    AArch64pcWindowsMsvc,
    #[clap(name = "x86_64-unknown-linux-gnu")]
    X8664UnknownlinuxGNU,
    #[clap(name = "aarch64-unknown-linux-gnu")]
    AArch64UnknownlinuxGNU,
    #[clap(name = "aarch64-linux-android")]
    AArch64LinuxAndroid,
}

impl Target {
    pub fn for_current_platform() -> Self {
        <Target as FromStr>::from_str(&*version_meta().unwrap().host).unwrap()
    }

    pub fn platform(&self) -> Platform {
        match self {
            Target::X8664appleDarwin => Platform::Mac,
            Target::AArch64appleDarwin => Platform::Mac,
            Target::X8664pcWindowsMsvc => Platform::Windows,
            Target::AArch64pcWindowsMsvc => Platform::Windows,
            Target::X8664UnknownlinuxGNU => Platform::Linux,
            Target::AArch64UnknownlinuxGNU => Platform::Linux,
            Target::AArch64LinuxAndroid => Platform::Android,
        }
    }

    pub fn is_unix(&self) -> bool {
        self.platform().is_unix()
    }

    pub fn is_windows(&self) -> bool {
        self.platform().is_windows()
    }

    pub fn is_current(&self) -> bool {
        self.eq(&Self::for_current_platform())
    }

    pub fn possible_variants() -> Vec<String> {
        Self::value_variants()
            .iter()
            .map(|each| each.to_string())
            .collect()
    }
}

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Target as ArgEnum>::from_str(s, true)
    }
}

impl ToString for Target {
    fn to_string(&self) -> String {
        self.to_possible_value().unwrap().get_name().to_string()
    }
}

impl TryFrom<String> for Target {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        <Target as ArgEnum>::from_str(value.as_str(), true)
    }
}

impl From<Target> for String {
    fn from(target: Target) -> Self {
        target.to_string()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Platform {
    Mac,
    Windows,
    Linux,
    Android,
}

impl Platform {
    pub fn is_unix(&self) -> bool {
        match self {
            Platform::Mac | Platform::Linux | Platform::Android => true,
            Platform::Windows => false,
        }
    }

    pub fn is_windows(&self) -> bool {
        self == &Self::Windows
    }

    pub fn is_android(&self) -> bool {
        self == &Self::Android
    }
}

#[derive(Parser, Clone, Debug, Default, Serialize, Deserialize)]
#[clap(version = "1.0", author = "feenk gmbh <contact@feenk.com>")]
pub struct BuilderOptions {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    /// Build in release mode
    #[clap(long, conflicts_with = "debug")]
    release: bool,
    /// Build in debug mode
    #[clap(long, conflicts_with = "release")]
    debug: bool,
    /// Include debug symbols in the bundle
    #[clap(long)]
    include_debug_symbols: bool,
    #[clap(long, arg_enum, ignore_case = true)]
    /// To cross-compile and bundle an application for another OS
    target: Option<Target>,
    #[clap(long, parse(from_os_str))]
    #[serde(skip)]
    /// Path to directory which cargo will use as the root of build directory.
    target_dir: Option<PathBuf>,
    /// A name of the app
    #[clap(long)]
    app_name: Option<String>,
    /// An output location of the bundle. By default, a bundle is placed inside of the cargo's target dir in the following format: target/{target architecture}/{build|release}/
    #[clap(long, parse(from_os_str))]
    #[serde(skip)]
    bundle_dir: Option<PathBuf>,
    /// MacOS only. Specify a path to a plist file to be bundled with the app
    #[clap(long, parse(from_os_str))]
    plist_file: Option<PathBuf>,
    /// Change the name of the executable. By default it is the same as app_name.
    #[clap(long)]
    executable_name: Option<String>,
    /// A future version in format X.Y.Z or vX.Y.Z
    #[clap(long)]
    version: Option<String>,
    /// A unique app identifier in the reverse domain notation, for example com.example.app
    #[clap(long)]
    identifier: Option<String>,
    /// An author entity of the application (company or person)
    #[clap(long)]
    author: Option<String>,
    /// A list of icons of different sizes to package with the app. When packaging for MacOS the icons converted
    /// into one .icns icon file. If .icns file is provided it is used instead and not processed.
    #[clap(long)]
    icons: Option<Vec<String>>,
    #[clap(long, arg_enum, ignore_case = true, multiple_values = true)]
    /// Include third party libraries
    libraries: Option<Vec<ThirdPartyLibrary>>,
    #[clap(long, parse(from_os_str))]
    /// A file that describes the versions of libraries
    libraries_versions: Option<PathBuf>,
    #[clap(long, value_parser = parse_key_val::<ThirdPartyLibrary, String>, multiple_values = true)]
    /// Override a library version specified in LIBRARY=version format. Multiple libraries are allowed.
    override_library_version: Option<Vec<(ThirdPartyLibrary, String)>>,
    /// Use a specific VM to run a VMMaker, must be a path to the executable.
    /// When specified, the build will not attempt to download a VM.
    /// By default, it is assumed that the VM is a GToolkit VM.
    /// However, it is possible to specify the type of the VM using the following syntax:
    ///     gtoolkit:/path/to/vm - to use it as a GToolkit VM
    ///     pharo:/path/to/vm - to use it as a Pharo VM
    #[clap(long, parse(from_os_str), verbatim_doc_comment)]
    #[serde(skip)]
    vmmaker_vm: Option<PathBuf>,
    /// Use a specific image to build a VMMaker from, must be a path to the .image. When specified, the build will not attempt to download an image
    #[clap(long, parse(from_os_str))]
    #[serde(skip)]
    vmmaker_image: Option<PathBuf>,
    /// Pick which executables to compile. This allows users to create an app without CLI or GUI interface.
    #[clap(long, arg_enum, ignore_case = true, multiple_values = true)]
    executables: Option<Vec<Executable>>,
    /// Build with specific features selected
    #[clap(long)]
    features: Option<Vec<String>>,
}

impl BuilderOptions {
    pub fn target(&self) -> Target {
        self.target.as_ref().map_or_else(
            || <Target as FromStr>::from_str(&*version_meta().unwrap().host).unwrap(),
            |target| target.clone(),
        )
    }

    pub fn target_dir(&self) -> Option<&Path> {
        self.target_dir.as_ref().map(|dir| dir.as_path())
    }

    pub fn bundle_dir(&self) -> Option<&Path> {
        self.bundle_dir.as_ref().map(|dir| dir.as_path())
    }

    pub fn vmmaker_vm(&self) -> Option<&Path> {
        self.vmmaker_vm.as_ref().map(|dir| dir.as_path())
    }

    pub fn vmmaker_image(&self) -> Option<&Path> {
        self.vmmaker_image.as_ref().map(|dir| dir.as_path())
    }

    pub fn workspace_directory(&self) -> Option<PathBuf> {
        let error_message = "Failed to locate Cargo.toml of the VM project";

        which::which("cargo").expect(&format!(
            "{}: `cargo` must be installed and be in the PATH",
            error_message
        ));

        let mut command = Command::new("cargo");
        command
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format")
            .arg("plain");

        let output = command
            .output()
            .map_err(|error| {
                format!("{}: {:?} panicked due to {}", error_message, command, error);
            })
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            panic!(
                "{}: {:?} didn't finish successfully.\n\tExit code: {}\n\tStdout: {}\n\tStderr: {}",
                error_message,
                command,
                output.status.to_string(),
                &stdout,
                &stderr
            );
        }

        let workspace_toml_path = PathBuf::new().join(stdout);
        Some(workspace_toml_path.parent().unwrap().to_path_buf())
    }

    pub fn app_name(&self) -> Option<&str> {
        self.app_name.as_ref().map(|name| name.as_str())
    }

    pub fn identifier(&self) -> Option<&str> {
        self.identifier
            .as_ref()
            .map(|identifier| identifier.as_str())
    }

    pub fn executable_name(&self) -> Option<&str> {
        self.executable_name.as_ref().map(|name| name.as_str())
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_ref().map(|version| version.as_str())
    }

    pub fn verbose(&self) -> i32 {
        self.verbose
    }

    pub fn release(&self) -> bool {
        if self.release {
            true
        } else if self.debug {
            false
        } else {
            false // default
        }
    }
    
    pub fn include_debug_symbols(&self) -> bool {
        self.include_debug_symbols
    }

    pub fn icons(&self) -> Option<&Vec<String>> {
        self.icons.as_ref()
    }

    pub fn libraries(&self) -> Option<&Vec<ThirdPartyLibrary>> {
        self.libraries.as_ref()
    }

    pub fn libraries_versions(&self) -> VersionedThirdPartyLibraries {
        let mut versioned_libraries = match &self.libraries_versions {
            None => VersionedThirdPartyLibraries::new(),
            Some(versions_file) => serde_json::from_str(
                fs::read_to_string(versions_file)
                    .expect("Failed to read versions file")
                    .as_str(),
            )
            .expect("Failed to deserialized versions"),
        };

        if let Some(ref overridden_versions) = self.override_library_version {
            for (library, version) in overridden_versions {
                versioned_libraries.set_version_of(library.clone(), version);
            }
        }

        versioned_libraries
    }

    pub fn executables(&self) -> Option<&Vec<Executable>> {
        self.executables.as_ref()
    }

    pub fn features(&self) -> &[String] {
        self.features
            .as_ref()
            .map(|features| features.as_slice())
            .unwrap_or(&[])
    }
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: FromStr + Debug,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: FromStr + Debug,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;

    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
