#![allow(non_snake_case)]

use std::error::Error;
use std::io::{self, ErrorKind, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};

use std::ffi::CString;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

// import the trait to be derived
use dlopen::wrapper::{Container, WrapperApi};
// import the derive macro
use dlopen_derive::WrapperApi;
use lazy_static::lazy_static;
use tempfile::NamedTempFile;

/// A wrapper for the SKE kernel functionality
///
/// Quite sparse ATM.
pub struct SkeServer {
    lib: Arc<Container<SkeLib<'static>>>,
    _named_file: Option<NamedTempFile>,
}

impl SkeServer {
    /// Create a new instance of `SkeServer` from vendored library
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut file = NamedTempFile::new()?;
        let written_bytes = file.write(SKELIB)?;
        assert_eq!(written_bytes, SKELIB.len());
        let mut new_instance = Self::from_file(file.path())?;
        new_instance._named_file = Some(file);
        Ok(new_instance)
    }

    /// Create a new instance of `SkeServer` from custom library
    pub fn from_file(libfile: &Path) -> Result<Self, Box<dyn Error>> {
        let lib = Arc::new(unsafe { Container::load(libfile) }?);
        Ok(Self {
            lib,
            _named_file: None,
        })
    }

    // Wrapped SKE functionality

    /// Run the kernel
    pub fn run(&self, duration_us: i64) {
        unsafe { self.lib.KRun(duration_us) }
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
        self.set_output();
        Ok(())
    }

    /// Get the number of partitions
    pub fn num_partitions(&self) -> usize {
        unsafe { self.lib.CfgGetNoPartitions() as usize }
    }

    /// Set the console callback for all partitions
    fn set_output(&self) {
        let mut lib = LIB_HANDLE.write().unwrap();
        *lib = Some(self.lib.clone());

        for i in 0..self.num_partitions() {
            unsafe {
                let p = self.lib.KPartitionGetByIdx(i as i32);
                self.lib
                    .KPartitionSetConsole(p, simple_console_cb as *const _);
            }
        }
    }
}

lazy_static! {
    static ref LIB_HANDLE: RwLock<Option<Arc<Container<SkeLib<'static>>>>> = RwLock::new(None);
}

/// This function is our console callback. It's called by SKE everytime a partition want's to print
extern "C" fn simple_console_cb(partition: *const Partition, ptr: *const i8, len: u32) {
    let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    let message = std::str::from_utf8(slice)
        .expect("unable to parse output from partition as utf8")
        .trim_end_matches('\n');

    let lib_guard = LIB_HANDLE.read();
    let maybe_lib = lib_guard.expect("the lock is poisoned");
    let lib = maybe_lib.as_ref().expect("LIB_HANDLE not propagated");

    let p_cfg = unsafe { lib.KPartitionGetCfg(partition) };
    let p_name = std::str::from_utf8(unsafe {
        std::slice::from_raw_parts((*p_cfg).name.as_ptr(), MAX_STRING_LENGTH)
    })
    .expect("unable to parse partition name as utf8")
    .trim_end_matches('\0');

    let prefix = format!("{}: ", p_name);

    let mut filler = String::from("\n");
    filler.push_str(&" ".repeat(prefix.len()));
    println!("{}{}", prefix, message.replace('\n', &filler));
}

#[derive(WrapperApi)]
struct SkeLib<'a> {
    // functions
    KLoadCfg: unsafe extern "C" fn(cfg: *const i8) -> bool,
    KRun: unsafe extern "C" fn(duration: i64),

    CfgGetNoPartitions: unsafe extern "C" fn() -> i32,
    CfgGetPartitionByIdx: unsafe extern "C" fn(index: i32) -> *const PartitionConfig,

    CfgGetScheduleByIdx: unsafe extern "C" fn(index: i32) -> *const ScheduleConfig,

    KPartitionGetCfg: unsafe extern "C" fn(partition: *const Partition) -> *const PartitionConfig,

    KPartitionGetByIdx: unsafe extern "C" fn(index: i32) -> *const Partition,
    KPartitionSetConsole: unsafe extern "C" fn(
        partition: *const Partition,
        console_callback: *const ConsoleCallback,
    ) -> *const Partition,

    // variables
    skeMaxStringLength: &'a i32,
    skeMaxNoSchedPWnds: &'a i32,
    skeMaxNoPorts: &'a i32,
    skeKTraceEventSize: &'a i32,
    kClockScaleFactor: &'a i32,
}

// Types defined in C
type ConsoleCallback = extern "C" fn(*const Partition, *const i8, u32);

/*
#[repr(C)]
struct PartitionSchedule {
    period: i64,
    duration: i64,
}
*/

#[repr(C)]
struct ScheduleConfig {
    period: i64,
    duration: i64,
}

#[repr(C)]
struct Port {
    name: [i8; MAX_STRING_LENGTH],
    channelIdx: i32,
    r#type: i32,
    direction: i32,
}

#[repr(C)]
struct PartitionConfig {
    name: [u8; MAX_STRING_LENGTH],
    flags: i32,
    schedule: Schedule,
    ports: [Port; MAX_NO_PORTS],
    potrs_len: i32,
}

#[repr(C)]
struct Schedule {
    period: i64,
    duration: i64,
}

#[repr(C)]
struct Partition {
    _private: [u8; 0],
}

const MAX_STRING_LENGTH: usize = 32; // TODO check this value
const MAX_NO_PORTS: usize = 32; // TODO check this value

/// This constant stores vendored `libskeserver.so` so that ske-rs can work standalone
const SKELIB: &[u8] = include_bytes!("../libskeserver.so");
