#![warn(rust_2018_idioms)]

pub mod constants;
pub mod arm64;
pub mod boot;
pub mod bus;
pub mod devices;
pub mod dtb;
pub mod efi;
pub mod initrd;
pub mod loader;
pub mod memory;

#[cfg(target_arch = "wasm64")]
pub mod wasi64;

#[cfg(feature = "wasm")]
pub mod wasm_main;
