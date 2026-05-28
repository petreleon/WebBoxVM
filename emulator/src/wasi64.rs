//! Manual WASI fd_write binding for wasm64 — explicit u64 types for pointers.

use core::ffi::c_void;

#[link(wasm_import_module = "wasi_snapshot_preview1")]
unsafe extern "C" {
    // On wasm64, pointers are i64 in the WASM ABI
    fn fd_write(fd: u32, iovs: u64, iovs_len: u64, nwritten: u64) -> u32;
}

#[repr(C)]
struct Ciovec {
    ptr: *const u8,
    len: usize,
}

/// Write bytes to a WASI file descriptor (1=stdout, 2=stderr).
pub fn wasi_write(fd: u32, buf: &[u8]) -> usize {
    let iov = Ciovec { ptr: buf.as_ptr(), len: buf.len() };
    let mut nwritten: usize = 0;
    unsafe {
        fd_write(
            fd,
            &iov as *const _ as u64,
            1u64,
            &mut nwritten as *mut _ as u64,
        );
    }
    nwritten
}
