use super::super::VirtioGpu;
use super::super::protocol::*;
use super::super::resource::FORMAT_B8G8R8A8_UNORM;
use super::{create_2d, header, response_type};
use crate::constants::RAM_BASE;
use crate::memory::PhysicalMemory;

#[test]
fn page_sg_backing_larger_than_256_entries_is_accepted() {
    const PAGE_COUNT: u32 = 300;
    let mut gpu = VirtioGpu::new();
    let mem = PhysicalMemory::new();
    let create = create_2d(1, FORMAT_B8G8R8A8_UNORM, 1024, PAGE_COUNT);
    assert_eq!(
        response_type(&gpu.execute_command(&mem, &create)),
        RESP_OK_NODATA
    );

    let mut attach = header(CMD_RESOURCE_ATTACH_BACKING);
    push_u32(&mut attach, 1);
    push_u32(&mut attach, PAGE_COUNT);
    for page in 0..PAGE_COUNT {
        push_u64(&mut attach, RAM_BASE + u64::from(page) * 4096);
        push_u32(&mut attach, 4096);
        push_u32(&mut attach, 0);
    }
    assert_eq!(
        response_type(&gpu.execute_command(&mem, &attach)),
        RESP_OK_NODATA
    );
    assert_eq!(gpu.resources.get(&1).unwrap().backing.len(), 300);
}
