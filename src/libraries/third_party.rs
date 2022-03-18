use clap::ArgEnum;
use serde::{Deserialize, Serialize};
use shared_library_builder::Library;
use std::collections::HashMap;
use std::str::FromStr;

use crate::libraries::{boxer, clipboard, test_library};
use libcairo_library::libcairo;
use libfreetype_library::libfreetype;
use libgit2_library::libgit2;
use libgleam_library::libgleam;
use libglutin_library::libglutin;
use libopenssl_library::{libcrypto, libssl};
use libpixels_library::libpixels;
use libprocess_library::libprocess;
use libsdl2_library::libsdl2;
use libskia_library::libskia;
use libwinit_library::libwinit;

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
}

impl FromStr for ThirdPartyLibrary {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <ThirdPartyLibrary as ArgEnum>::from_str(s, true)
    }
}

impl ToString for ThirdPartyLibrary {
    fn to_string(&self) -> String {
        self.to_possible_value().unwrap().get_name().to_string()
    }
}

impl ThirdPartyLibrary {
    pub fn as_library(&self, versions: &VersionedThirdPartyLibraries) -> Box<dyn Library> {
        match self {
            ThirdPartyLibrary::Boxer => boxer().into(),
            ThirdPartyLibrary::Cairo => {
                libcairo(versions.get_version_of(ThirdPartyLibrary::Cairo)).into()
            }
            ThirdPartyLibrary::Clipboard => clipboard().into(),
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
                libgleam(versions.version_of(ThirdPartyLibrary::Gleam)).into()
            }
            ThirdPartyLibrary::Glutin => {
                libglutin(versions.version_of(ThirdPartyLibrary::Glutin)).into()
            }
            ThirdPartyLibrary::Process => {
                libprocess(versions.version_of(ThirdPartyLibrary::Process)).into()
            }
            ThirdPartyLibrary::Sdl2 => {
                libsdl2(versions.get_version_of(ThirdPartyLibrary::Sdl2)).into()
            }
            ThirdPartyLibrary::Skia => libskia(versions.version_of(ThirdPartyLibrary::Skia)).into(),
            ThirdPartyLibrary::Ssl => {
                libssl(versions.get_version_of(ThirdPartyLibrary::Ssl)).into()
            }
            ThirdPartyLibrary::Winit => {
                libwinit(versions.version_of(ThirdPartyLibrary::Winit)).into()
            }
            ThirdPartyLibrary::Pixels => {
                libpixels(versions.version_of(ThirdPartyLibrary::Pixels)).into()
            }
            ThirdPartyLibrary::TestLibrary => test_library().into(),
        }
    }
}
