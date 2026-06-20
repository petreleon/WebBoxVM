use crate::arch::arm64::Armv8Cpu;
use crate::boot::{BootPlan, build_busybox_initrd, build_default_initrd};
use crate::constants::{PL011_UART_IRQ_ID, SCTLR_MMU_ENABLE};
use crate::dtb::load_dtb;
use crate::initrd::load_initrd;
use crate::runtime::Machine;

/// Live runtime context created by applying a pure boot plan to a machine.
pub struct BootContext {
    pub machine: Machine,
    pub dtb_addr: u64,
}

impl BootContext {
    pub fn new(kernel_image: &[u8], num_cores: usize) -> Result<Self, String> {
        Self::from_plan(BootPlan::new_with_initrd(
            kernel_image,
            num_cores,
            &build_default_initrd(),
        )?)
    }

    pub fn new_with_busybox(
        kernel_image: &[u8],
        num_cores: usize,
        busybox: &[u8],
    ) -> Result<Self, String> {
        let initrd = build_busybox_initrd(busybox)?;
        Self::new_with_initrd(kernel_image, num_cores, &initrd)
    }

    pub fn new_with_initrd(
        kernel_image: &[u8],
        num_cores: usize,
        initrd: &[u8],
    ) -> Result<Self, String> {
        Self::from_plan(BootPlan::new_with_initrd(kernel_image, num_cores, initrd)?)
    }

    pub fn new_with_initrd_and_bootargs(
        kernel_image: &[u8],
        num_cores: usize,
        initrd: &[u8],
        bootargs: &str,
    ) -> Result<Self, String> {
        Self::from_plan(BootPlan::new_with_initrd_and_bootargs(
            kernel_image,
            num_cores,
            initrd,
            bootargs,
        )?)
    }

    pub fn new_from_iso(iso_image: &[u8], num_cores: usize) -> Result<Self, String> {
        Self::from_plan(BootPlan::new_from_iso(iso_image, num_cores)?)
    }

    pub fn new_from_iso_owned(iso_image: Vec<u8>, num_cores: usize) -> Result<Self, String> {
        Self::from_plan(BootPlan::new_from_iso_owned(iso_image, num_cores)?)
    }

    pub fn from_plan(plan: BootPlan) -> Result<Self, String> {
        let mut machine = Machine::new(plan.num_cores);
        machine
            .bus
            .mem
            .write_bytes(plan.entry, &plan.kernel_image)
            .ok_or_else(|| "kernel image does not fit in guest RAM".to_string())?;

        configure_primary_core(&mut machine.cpus[0], &plan);
        load_initrd(&mut machine.bus, plan.initrd_addr, &plan.initrd_image);
        load_dtb(&mut machine.bus, plan.dtb_addr, &plan.dtb_image);
        if let Some(media) = plan.boot_media {
            machine.bus.virtio_blk.set_image_owned(media);
        }

        Ok(Self {
            machine,
            dtb_addr: plan.dtb_addr,
        })
    }

    /// No-op: EFI stub is skipped. We boot via the standard ARM64 protocol.
    pub fn run_efi_phase(&mut self, _max_steps: usize) -> usize {
        0
    }

    pub fn run_kernel_phase(&mut self, max_steps: usize) -> usize {
        self.machine.run(max_steps)
    }

    pub fn set_install_disk_size(&mut self, size_bytes: u64) {
        self.machine.bus.virtio_disk.set_sparse_disk(size_bytes);
    }

    pub fn attach_virtio_block(&mut self, image: &[u8]) {
        self.machine.bus.virtio_blk.set_image(image);
    }

    pub fn attach_virtio_block_owned(&mut self, image: Vec<u8>) {
        self.machine.bus.virtio_blk.set_image_owned(image);
    }

    pub fn feed_uart_input(&mut self, input: &str) {
        self.feed_uart_bytes(input.as_bytes());
    }

    pub fn feed_uart_bytes(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        self.machine.bus.uart.feed_input_bytes(bytes);
        self.machine.inject_irq(PL011_UART_IRQ_ID);
    }

    pub fn uart_output(&self) -> String {
        self.machine.bus.uart.output_string()
    }

    pub fn uart_output_len(&self) -> usize {
        self.machine.bus.uart.output.len()
    }

    pub fn uart_output_since(&self, offset: usize) -> String {
        let output = &self.machine.bus.uart.output;
        String::from_utf8_lossy(&output[offset.min(output.len())..]).to_string()
    }

    pub fn total_steps(&self) -> u64 {
        self.machine.total_steps
    }

    pub fn pc(&self) -> u64 {
        self.machine.cpus[0].regs.pc
    }

    pub fn allocated_pages(&self) -> usize {
        self.machine.bus.mem.allocated_pages()
    }

    pub fn install_disk_allocated_bytes(&self) -> u64 {
        self.machine.bus.virtio_disk.allocated_storage_bytes()
    }

    pub fn install_disk_size_bytes(&self) -> u64 {
        self.machine.bus.virtio_disk.sparse_disk_size_bytes()
    }

    pub fn install_disk_generation(&self) -> u64 {
        self.machine.bus.virtio_disk.storage_generation()
    }

    pub fn install_disk_snapshot(&self) -> Result<Vec<u8>, String> {
        self.machine.bus.virtio_disk.snapshot_sparse_disk()
    }

    pub fn restore_install_disk(&mut self, snapshot: &[u8]) -> Result<(), String> {
        self.machine.bus.virtio_disk.restore_sparse_disk(snapshot)
    }
}

fn configure_primary_core(cpu: &mut Armv8Cpu, plan: &BootPlan) {
    cpu.regs.set_x(0, plan.dtb_addr);
    cpu.regs.set_x(1, 0);
    cpu.regs.set_x(2, 0);
    cpu.regs.set_x(3, 0);
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(true);
    cpu.sys.sctlr_el1 &= !SCTLR_MMU_ENABLE;
    cpu.regs.pc = plan.entry;
}
