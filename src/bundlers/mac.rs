use crate::bundlers::Bundler;
use crate::options::BundleOptions;
use crate::{Executable, Result};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
use mach_object::{LoadCommand, OFile, LC_ID_DYLIB};
#[cfg(target_os = "macos")]
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MacBundler {}

impl MacBundler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_icns(&self, configuration: &BundleOptions) -> Option<PathBuf> {
        for icon in configuration.icons() {
            let icon_path = Path::new(&icon);
            if icon_path.exists() {
                if let Some(extension) = icon_path.extension() {
                    if extension == "icns" {
                        return Some(icon_path.to_path_buf());
                    }
                }
            }
        }
        None
    }

    #[cfg(not(target_os = "macos"))]
    fn set_rpath(_filename: impl AsRef<Path>) -> Result<()> {
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn set_rpath_to(_filename: impl AsRef<Path>, _path: impl AsRef<str>) -> Result<()> {
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub fn set_rpath_to(filename: impl AsRef<Path>, path: impl AsRef<str>) -> Result<()> {
        let file = File::open(filename.as_ref())?;
        let mmap = unsafe { memmap::Mmap::map(&file) }?;
        let payload = mmap.as_ref();
        let mut cur = std::io::Cursor::new(payload);
        let file = OFile::parse(&mut cur)?;

        println!("Processing {}...", filename.as_ref().display());

        match file {
            OFile::MachFile {
                header: _,
                ref commands,
            } => {
                if !Command::new("install_name_tool")
                    .arg("-add_rpath")
                    .arg(format!("@executable_path/{}", path.as_ref()))
                    .arg(&filename.as_ref())
                    .status()?
                    .success()
                {
                    panic!("Failed to add rpath to {}", filename.as_ref().display());
                }
                println!("   Added rpath to {}", filename.as_ref().display());

                let commands = commands
                    .iter()
                    .map(|load| load.command())
                    .cloned()
                    .collect::<Vec<LoadCommand>>();

                for command in commands {
                    match command {
                        LoadCommand::IdDyLib(ref dylib)
                        | LoadCommand::LoadDyLib(ref dylib)
                        | LoadCommand::LoadWeakDyLib(ref dylib)
                        | LoadCommand::ReexportDyLib(ref dylib)
                        | LoadCommand::LoadUpwardDylib(ref dylib)
                        | LoadCommand::LazyLoadDylib(ref dylib) => {
                            if !dylib.name.starts_with("/") {
                                let current_path = dylib.name.as_str();
                                let file_name = Path::new(current_path)
                                    .file_name()
                                    .unwrap()
                                    .to_str()
                                    .unwrap();
                                let new_path =
                                    format!("@executable_path/{}/{}", path.as_ref(), &file_name);

                                if command.cmd() == LC_ID_DYLIB {
                                    if !Command::new("install_name_tool")
                                        .arg("-id")
                                        .arg(&new_path)
                                        .arg(&filename.as_ref())
                                        .status()?
                                        .success()
                                    {
                                        panic!(
                                            "Failed to change id to {} of {}",
                                            &new_path,
                                            filename.as_ref().display()
                                        );
                                    };
                                    println!(
                                        "   Changed id of {} to {}",
                                        filename.as_ref().display(),
                                        &new_path
                                    );
                                } else {
                                    if !Command::new("install_name_tool")
                                        .arg("-change")
                                        .arg(&current_path)
                                        .arg(&new_path)
                                        .arg(&filename.as_ref())
                                        .status()?
                                        .success()
                                    {
                                        panic!(
                                            "Failed to change {} to {} in {}",
                                            current_path,
                                            &new_path,
                                            filename.as_ref().display()
                                        );
                                    };
                                    println!(
                                        "   Changed dependency of {} from {} to {}",
                                        filename.as_ref().display(),
                                        current_path,
                                        &new_path
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            OFile::FatFile { .. } => {}
            OFile::ArFile { .. } => {}
            OFile::SymDef { .. } => {}
        };

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn set_rpath(filename: impl AsRef<Path>) -> Result<()> {
        Self::set_rpath_to(filename, "Plugins")
    }

    fn debug_symbol_file(binary: &Path) -> PathBuf {
        let debug_symbols_folder_name = binary.file_name().and_then(|name|name.to_str()).map(|name| format!("{}.dSYM", name)).unwrap();
        binary.with_file_name(debug_symbols_folder_name)
    }
}

impl Bundler for MacBundler {
    fn bundle(&self, options: &BundleOptions) {
        let bundle_location = options.bundle_location();
        let app_name = options.app_name();

        let app_dir = bundle_location.join(format!("{}.app", &app_name));
        let contents_dir = app_dir.join("Contents");
        let resources_dir = contents_dir.join("Resources");
        let macos_dir = contents_dir.join("MacOS");
        let plugins_dir = macos_dir.join("Plugins");

        if app_dir.exists() {
            fs::remove_dir_all(&app_dir).unwrap();
        }
        fs::create_dir_all(&app_dir).unwrap();
        fs::create_dir(&contents_dir).unwrap();
        fs::create_dir(&resources_dir).unwrap();
        fs::create_dir(&macos_dir).unwrap();
        fs::create_dir(&plugins_dir).unwrap();

        options.executables().iter().for_each(|executable| {
            let compiled_executable_path = options.compiled_executable_path(executable);
            let bundled_executable_path = self
                .bundled_executable_directory(options)
                .join(options.bundled_executable_name(executable));
            match fs::copy(&compiled_executable_path, &bundled_executable_path) {
                Ok(_) => {
                    Self::set_rpath(&bundled_executable_path).expect(&format!(
                        "Failed to set the rpath of {}",
                        &bundled_executable_path.display()
                    ));
                }
                Err(error) => {
                    panic!(
                        "Could not copy {} to {} due to {}",
                        &compiled_executable_path.display(),
                        &bundled_executable_path.display(),
                        error
                    );
                }
            };
        });

        fs_extra::copy_items(
            &self.compiled_libraries(options),
            &plugins_dir,
            &fs_extra::dir::CopyOptions::new(),
        )
        .unwrap();
        
        if options.include_debug_symbols() {
            for each_library in self.compiled_libraries(options) {
                let debug_symbols = Self::debug_symbol_file(&each_library);
                if debug_symbols.exists() {
                    let options = fs_extra::dir::CopyOptions::default();
                    fs_extra::dir::copy(debug_symbols, &plugins_dir, &options).unwrap();
                }
            }
        }

        for library_path in self.compiled_libraries_in(&plugins_dir, options) {
            Self::set_rpath(&library_path).expect(&format!(
                "Failed to set the rpath of {}",
                &library_path.display()
            ));
        }

        let icon = if let Some(icon) = self.create_icns(options) {
            let resource_icon_name = resources_dir
                .join(options.app_name())
                .with_extension("icns");
            fs::copy(icon, resource_icon_name.clone()).unwrap();
            Some(resource_icon_name.clone())
        } else {
            None
        };

        let info_plist_template = mustache::compile_str(INFO_PLIST).unwrap();
        let info = Info {
            bundle_name: options.app_name().to_owned(),
            bundle_display_name: options.app_name().to_owned(),
            executable_name: options.bundled_executable_name(&Executable::App),
            bundle_identifier: options.identifier().to_owned(),
            bundle_version: options.version().to_string(),
            bundle_icon: icon.as_ref().map_or("".to_string(), |icon| {
                icon.file_name().unwrap().to_str().unwrap().to_string()
            }),
        };

        let mut file = File::create(contents_dir.join(Path::new("Info.plist"))).unwrap();
        info_plist_template.render(&mut file, &info).unwrap();
    }

    fn bundled_executable_directory(&self, options: &BundleOptions) -> PathBuf {
        options
            .bundle_location()
            .join(format!("{}.app", options.app_name()))
            .join("Contents")
            .join("MacOS")
    }

    fn bundled_resources_directory(&self, options: &BundleOptions) -> PathBuf {
        options
            .bundle_location()
            .join(format!("{}.app", options.app_name()))
            .join("Contents")
            .join("Resources")
    }

    fn clone_bundler(&self) -> Box<dyn Bundler> {
        Box::new(Clone::clone(self))
    }
}

#[derive(Serialize, Deserialize)]
struct Info {
    bundle_name: String,
    bundle_display_name: String,
    executable_name: String,
    bundle_identifier: String,
    bundle_version: String,
    bundle_icon: String,
}

const INFO_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>English</string>
  <key>CFBundleDisplayName</key>
  <string>{{bundle_display_name}}</string>
  <key>CFBundleExecutable</key>
  <string>{{executable_name}}</string>
  <key>CFBundleIdentifier</key>
  <string>{{bundle_identifier}}</string>
  <key>CFBundleIconFile</key>
  <string>{{bundle_icon}}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>{{bundle_name}}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>{{bundle_version}}</string>
  <key>CFBundleVersion</key>
  <string>{{bundle_version}}</string>
  <key>CSResourcesFileMapped</key>
  <true/>
  <key>LSRequiresCarbon</key>
  <true/>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>LSEnvironment</key>
	<dict>
	<key>WANTS_INTERACTIVE_SESSION</key>
	<string>true</string>
	</dict>
</dict>
</plist>
"#;
