use crate::{BundleOptions, Executable, Target};
use feenk_releaser::Version;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct ExecutableOptions<'bundle_options> {
    options: &'bundle_options BundleOptions,
    executable: Executable,
}

impl<'bundle_options> ExecutableOptions<'bundle_options> {
    pub fn new(options: &'bundle_options BundleOptions, executable: Executable) -> Self {
        Self {
            options,
            executable,
        }
    }

    pub fn app_name(&self) -> &str {
        self.options.app_name()
    }

    pub fn executable(&self) -> &Executable {
        &self.executable
    }

    pub fn executable_name(&self) -> String {
        self.options.bundled_executable_name(&self.executable)
    }

    pub fn version(&self) -> &Version {
        self.options.version()
    }

    pub fn verbose(&self) -> i32 {
        self.options.verbose()
    }

    pub fn vmmaker_vm(&self) -> Option<&Path> {
        self.options.vmmaker_vm()
    }

    pub fn vmmaker_image(&self) -> Option<&Path> {
        self.options.vmmaker_image()
    }

    pub fn target(&self) -> &Target {
        self.options.target()
    }

    pub fn target_dir(&self) -> &PathBuf {
        self.options.target_dir()
    }

    pub fn release(&self) -> bool {
        self.options.release()
    }

    pub fn identifier(&self) -> &str {
        self.options.identifier()
    }

    pub fn icons(&self) -> &Vec<PathBuf> {
        self.options.icons()
    }

    pub fn features(&self) -> &[String] {
        self.options.features()
    }

    pub fn cargo_build_command(&self) -> Command {
        self.executable().cargo_build_command()
    }

    pub fn cargo_package_name(&self) -> &str {
        self.executable().cargo_package_name()
    }

    pub fn compiled_executable_name(&self) -> String {
        self.options.compiled_executable_name(self.executable())
    }
}
