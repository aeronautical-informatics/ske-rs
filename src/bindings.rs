#![allow(non_snake_case)]

use std::error::Error;
use std::io::{self, ErrorKind, Write};
use std::path::Path;

use std::ffi::CString;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

// import the trait to be derived
use dlopen::wrapper::{Container, WrapperApi};
// import the derive macro
use dlopen_derive::WrapperApi;

use tempfile::NamedTempFile;

/// A wrapper for the SKE kernel functionality
///
/// Quite sparse ATM.
pub struct SkeServer {
    lib: Container<SkeLib>,
    _named_file: Option<NamedTempFile>,
}

impl SkeServer {
    /// Create a new instance of `SkeServer` from vendored library
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut file = NamedTempFile::new()?;
        file.write(SKELIB)?;
        let mut new_instance = Self::from_file(file.path())?;
        new_instance._named_file = Some(file);
        Ok(new_instance)
    }

    /// Create a new instance of `SkeServer` from custom library
    pub fn from_file(libfile: &Path) -> Result<Self, Box<dyn Error>> {
        let lib = unsafe { Container::load(libfile) }?;
        Ok(Self {
            lib,
            _named_file: None,
        })
    }

    // Wrapped implementation details

    /// Run the kernel
    pub fn run(&self) {
        unsafe { self.lib.KRun() }
    }

    /// Load a SKE configuration
    pub fn config(&self, conf_file: &Path) -> Result<(), Box<dyn Error>> {
        // Check whether the config file actually exists
        if !conf_file.is_file() {
            return Err(io::Error::new(
                ErrorKind::NotFound,
                format!("{:?} is not a file", conf_file),
            )
            .into());
        }

        // creates a NULL terminated C String
        let config_file_path = CString::new(AsRef::<OsStr>::as_ref(conf_file).as_bytes())?;
        unsafe { self.lib.KLoadCfg(config_file_path.as_ptr()) };
        Ok(())
    }
}

#[derive(WrapperApi)]
struct SkeLib {
    KRun: unsafe extern "C" fn(),
    KLoadCfg: unsafe extern "C" fn(cfg: *const i8),
}

/// This constant stores vendored `libskeserver.so` so that ske-rs can work standalone
const SKELIB: &[u8] = include_bytes!("../libskeserver.so");
