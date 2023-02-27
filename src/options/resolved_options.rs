use crate::{BuilderOptions, Executable, Platform, Target};
use chrono::Utc;
use feenk_releaser::{Version, VersionBump};
use serde::{Deserialize, Serialize};
use shared_library_builder::{Library, LibraryTarget};
use std::path::{Path, PathBuf};
use std::str::FromStr;

const DEFAULT_BUILD_DIR: &str = "target";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderInfo {
    build_timestamp: String,
    git_branch: String,
    git_commit_timestamp: String,
    git_sha: String,
    os_version: String,
}

impl BuilderInfo {
    pub fn new() -> Self {
        Self {
            build_timestamp: env!("VERGEN_BUILD_TIMESTAMP").to_string(),
            git_branch: env!("VERGEN_GIT_BRANCH").to_string(),
            git_commit_timestamp: env!("VERGEN_GIT_COMMIT_TIMESTAMP").to_string(),
            git_sha: env!("VERGEN_GIT_SHA").to_string(),
            os_version: env!("VERGEN_SYSINFO_OS_VERSION").to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    build_timestamp: String,
    git_branch: Option<String>,
    git_sha: Option<String>,
}

impl AppInfo {
    pub fn new() -> Self {
        let info = git_info::get();

        Self {
            build_timestamp: Utc::now().to_string(),
            git_branch: info.current_branch,
            git_sha: info.head.last_commit_hash,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolvedOptions {
    builder_flags: BuilderOptions,
    builder_info: BuilderInfo,
    app_build_info: AppInfo,
    #[serde(skip)]
    target_dir: PathBuf,
    target: Target,
    identifier: String,
    app_name: String,
    executable_name: String,
    version: Version,
    icons: Vec<PathBuf>,
    libraries: Vec<Box<dyn Library>>,
    executables: Vec<Executable>,
}

impl ResolvedOptions {
    pub fn new(options: BuilderOptions) -> Self {
        let target_dir: PathBuf = options.target_dir().map_or_else(
            || {
                options
                    .workspace_directory()
                    .map_or(PathBuf::from(DEFAULT_BUILD_DIR), |workspace| {
                        workspace.join(DEFAULT_BUILD_DIR)
                    })
            },
            |target_dir| target_dir.to_path_buf(),
        );

        let target = options.target();

        let app_name = options
            .app_name()
            .map_or("VM".to_owned(), |name| name.to_owned());

        let identifier = options
            .identifier()
            .map_or_else(|| app_name.clone(), |identifier| identifier.to_owned());

        let executable_name = options
            .executable_name()
            .map_or_else(|| app_name.clone(), |name| name.to_owned());

        let version = options.version().map_or_else(
            || Version::new(VersionBump::Patch),
            |version| {
                Version::parse(version).expect(&format!("Could not parse version {}", version))
            },
        );

        let icons = options.icons().map_or(vec![], |icons| {
            icons
                .iter()
                .map(|icon| PathBuf::from(icon))
                .collect::<Vec<PathBuf>>()
        });

        let library_target: LibraryTarget =
            LibraryTarget::from_str(target.to_string().as_str()).unwrap();
        let libraries_versions = options.libraries_versions();
        let libraries = options.libraries().map_or(vec![], |libraries| {
            libraries
                .iter()
                .map(|each| each.as_library(library_target, &libraries_versions))
                .collect::<Vec<Box<dyn Library>>>()
        });

        let executables = options
            .executables()
            .map_or(vec![Executable::Cli, Executable::App], |values| {
                values.clone()
            });

        Self {
            builder_flags: options,
            builder_info: BuilderInfo::new(),
            app_build_info: AppInfo::new(),
            target_dir,
            target,
            app_name,
            identifier,
            executable_name,
            version,
            icons,
            libraries,
            executables,
        }
    }

    pub fn target(&self) -> &Target {
        &self.target
    }

    pub fn platform(&self) -> Platform {
        self.target.platform()
    }

    pub fn target_dir(&self) -> &PathBuf {
        &self.target_dir
    }

    pub fn identifier(&self) -> &str {
        self.identifier.as_str()
    }

    pub fn app_name(&self) -> &str {
        self.app_name.as_str()
    }

    pub fn executable_name(&self) -> &str {
        self.executable_name.as_str()
    }

    pub fn executable_artefact_extension(&self) -> Option<String> {
        match self.target().platform() {
            Platform::Mac => None,
            Platform::Windows => Some("exe".to_string()),
            Platform::Linux => None,
            Platform::Android => Some("so".to_string()),
        }
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn verbose(&self) -> i32 {
        self.builder_flags.verbose()
    }

    pub fn release(&self) -> bool {
        self.builder_flags.release()
    }

    pub fn icons(&self) -> &Vec<PathBuf> {
        &self.icons
    }

    pub fn bundle_dir(&self) -> Option<&Path> {
        self.builder_flags.bundle_dir()
    }

    pub fn vmmaker_vm(&self) -> Option<&Path> {
        self.builder_flags.vmmaker_vm()
    }

    pub fn vmmaker_image(&self) -> Option<&Path> {
        self.builder_flags.vmmaker_image()
    }

    pub fn libraries(&self) -> &Vec<Box<dyn Library>> {
        &self.libraries
    }

    pub fn executables(&self) -> &Vec<Executable> {
        &self.executables
    }

    pub fn workspace_directory(&self) -> Option<PathBuf> {
        self.builder_flags.workspace_directory()
    }
}

impl Clone for ResolvedOptions {
    fn clone(&self) -> Self {
        Self {
            builder_flags: self.builder_flags.clone(),
            builder_info: self.builder_info.clone(),
            app_build_info: self.app_build_info.clone(),
            target_dir: self.target_dir.clone(),
            target: self.target.clone(),
            identifier: self.identifier.clone(),
            app_name: self.app_name.clone(),
            executable_name: self.executable_name.clone(),
            version: self.version.clone(),
            icons: self.icons.clone(),
            libraries: self
                .libraries
                .iter()
                .map(|library| library.clone_library())
                .collect(),
            executables: self.executables.clone(),
        }
    }
}
