use shared_library_builder::{GitLocation, LibraryLocation, RustLibrary};

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
