use error::*;
use kernel32;
use os::windows::OkOrGetLastError;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use util;
use winapi::HMODULE;
use winapi::LPCSTR;

#[derive(Debug)]
pub struct Lib {
    handle: HMODULE
}

impl Lib {
    pub unsafe fn new<TPath>(path_to_lib: TPath) -> Result<Lib>
        where TPath: AsRef<Path> {
        let path_to_lib_vec: Vec<_> =
            path_to_lib
                .as_ref()
                .as_os_str()
                .encode_wide()
                .chain((0..1))
                .collect();
        let path_to_lib_ptr = path_to_lib_vec.as_ptr();

        util::error_guard(
            || {
                let handle = kernel32::LoadLibraryW(path_to_lib_ptr);
                let lib_option =
                    if handle.is_null()  {
                        None
                    } else {
                        let lib = Lib { handle: handle };
                        Some(lib)
                    };
                lib_option.ok_or_get_last_error("LoadLibraryW")
            }
        ).chain_err(
            || ErrorKind::LibraryOpen(path_to_lib.as_ref().to_path_buf())
        )
    }

    pub unsafe fn find<T, TStr>(&self, symbol_str: TStr) -> Result<*const T>
        where TStr: AsRef<str> {
        let symbol = symbol_str.as_ref();
        let symbol = symbol.as_ptr();
        let symbol = symbol as LPCSTR;

        util::error_guard(
            || {
                let symbol = kernel32::GetProcAddress(self.handle, symbol);
                if symbol.is_null() {
                    None
                } else {
                    Some(mem::transmute(symbol))
                }.ok_or_get_last_error("GetProcAddress")
            }
        ).chain_err(
            || ErrorKind::LibraryFindSymbol(symbol_str.as_ref().to_string())
        )
    }
}

unsafe impl Send for Lib { }

unsafe impl Sync for Lib { }

impl Drop for Lib {
    fn drop(&mut self) {
        util::error_guard(
            || {
                if unsafe { kernel32::FreeLibrary(self.handle) } == 0 {
                    None
                } else {
                    Some(())
                }.ok_or_get_last_error("FreeLibrary")
            }
        ).chain_err(
            || ErrorKind::LibraryClose
        ).unwrap()
    }
}
