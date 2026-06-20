#![warn(rust_2018_idioms)]

pub mod api;
pub mod arch;
#[deprecated(note = "use emulator::arch::arm64 instead")]
pub mod arm64;
pub mod boot;
#[deprecated(note = "use emulator::platform::virt::SystemBus instead")]
pub mod bus;
pub mod constants;
pub mod devices;
pub mod dtb;
pub mod efi;
pub mod host;
pub mod images;
pub mod initrd;
pub mod loader;
pub mod memory;
pub mod observability;
pub mod platform;
pub mod runtime;

#[cfg(target_arch = "wasm64")]
pub mod wasi64;

#[cfg(feature = "wasm")]
#[deprecated(note = "use emulator::host::wasm instead")]
pub mod wasm_main;
