mod bzip2_library;
mod cairo_library;
mod freetype_library;
mod openssl_library;
mod pixman_library;
mod png_library;
mod zlib_library;

pub use bzip2_library::BZip2Library;
pub use cairo_library::CairoLibrary;
pub use freetype_library::FreetypeLibrary;
pub use openssl_library::OpenSSLLibrary;
pub use pixman_library::PixmanLibrary;
pub use png_library::PngLibrary;
use shared_library_builder::{
    CMakeLibrary, CompiledLibraryName, GitLocation, LibraryLocation, RustLibrary,
};
pub use zlib_library::ZLibLibrary;

pub fn crypto() -> OpenSSLLibrary {
    OpenSSLLibrary::new().be_crypto()
}

pub fn ssl() -> OpenSSLLibrary {
    OpenSSLLibrary::new().be_ssl()
}

pub fn git() -> CMakeLibrary {
    let openssl = OpenSSLLibrary::new();

    let libssh2 = CMakeLibrary::new(
        "ssh2",
        LibraryLocation::Git(GitLocation::github("libssh2", "libssh2").tag("libssh2-1.9.0")),
    )
    .define_common("CRYPTO_BACKEND", "OpenSSL")
    .depends(Box::new(openssl));

    CMakeLibrary::new(
        "git2",
        LibraryLocation::Git(
            GitLocation::github("syrel", "libgit2").branch("v1.1.1-windows-openssl"),
        ),
    )
    .compiled_name(CompiledLibraryName::Matching("git2".to_string()))
    .define_common("BUILD_CLAR", "OFF")
    .define_common("REGEX_BACKEND", "builtin")
    .define_common("USE_BUNDLED_ZLIB", "ON")
    .depends(Box::new(libssh2))
}

pub fn boxer() -> RustLibrary {
    RustLibrary::new(
        "Boxer",
        LibraryLocation::Git(GitLocation::github("feenkcom", "gtoolkit-boxer")),
    )
}

pub fn winit() -> RustLibrary {
    RustLibrary::new(
        "Winit",
        LibraryLocation::Git(GitLocation::github("feenkcom", "libwinit")),
    )
}

pub fn clipboard() -> RustLibrary {
    RustLibrary::new(
        "Clipboard",
        LibraryLocation::Git(GitLocation::github("feenkcom", "libclipboard")),
    )
}
