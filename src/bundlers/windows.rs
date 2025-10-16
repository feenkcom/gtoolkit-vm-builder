use crate::bundlers::Bundler;
use crate::options::BundleOptions;
use crate::{Executable, ExecutableOptions};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use user_error::UserFacingError;

#[derive(Debug, Clone)]
pub struct WindowsBundler {}

const STACK_SIZE: usize = 16000000;

impl WindowsBundler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_ico(&self, options: &ExecutableOptions) -> Option<PathBuf> {
        for icon in options.icons() {
            let icon_path = Path::new(&icon);
            if icon_path.exists() {
                if let Some(extension) = icon_path.extension() {
                    if extension == "ico" {
                        let icon_path =
                            fs::canonicalize(icon_path).expect("Icon could not be located");
                        return Some(icon_path);
                    }
                }
            }
        }
        None
    }

    fn set_stack_size(
        &self,
        bundle_options: &BundleOptions,
        binary: impl AsRef<Path>,
        size_in_bytes: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut editbin =
            cc::windows_registry::find(bundle_options.target().to_string().as_str(), "editbin.exe")
                .expect("Could not find editbin.exe");

        let binary = binary.as_ref();
        if !editbin
            .arg(format!("/STACK:{}", size_in_bytes))
            .arg(binary)
            .status()?
            .success()
        {
            return Err(Box::new(UserFacingError::new(format!(
                "Failed to set /STACK of {}",
                binary.display(),
            ))));
        }
        Ok(())
    }

    fn temporary_directory(&self) -> PathBuf {
        std::env::current_dir().unwrap().join("temp")
    }

    fn debug_symbol_file(binary: &Path) -> PathBuf {
        // PDB filenames are derived from the executable name,
        // but MSVC/LLVM does not allow certain characters like "-"
        // in filenames for PDBs, so it replaces hyphens - with underscores _.
        // So my_app-cli.exe becomes my_app_cli.pdb.
        let name = binary
            .file_name()
            .and_then(|s| s.to_str())
            .map(|name| name.replace("-", "_"));
        let binary = match name {
            Some(name) => binary.with_file_name(name),
            None => binary.to_owned(),
        };
        binary.with_extension("pdb")
    }
}

