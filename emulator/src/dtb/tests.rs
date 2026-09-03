use super::*;
use crate::platform::virt::SystemBus;
use std::collections::BTreeMap;

type Properties = BTreeMap<String, Vec<u8>>;

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_string(bytes: &[u8], offset: usize) -> (&str, usize) {
    let end = bytes[offset..]
        .iter()
        .position(|byte| *byte == 0)
        .map(|length| offset + length)
        .unwrap();
    let value = std::str::from_utf8(&bytes[offset..end]).unwrap();
    (value, end + 1)
}

fn node_path(stack: &[String]) -> String {
    let names: Vec<_> = stack.iter().filter(|name| !name.is_empty()).collect();
    format!(
        "/{}",
        names.into_iter().cloned().collect::<Vec<_>>().join("/")
    )
}

fn parse_nodes(dtb: &[u8]) -> BTreeMap<String, Properties> {
    let mut cursor = read_u32(dtb, 8) as usize;
    let strings = read_u32(dtb, 12) as usize;
    let mut stack = Vec::new();
    let mut nodes = BTreeMap::new();

    loop {
        let token = read_u32(dtb, cursor);
        cursor += 4;
        match token {
            FDT_BEGIN_NODE => {
                let (name, next) = read_string(dtb, cursor);
                cursor = (next + 3) & !3;
                stack.push(name.to_string());
                nodes
                    .entry(node_path(&stack))
                    .or_insert_with(Properties::new);
            }
            FDT_END_NODE => {
                stack.pop();
            }
            FDT_PROP => {
                let length = read_u32(dtb, cursor) as usize;
                let name_offset = read_u32(dtb, cursor + 4) as usize;
                cursor += 8;
                let (name, _) = read_string(dtb, strings + name_offset);
                let value = dtb[cursor..cursor + length].to_vec();
                cursor = (cursor + length + 3) & !3;
                nodes
                    .get_mut(&node_path(&stack))
                    .unwrap()
                    .insert(name.to_string(), value);
            }
            FDT_END => break,
            token => panic!("unexpected FDT token {token:#x}"),
        }
    }
    nodes
}

fn property_u32(properties: &Properties, name: &str) -> u32 {
    u32::from_be_bytes(properties[name].as_slice().try_into().unwrap())
}

#[test]
fn dtb_magic_and_size() {
    let dtb = build_dtb(0x4000_0000, 0x4000_0000, None, None, None);
    assert!(dtb.len() >= 40);
    let magic = u32::from_be_bytes([dtb[0], dtb[1], dtb[2], dtb[3]]);
    assert_eq!(magic, 0xd00dfeed);
    let totalsize = u32::from_be_bytes([dtb[4], dtb[5], dtb[6], dtb[7]]);
    assert_eq!(totalsize as usize, dtb.len());
}

#[test]
fn dtb_with_initrd() {
    let dtb = build_dtb(
        0x4000_0000,
        0x4000_0000,
        Some(0x4200_0000),
        Some(0x4300_0000),
        Some("console=ttyAMA0"),
    );
    assert!(dtb.len() > 40);
    // Verify we can load it into memory
    let mut bus = SystemBus::new();
    load_dtb(&mut bus, 0x4800_0000, &dtb);
    let magic = u32::from_be_bytes([
        bus.mem.read(0x4800_0000, 1).unwrap() as u8,
        bus.mem.read(0x4800_0001, 1).unwrap() as u8,
        bus.mem.read(0x4800_0002, 1).unwrap() as u8,
        bus.mem.read(0x4800_0003, 1).unwrap() as u8,
    ]);
    assert_eq!(magic, 0xd00dfeed);
}

#[test]
fn dtb_advertises_virtio_network_mmio_device() {
    let dtb = build_dtb(0x4000_0000, 0x4000_0000, None, None, None);
    let text = String::from_utf8_lossy(&dtb);

    assert!(text.contains("virtio_blk@a000000"));
    assert!(text.contains("virtio_blk@a001000"));
    assert!(text.contains("virtio_net@a002000"));
    assert!(text.contains("virtio_gpu@a003000"));
    assert!(text.contains("virtio,mmio"));
}

#[test]
fn dtb_can_omit_boot_media_block_device() {
    let dtb = build_dtb_with_boot_media_device(0x4000_0000, 0x4000_0000, None, None, None, false);
    let text = String::from_utf8_lossy(&dtb);

    assert!(!text.contains("virtio_blk@a000000"));
    assert!(text.contains("virtio_blk@a001000"));
    assert!(text.contains("virtio_net@a002000"));
    assert!(text.contains("virtio_gpu@a003000"));
}

#[test]
fn legacy_dtb_builders_default_to_one_core() {
    let dtbs = [
        build_dtb(0x4000_0000, 0x4000_0000, None, None, None),
        build_dtb_with_boot_media_device(0x4000_0000, 0x4000_0000, None, None, None, false),
    ];

    for dtb in dtbs {
        let nodes = parse_nodes(&dtb);
        assert!(nodes.contains_key("/cpus/cpu@0"));
        assert!(!nodes.contains_key("/cpus/cpu@1"));
    }
}

#[test]
fn counted_dtb_advertises_psci_cpu_topology() {
    for num_cores in [2, 4] {
        let dtb = build_dtb_with_num_cores(0x4000_0000, 0x4000_0000, None, None, None, num_cores);
        let nodes = parse_nodes(&dtb);
        let cpus = &nodes["/cpus"];

        assert_eq!(property_u32(cpus, "#address-cells"), 1);
        assert_eq!(property_u32(cpus, "#size-cells"), 0);
        assert_eq!(
            nodes
                .keys()
                .filter(|path| path.starts_with("/cpus/cpu@"))
                .count(),
            num_cores
        );

        for core_id in 0..num_cores {
            let cpu = &nodes[&format!("/cpus/cpu@{core_id:x}")];
            assert_eq!(cpu["device_type"], b"cpu\0");
            assert_eq!(cpu["compatible"], b"arm,armv8\0");
            assert_eq!(property_u32(cpu, "reg"), core_id as u32);
            assert_eq!(cpu["enable-method"], b"psci\0");
        }

        let psci = &nodes["/psci"];
        assert_eq!(psci["compatible"], b"arm,psci-0.2\0");
        assert_eq!(psci["method"], b"hvc\0");
    }
}
