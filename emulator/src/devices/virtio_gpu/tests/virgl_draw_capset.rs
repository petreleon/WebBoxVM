use super::super::VirtioGpu;
use super::super::protocol::*;
use super::super::three_d::VIRGL_CAPSET_ID;
use super::{header, response_type};
use crate::memory::PhysicalMemory;

#[test]
fn virgl_capset_advertises_the_implemented_triangle_requirements() {
    let mut gpu = VirtioGpu::new();
    let mut mem = PhysicalMemory::new();
    let mut request = header(CMD_GET_CAPSET);
    for value in [VIRGL_CAPSET_ID, 1] {
        push_u32(&mut request, value);
    }
    let response = gpu.execute_command(&mut mem, &request);
    assert_eq!(response_type(&response), RESP_OK_CAPSET);
    assert_eq!(read_u32(&response, 24 + 196), Some(1 << 31));
    assert_eq!(read_u32(&response, 24 + 288), Some(1 << 4));
}
