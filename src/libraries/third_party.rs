use std::collections::HashMap;
use std::str::FromStr;

use clap::ArgEnum;
use libboxer_library::libboxer;
use libcairo_library::libcairo;
use libclipboard_library::libclipboard;
use libfreetype_library::libfreetype;
use libgit2_library::libgit2;
use libgleam_library::libgleam;
use libglutin_library::libglutin;
use libopenssl_library::{libcrypto, libssl};
use libphlow_builder::libphlow;
use libpixels_library::libpixels;
use libprocess_library::libprocess;
use libsdl2_library::libsdl2;
use libskia_library::libskia;
use libwinit_library::libwinit;
use serde::{Deserialize, Serialize};
use shared_library_builder::{Library, LibraryTarget};

use crate::libraries::test_library;

#[derive(ArgEnum, Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ThirdPartyLibrary {
    #[clap(name = "git")]
    Git,
    #[clap(name = "crypto")]
    Crypto,
    #[clap(name = "ssl")]
    Ssl,
    #[clap(name = "sdl2")]
    Sdl2,
    #[clap(name = "boxer")]
    Boxer,
    #[clap(name = "freetype")]
    Freetype,
    #[clap(name = "cairo")]
    Cairo,
    #[clap(name = "skia")]
    Skia,
    #[clap(name = "glutin")]
    Glutin,
    #[clap(name = "gleam")]
    Gleam,
    #[clap(name = "winit")]
    Winit,
    #[clap(name = "pixels")]
    Pixels,
    #[clap(name = "clipboard")]
    Clipboard,
    #[clap(name = "process")]
    Process,
    #[clap(name = "phlow")]
    Phlow,
    #[clap(name = "test-library")]
    TestLibrary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionedThirdPartyLibraries {
    #[serde(flatten)]
    libraries: HashMap<ThirdPartyLibrary, String>,
}

impl VersionedThirdPartyLibraries {
    pub fn new() -> Self {
        Self {
            libraries: HashMap::new(),
        }
    }

    pub fn get_version_of(&self, library: ThirdPartyLibrary) -> Option<&str> {
        self.libraries.get(&library).map(|version| version.as_str())
    }

    pub fn version_of(&self, library: ThirdPartyLibrary) -> &str {
        self.get_version_of(library)
            .expect("Could not find a library version")
    }

    pub fn set_version_of(&mut self, library: ThirdPartyLibrary, version: impl Into<String>) {
        self.libraries.insert(library, version.into());
    }
}

impl FromStr for ThirdPartyLibrary {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <ThirdPartyLibrary as ArgEnum>::from_str(s, true).map_err(|error| Self::Err::new(error))
    }
}

impl ToString for ThirdPartyLibrary {
    fn to_string(&self) -> String {
        self.to_possible_value().unwrap().get_name().to_string()
    }
}

impl ThirdPartyLibrary {
    pub fn as_library(
        &self,
        target: LibraryTarget,
        versions: &VersionedThirdPartyLibraries,
    ) -> Box<dyn Library> {
        match self {
            ThirdPartyLibrary::Boxer => {
                libboxer(versions.get_version_of(ThirdPartyLibrary::Boxer)).into()
            }
            ThirdPartyLibrary::Cairo => {
                libcairo(versions.get_version_of(ThirdPartyLibrary::Cairo)).into()
            }
            ThirdPartyLibrary::Clipboard => {
                libclipboard(Some(versions.version_of(ThirdPartyLibrary::Clipboard))).into()
            }
            ThirdPartyLibrary::Crypto => {
                libcrypto(versions.get_version_of(ThirdPartyLibrary::Crypto)).into()
            }
            ThirdPartyLibrary::Freetype => {
                libfreetype(versions.get_version_of(ThirdPartyLibrary::Freetype)).into()
            }
            ThirdPartyLibrary::Git => {
                libgit2(versions.get_version_of(ThirdPartyLibrary::Git)).into()
            }
            ThirdPartyLibrary::Gleam => {
                libgleam(versions.get_version_of(ThirdPartyLibrary::Gleam)).into()
            }
            ThirdPartyLibrary::Glutin => {
                libglutin(versions.get_version_of(ThirdPartyLibrary::Glutin)).into()
            }
            ThirdPartyLibrary::Process => {
                libprocess(Some(versions.version_of(ThirdPartyLibrary::Process))).into()
            }
            ThirdPartyLibrary::Sdl2 => {
                libsdl2(versions.get_version_of(ThirdPartyLibrary::Sdl2)).into()
            }
            ThirdPartyLibrary::Skia => {
                libskia(target, Some(versions.version_of(ThirdPartyLibrary::Skia))).into()
            }
            ThirdPartyLibrary::Ssl => {
                libssl(versions.get_version_of(ThirdPartyLibrary::Ssl)).into()
            }
            ThirdPartyLibrary::Winit => {
                libwinit(Some(versions.version_of(ThirdPartyLibrary::Winit))).into()
            }
            ThirdPartyLibrary::Pixels => {
                libpixels(Some(versions.version_of(ThirdPartyLibrary::Pixels))).into()
            }
            ThirdPartyLibrary::Phlow => {
                libphlow(Some(versions.version_of(ThirdPartyLibrary::Phlow))).into()
            }
            ThirdPartyLibrary::TestLibrary => test_library().into(),
        }
    }
}
