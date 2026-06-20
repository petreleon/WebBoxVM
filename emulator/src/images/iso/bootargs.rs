pub(super) const ISO_VIRTIO_MMIO_ARG: &str = "virtio_mmio.device=4K@0x0a000000:48";
pub(super) const DISK_VIRTIO_MMIO_ARG: &str = "virtio_mmio.device=4K@0x0a001000:49";
pub(super) const NET_VIRTIO_MMIO_ARG: &str = "virtio_mmio.device=4K@0x0a002000:50";
pub(super) const DEFAULT_ISO_BOOTARGS: &str = "earlycon=pl011,0x09000000 console=ttyAMA0,115200n8 loglevel=7 kvm-arm.mode=none kvm.enable_virt_at_load=0 initcall_blacklist=finalize_pkvm,bpf_tcp_ca_kfunc_init cryptomgr.notests=1 virtio_mmio.device=4K@0x0a000000:48 virtio_mmio.device=4K@0x0a001000:49 virtio_mmio.device=4K@0x0a002000:50 clocksource.arm_arch_timer.evtstrm=false auto=false";
const DI_SINGLE_CONSOLE_ARG: &str = "auto=false";

pub(super) fn ensure_serial_bootargs(args: &str) -> String {
    let trimmed = args.trim();
    if trimmed.is_empty() {
        return DEFAULT_ISO_BOOTARGS.to_string();
    }

    let mut tokens: Vec<String> = trimmed.split_whitespace().map(str::to_string).collect();
    ensure_kernel_arg(&mut tokens, "earlycon=", "earlycon=pl011,0x09000000");
    ensure_kernel_arg(&mut tokens, "console=ttyAMA", "console=ttyAMA0,115200n8");
    ensure_kernel_arg(&mut tokens, "loglevel=", "loglevel=7");
    ensure_kernel_arg(&mut tokens, "kvm-arm.mode=", "kvm-arm.mode=none");
    ensure_kernel_arg(
        &mut tokens,
        "kvm.enable_virt_at_load=",
        "kvm.enable_virt_at_load=0",
    );
    ensure_kernel_arg(
        &mut tokens,
        "initcall_blacklist=",
        "initcall_blacklist=finalize_pkvm,bpf_tcp_ca_kfunc_init",
    );
    ensure_kernel_arg(&mut tokens, "cryptomgr.notests=", "cryptomgr.notests=1");
    ensure_kernel_token(&mut tokens, ISO_VIRTIO_MMIO_ARG);
    ensure_kernel_token(&mut tokens, DISK_VIRTIO_MMIO_ARG);
    ensure_kernel_token(&mut tokens, NET_VIRTIO_MMIO_ARG);
    ensure_kernel_arg(
        &mut tokens,
        "clocksource.arm_arch_timer.evtstrm=",
        "clocksource.arm_arch_timer.evtstrm=false",
    );
    ensure_kernel_arg(&mut tokens, "auto=", DI_SINGLE_CONSOLE_ARG);
    remove_installer_arg(&mut tokens, "quiet");
    ensure_installer_arg(&mut tokens, "console=ttyAMA", "console=ttyAMA0,115200n8");
    ensure_installer_arg(&mut tokens, "DEBIAN_FRONTEND=", "DEBIAN_FRONTEND=text");
    ensure_installer_arg(&mut tokens, "TERM=", "TERM=vt102");
    tokens.join(" ")
}

fn ensure_kernel_arg(tokens: &mut Vec<String>, prefix: &str, arg: &str) {
    let insert_at = kernel_arg_insert_index(tokens);
    if !tokens[..insert_at]
        .iter()
        .any(|token| token.starts_with(prefix))
    {
        tokens.insert(insert_at, arg.to_string());
    }
}

fn ensure_kernel_token(tokens: &mut Vec<String>, arg: &str) {
    let insert_at = kernel_arg_insert_index(tokens);
    if !tokens[..insert_at].iter().any(|token| token == arg) {
        tokens.insert(insert_at, arg.to_string());
    }
}

fn kernel_arg_insert_index(tokens: &[String]) -> usize {
    tokens
        .iter()
        .position(|token| token == "---" || token == "--")
        .unwrap_or(tokens.len())
}

fn ensure_installer_arg(tokens: &mut Vec<String>, prefix: &str, arg: &str) {
    let insert_at = installer_arg_insert_index(tokens);
    if !tokens[insert_at..]
        .iter()
        .any(|token| token.starts_with(prefix))
    {
        tokens.insert(insert_at, arg.to_string());
    }
}

fn remove_installer_arg(tokens: &mut Vec<String>, arg: &str) {
    if let Some(separator) = tokens
        .iter()
        .position(|token| token == "---" || token == "--")
    {
        let kept: Vec<String> = tokens
            .drain(separator + 1..)
            .filter(|token| token != arg)
            .collect();
        tokens.extend(kept);
    }
}

fn installer_arg_insert_index(tokens: &mut Vec<String>) -> usize {
    match tokens
        .iter()
        .position(|token| token == "---" || token == "--")
    {
        Some(separator) => separator + 1,
        None => {
            tokens.push("---".to_string());
            tokens.len()
        }
    }
}
