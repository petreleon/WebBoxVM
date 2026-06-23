use super::*;

pub(in crate::arch::arm64::execute::load_store) fn read_pair_scalars(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    pas: (u64, u64),
    size: u8,
    err: &'static str,
) -> Result<(u64, u64), &'static str> {
    if !access_crosses_page(va, size * 2) {
        let len = size as usize * 2;
        let mut bytes = [0; 16];
        read_guest_bytes_from_first_pa(cpu, bus, va, pas.0, &mut bytes[..len], err, err)?;
        return Ok((
            read_le(&bytes[..size as usize]),
            read_le(&bytes[size as usize..len]),
        ));
    }
    let lo = read_guest_translated(cpu, bus, va, pas.0, size, err)?;
    let hi = read_guest_translated(cpu, bus, va + size as u64, pas.1, size, err)?;
    Ok((lo, hi))
}

pub(in crate::arch::arm64::execute::load_store) fn write_pair_scalars(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    pa: u64,
    size: u8,
    val1: u64,
    val2: u64,
    err: &'static str,
) -> Result<(), &'static str> {
    let len = size as usize * 2;
    let mut bytes = [0; 16];
    bytes[..size as usize].copy_from_slice(&val1.to_le_bytes()[..size as usize]);
    bytes[size as usize..len].copy_from_slice(&val2.to_le_bytes()[..size as usize]);
    write_guest_bytes_from_first_pa(cpu, bus, va, pa, &bytes[..len], err).map(|_| ())
}

fn read_le(bytes: &[u8]) -> u64 {
    bytes.iter().enumerate().fold(0, |value, (offset, byte)| {
        value | ((*byte as u64) << (offset * 8))
    })
}
