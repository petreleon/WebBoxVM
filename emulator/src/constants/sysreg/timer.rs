pub const TIMER_FREQ_HZ: u64 = 62_500_000;
pub const PHYSICAL_TIMER_IRQ_ID: u32 = 30;
pub const VIRTUAL_TIMER_IRQ_ID: u32 = 27;
pub const PL011_UART_IRQ_ID: u32 = 33;
pub const VIRTIO_BLK_IRQ_ID: u32 = 48;
pub const VIRTIO_DISK_IRQ_ID: u32 = 49;
pub const GIC_SPURIOUS_INTERRUPT: u64 = 1023;

pub const TIMER_CTL_ENABLE: u64 = 1;
pub const TIMER_CTL_IMASK: u64 = 1 << 1;
pub const TIMER_CTL_ISTATUS: u64 = 1 << 2;
