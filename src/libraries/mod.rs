mod third_party;

pub use third_party::{ThirdPartyLibrary, VersionedThirdPartyLibraries};

use shared_library_builder::{LibraryLocation, PathLocation, RustLibrary};

pub fn test_library() -> RustLibrary {
    RustLibrary::new(
        "TestLibrary",
        LibraryLocation::Path(PathLocation::new("vm-client-test-library")),
    )
}