impl Bundler for WindowsBundler {
    fn pre_compile(&self, options: &ExecutableOptions) {
        let temp_dir = self.temporary_directory();

        let icon = self.create_ico(options);

        let info = Info {
            bundle_name: options.app_name().to_owned(),
            bundle_identifier: options.identifier().to_owned(),
            bundle_author: "".to_string(),
            bundle_major_version: options.version().major(),
            bundle_minor_version: options.version().minor(),
            bundle_patch_version: options.version().patch(),
            bundle_icon: icon.as_ref().map_or("".to_string(), |icon| {
                format!("100 ICON {:?}", icon.display())
            }),
            executable_name: options.executable_name(),
        };

        let resource = mustache::compile_str(RESOURCE).unwrap();
        let manifest = mustache::compile_str(MANIFEST).unwrap();

        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir).unwrap();
        }

        let resource_file_path = temp_dir.join(format!("{}.rc", options.executable_name()));

        let manifest_file_path = temp_dir.join(format!("{}.manifest", options.executable_name()));

        let mut resource_file = File::create(&resource_file_path).unwrap();
        let mut manifest_file = File::create(&manifest_file_path).unwrap();

        resource.render(&mut resource_file, &info).unwrap();
        manifest.render(&mut manifest_file, &info).unwrap();

        std::env::set_var(
            "VM_CLIENT_EMBED_RESOURCES",
            format!("{}", &resource_file_path.display()),
        );
    }

    fn post_compile(
        &self,
        bundle_options: &BundleOptions,
        executable: &Executable,
        _executable_options: &ExecutableOptions,
    ) {
        let temp_dir = self.temporary_directory();
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }

        self.set_stack_size(
            bundle_options,
            bundle_options.compiled_executable_path(executable),
            STACK_SIZE,
        )
        .expect("Failed to set /STACK size");
    }

    fn bundle(&self, options: &BundleOptions) {
        let bundle_location = options.bundle_location();
        let app_name = options.app_name();

        let app_dir = bundle_location.join(&app_name);
        let binary_dir = self.bundled_executable_directory(options);

        if app_dir.exists() {
            fs::remove_dir_all(&app_dir).unwrap();
        }
        fs::create_dir_all(&app_dir).unwrap();
        fs::create_dir(&binary_dir).unwrap();

        options.executables().iter().for_each(|executable| {
            let compiled_executable_path = options.compiled_executable_path(executable);
            let bundled_executable_path =
                binary_dir.join(options.bundled_executable_name(executable));
            match fs::copy(&compiled_executable_path, &bundled_executable_path) {
                Ok(_) => {}
                Err(error) => {
                    panic!(
                        "Could not copy {} to {} due to {}",
                        &compiled_executable_path.display(),
                        &bundled_executable_path.display(),
                        error
                    );
                }
            };

            if !options.release() {
                let compiled_symbols_path = Self::debug_symbol_file(&compiled_executable_path);
                let bundled_symbols_path = Self::debug_symbol_file(&bundled_executable_path);

                match fs::copy(&compiled_symbols_path, &bundled_symbols_path) {
                    Ok(_) => {}
                    Err(error) => {
                        panic!(
                            "Could not copy {} to {} due to {}",
                            &compiled_symbols_path.display(),
                            &bundled_symbols_path.display(),
                            error
                        );
                    }
                };
            }
        });

        fs_extra::copy_items(
            &self.compiled_libraries(options),
            &binary_dir,
            &fs_extra::dir::CopyOptions::new(),
        )
        .unwrap();

        if !options.release() {
            let compiled_libraries_debug_symbols: Vec<PathBuf> = self
                .compiled_libraries(options)
                .iter()
                .map(|each| Self::debug_symbol_file(each))
                .filter(|each| each.exists())
                .collect();
            fs_extra::copy_items(
                &compiled_libraries_debug_symbols,
                &binary_dir,
                &fs_extra::dir::CopyOptions::new(),
            )
            .unwrap();
        }
    }

    fn bundled_executable_directory(&self, options: &BundleOptions) -> PathBuf {
        options
            .bundle_location()
            .join(options.app_name())
            .join("bin")
    }

    fn bundled_resources_directory(&self, options: &BundleOptions) -> PathBuf {
        options
            .bundle_location()
            .join(options.app_name())
            .join("share")
    }

    fn clone_bundler(&self) -> Box<dyn Bundler> {
        Box::new(Clone::clone(self))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Info {
    bundle_name: String,
    bundle_identifier: String,
    bundle_author: String,
    bundle_major_version: u64,
    bundle_minor_version: u64,
    bundle_patch_version: u64,
    bundle_icon: String,
    executable_name: String,
}

const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly manifestVersion="1.0" xmlns="urn:schemas-microsoft-com:asm.v1" xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
    <assemblyIdentity
            version="1.0.0.0"
            processorArchitecture="*"
            name="{{bundle_identifier}}"
            type="win32"
    />
    <description>Rust Manifest Example</description>
    <dependency>
        <dependentAssembly>
            <assemblyIdentity
                    type="win32"
                    name="Microsoft.Windows.Common-Controls"
                    version="6.0.0.0"
                    processorArchitecture="*"
                    publicKeyToken="6595b64144ccf1df"
                    language="*"
            />
        </dependentAssembly>
    </dependency>
    <asmv3:application>
        <asmv3:windowsSettings>
            <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">True/PM</dpiAware>
            <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
        </asmv3:windowsSettings>
    </asmv3:application>
</assembly>
"#;

const RESOURCE: &str = r#"#include "windows.h"

1 RT_MANIFEST "{{executable_name}}.manifest"
{{{bundle_icon}}}

VS_VERSION_INFO VERSIONINFO
FILEVERSION     {{bundle_major_version}},{{bundle_minor_version}},{{bundle_patch_version}},0
PRODUCTVERSION  {{bundle_major_version}},{{bundle_minor_version}},{{bundle_patch_version}},0
FILEFLAGSMASK   VS_FFI_FILEFLAGSMASK
FILEFLAGS       VS_FF_DEBUG
FILEOS          VOS__WINDOWS32
FILETYPE        VFT_APP
FILESUBTYPE     VFT2_UNKNOWN
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904E4"    // Lang=US English, CharSet=Windows Multilin
        BEGIN
            VALUE "CompanyName", "{{bundle_author}}\0"
            VALUE "FileDescription", "{{bundle_name}}\0"
            VALUE "FileVersion", "{{bundle_major_version}}.{{bundle_minor_version}}.{{bundle_patch_version}}\0"
            VALUE "ProductName", "{{bundle_name}}\0"
            VALUE "ProductVersion", "{{bundle_major_version}}.{{bundle_minor_version}}.{{bundle_patch_version}}\0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1252
    END
END
"#;
