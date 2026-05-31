use super::super::*;
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

const QUEUE_DESC: u64 = RAM_BASE + 0x1000;
const QUEUE_DRIVER: u64 = RAM_BASE + 0x2000;
const QUEUE_DEVICE: u64 = RAM_BASE + 0x3000;
const REQ_ADDR: u64 = RAM_BASE + 0x4000;
const DATA_ADDR: u64 = RAM_BASE + 0x5000;
const STATUS_ADDR: u64 = RAM_BASE + 0x6000;

fn configure_queue(device: &mut VirtioBlk, mem: &mut PhysicalMemory) {
    device.write(mem, 0x038, 8, 4);
    device.write(mem, 0x080, QUEUE_DESC as u32 as u64, 4);
    device.write(mem, 0x084, QUEUE_DESC >> 32, 4);
    device.write(mem, 0x090, QUEUE_DRIVER as u32 as u64, 4);
    device.write(mem, 0x094, QUEUE_DRIVER >> 32, 4);
    device.write(mem, 0x0a0, QUEUE_DEVICE as u32 as u64, 4);
    device.write(mem, 0x0a4, QUEUE_DEVICE >> 32, 4);
    device.write(mem, 0x044, 1, 4);
}

fn write_desc(mem: &mut PhysicalMemory, index: u16, addr: u64, len: u32, flags: u16, next: u16) {
    let base = QUEUE_DESC + index as u64 * 16;
    mem.write(base, 8, addr).unwrap();
    mem.write(base + 8, 4, len as u64).unwrap();
    mem.write(base + 12, 2, flags as u64).unwrap();
    mem.write(base + 14, 2, next as u64).unwrap();
}

fn submit_request(
    device: &mut VirtioBlk,
    mem: &mut PhysicalMemory,
    req_type: u32,
    sector: u64,
    data_len: u32,
    data_flags: u16,
    avail_idx: u16,
) -> u8 {
    mem.write(REQ_ADDR, 4, req_type as u64).unwrap();
    mem.write(REQ_ADDR + 4, 4, 0).unwrap();
    mem.write(REQ_ADDR + 8, 8, sector).unwrap();
    mem.write(STATUS_ADDR, 1, 0xff).unwrap();

    write_desc(mem, 0, REQ_ADDR, 16, VIRTQ_DESC_F_NEXT, 1);
    write_desc(
        mem,
        1,
        DATA_ADDR,
        data_len,
        data_flags | VIRTQ_DESC_F_NEXT,
        2,
    );
    write_desc(mem, 2, STATUS_ADDR, 1, VIRTQ_DESC_F_WRITE, 0);

    let ring_slot = avail_idx % 8;
    mem.write(QUEUE_DRIVER + 4 + ring_slot as u64 * 2, 2, 0)
        .unwrap();
    mem.write(QUEUE_DRIVER + 2, 2, avail_idx.wrapping_add(1) as u64)
        .unwrap();
    assert!(device.write(mem, 0x050, 0, 4));

    mem.read(STATUS_ADDR, 1).unwrap() as u8
}

#[test]
fn virtqueue_writes_to_sparse_disk_and_reads_back() {
    let mut mem = PhysicalMemory::new();
    let mut device = VirtioBlk::writable_sparse(SECTOR_SIZE as u64 * 8, b"disk\0");
    configure_queue(&mut device, &mut mem);

    mem.write_bytes(DATA_ADDR, b"hello").unwrap();
    assert_eq!(
        submit_request(&mut device, &mut mem, VIRTIO_BLK_T_OUT, 2, 5, 0, 0,),
        VIRTIO_BLK_S_OK
    );
    assert_eq!(
        device.allocated_storage_bytes(),
        SPARSE_DISK_CHUNK_SIZE as u64
    );

    mem.write_bytes(DATA_ADDR, &[0; 5]).unwrap();
    assert_eq!(
        submit_request(
            &mut device,
            &mut mem,
            VIRTIO_BLK_T_IN,
            2,
            5,
            VIRTQ_DESC_F_WRITE,
            1,
        ),
        VIRTIO_BLK_S_OK
    );

    let mut out = [0u8; 5];
    mem.read_bytes(DATA_ADDR, &mut out).unwrap();
    assert_eq!(&out, b"hello");
}
