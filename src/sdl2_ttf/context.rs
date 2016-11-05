use std::io;
use std::error;
use std::fmt;
use std::os::raw::{c_int, c_long};
use std::path::Path;
use sdl2::get_error;
use sdl2::rwops::RWops;
use sdl2::version::Version;

use font::{
    internal_load_font,
    internal_load_font_at_index,
    internal_load_font_from_ll,
    Font,
};

use ffi;

/// A context manager for `SDL2_TTF` to manage C code initialization and clean-up.
#[must_use]
pub struct Sdl2TtfContext;

// Clean up the context once it goes out of scope
impl Drop for Sdl2TtfContext {
    fn drop(&mut self) {
        unsafe { ffi::TTF_Quit(); }
    }
}

impl Sdl2TtfContext {
    /// Loads a font from the given file with the given size in points.
    pub fn load_font<'a>(&'a self, path: &'a Path, point_size: u16) -> Result<Font, String> {
        internal_load_font(path, point_size)
    }

    /// Loads the font at the given index of the file, with the given
    /// size in points.
    pub fn load_font_at_index<'a>(&'a self, path: &'a Path, index: u32, point_size: u16)
            -> Result<Font, String> {
        internal_load_font_at_index(path, index, point_size)
    }

    /// Loads a font from the given SDL2 rwops object with the given size in
    /// points.
    pub fn load_font_from_rwops<'a,'b>(&'a self, rwops: RWops<'b>, point_size: u16)
            -> Result<Font<'b>, String> {
        let raw = unsafe {
            ffi::TTF_OpenFontRW(rwops.raw(), 0, point_size as c_int)
        };
        if (raw as *mut ()).is_null() {
            Err(get_error())
        } else {
            Ok(internal_load_font_from_ll(raw, Some(rwops)))
        }
    }

    /// Loads the font at the given index of the SDL2 rwops object with
    /// the given size in points.
    pub fn load_font_at_index_from_rwops<'a,'b>(&'a self, rwops: RWops<'b>, index: u32,
            point_size: u16) -> Result<Font<'b>, String> {
        let raw = unsafe {
            ffi::TTF_OpenFontIndexRW(rwops.raw(), 0, point_size as c_int,
                index as c_long)
        };
        if (raw as *mut ()).is_null() {
            Err(get_error())
        } else {
            Ok(internal_load_font_from_ll(raw, Some(rwops)))
        }
    }
}

/// Returns the version of the dynamically linked `SDL_TTF` library
pub fn get_linked_version() -> Version {
    unsafe {
        Version::from_ll(*ffi::TTF_Linked_Version())
    }
}

/// An error for when `sdl2_ttf` is attempted initialized twice
/// Necessary for context management, unless we find a way to have a singleton
#[derive(Debug)]
pub enum InitError {
    InitializationError(io::Error),
    AlreadyInitializedError,
}

impl error::Error for InitError {
    fn description(&self) -> &str {
        match *self {
            InitError::AlreadyInitializedError => {
                "SDL2_TTF has already been initialized"
            },
            InitError::InitializationError(ref error) => {
                error.description()
            },
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            InitError::AlreadyInitializedError => {
                None
            },
            InitError::InitializationError(ref error) => {
                Some(error)
            },
        }
    }
}

impl fmt::Display for InitError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        formatter.write_str("SDL2_TTF has already been initialized")
    }
}

/// Initializes the truetype font API and returns a context manager which will
/// clean up the library once it goes out of scope.
pub fn init() -> Result<Sdl2TtfContext, InitError> {
    unsafe {
        if ffi::TTF_WasInit() == 1 {
            Err(InitError::AlreadyInitializedError)
        } else if ffi::TTF_Init() == 0 {
            Ok(Sdl2TtfContext)
        } else {
            Err(InitError::InitializationError(
                io::Error::last_os_error()
                    ))
        }
    }
}

/// Returns whether library has been initialized already.
pub fn has_been_initialized() -> bool {
    unsafe {
        ffi::TTF_WasInit() == 1
    }
}
