mod third_party;

pub use third_party::{ThirdPartyLibrary, VersionedThirdPartyLibraries};

use shared_library_builder::{GitLocation, LibraryLocation, PathLocation, RustLibrary};

pub fn boxer() -> RustLibrary {
    RustLibrary::new(
        "Boxer",
        LibraryLocation::Git(GitLocation::github("feenkcom", "gtoolkit-boxer")),
    )
}

pub fn clipboard() -> RustLibrary {
    RustLibrary::new(
        "Clipboard",
        LibraryLocation::Git(GitLocation::github("feenkcom", "libclipboard")),
    )
}

pub fn test_library() -> RustLibrary {
    RustLibrary::new(
        "TestLibrary",
        LibraryLocation::Path(PathLocation::new("vm-client-test-library")),
    )
}
