use clap::{ArgEnum, Parser};

use crate::libraries::{ThirdPartyLibrary, VersionedThirdPartyLibraries};
use rustc_version::version_meta;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;

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
    #[clap(name = "x86_64-unknown-linux-gnu")]
    X8664UnknownlinuxGNU,
}

impl Target {
    pub fn for_current_platform() -> Self {
        <Target as FromStr>::from_str(&*version_meta().unwrap().host).unwrap()
    }

    pub fn is_unix(&self) -> bool {
        match self {
            Target::X8664appleDarwin => true,
            Target::AArch64appleDarwin => true,
            Target::X8664pcWindowsMsvc => false,
            Target::X8664UnknownlinuxGNU => true,
        }
    }

    pub fn is_windows(&self) -> bool {
        match self {
            Target::X8664appleDarwin => false,
            Target::AArch64appleDarwin => false,
            Target::X8664pcWindowsMsvc => true,
            Target::X8664UnknownlinuxGNU => false,
        }
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

#[derive(Parser, Clone, Debug, Default, Serialize, Deserialize)]
#[clap(version = "1.0", author = "feenk gmbh <contact@feenk.com>")]
pub struct BuilderOptions {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    /// To bundle a release build
    #[clap(long)]
    release: bool,
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
    /// An output location of the bundle. By default a bundle is placed inside of the cargo's target dir in the following format: target/{target architecture}/{build|release}/
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
    /// Use a specific VM to run a VMMaker, must be a path to the executable. When specified, the build will not attempt to download a VM
    #[clap(long, parse(from_os_str))]
    #[serde(skip)]
    vmmaker_vm: Option<PathBuf>,
    /// Use a specific image to build a VMMaker from, must be a path to the .image. When specified, the build will not attempt to download an image
    #[clap(long, parse(from_os_str))]
    #[serde(skip)]
    vmmaker_image: Option<PathBuf>,
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
        let output = Command::new("cargo")
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format")
            .arg("plain")
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to execute command");

        let workspace_toml_path =
            PathBuf::new().join(String::from_utf8_lossy(&output.stdout).to_string());
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
        self.release
    }

    pub fn icons(&self) -> Option<&Vec<String>> {
        self.icons.as_ref()
    }

    pub fn libraries(&self) -> Option<&Vec<ThirdPartyLibrary>> {
        self.libraries.as_ref()
    }

    pub fn libraries_versions(&self) -> VersionedThirdPartyLibraries {
        match &self.libraries_versions {
            None => VersionedThirdPartyLibraries::new(),
            Some(versions_file) => serde_json::from_str(
                fs::read_to_string(versions_file)
                    .expect("Failed to read versions file")
                    .as_str(),
            )
            .expect("Failed to deserialized versions"),
        }
    }
}
