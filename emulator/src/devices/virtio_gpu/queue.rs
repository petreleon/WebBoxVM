use super::completion::{
    PendingCompletion, WritableRegion, output_capacity, push_used, response_target_valid,
    write_response,
};
use super::protocol::{CtrlHeader, RESP_ERR_INVALID_PARAMETER};
use super::{QUEUE_NUM_MAX, VirtioGpu};
use crate::memory::PhysicalMemory;

const DESC_F_NEXT: u16 = 1;
const DESC_F_WRITE: u16 = 2;
const DESC_F_INDIRECT: u16 = 4;
const MAX_BACKING_COMMAND_BYTES: usize = 32 + 16 * super::MAX_BACKING_ENTRIES;
const MAX_SUBMIT_COMMAND_BYTES: usize = 32 + super::three_d::packet::MAX_WBG3_PACKET_BYTES;
const MAX_COMMAND_BYTES: usize = if MAX_BACKING_COMMAND_BYTES > MAX_SUBMIT_COMMAND_BYTES {
    MAX_BACKING_COMMAND_BYTES
} else {
    MAX_SUBMIT_COMMAND_BYTES
};
#[derive(Clone, Copy)]
struct Descriptor {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

struct CommandChain {
    input: Vec<u8>,
    output: Vec<Descriptor>,
    malformed: bool,
}

impl VirtioGpu {
    pub(super) fn notify_queue(&mut self, mem: &mut PhysicalMemory, index: u32) -> bool {
        let index = index as usize;
        let Some(queue) = self.queues.get(index).copied() else {
            return false;
        };
        if !queue.ready || queue.num == 0 || queue.num > QUEUE_NUM_MAX {
            return false;
        }
        let Some(avail_idx_addr) = queue.driver.checked_add(2) else {
            return false;
        };
        let Some(avail_idx) = mem.read(avail_idx_addr, 2).map(|value| value as u16) else {
            return false;
        };
        let mut completed = false;
        for _ in 0..queue.num {
            let last = self.queues[index].last_avail_idx;
            if last == avail_idx {
                break;
            }
            let slot = last % queue.num;
            let head_addr = match queue.driver.checked_add(4 + u64::from(slot) * 2) {
                Some(addr) => addr,
                None => break,
            };
            let Some(head) = mem.read(head_addr, 2).map(|value| value as u16) else {
                break;
            };
            self.queues[index].last_avail_idx = last.wrapping_add(1);
            let chain = read_chain(mem, queue.desc, queue.num, head);
            let output = writable_regions(&chain.output);
            let (mut response, mut deferred) =
                if chain.malformed || !response_target_valid(mem, &output, 24) {
                    (error_response(&chain.input), None)
                } else {
                    let result = self.execute_queued_command(mem, &chain.input);
                    (result.response, result.deferred)
                };
            if output_capacity(&output) < response.len() {
                if let Some(command) = deferred.take() {
                    self.cancel_3d(command.sequence);
                }
                response = error_response(&chain.input);
            }
            if let Some(command) = deferred {
                let completion = PendingCompletion {
                    header: command.header,
                    output: output.clone(),
                    used: queue.device,
                    queue_size: queue.num,
                    head,
                };
                if !self.attach_3d_completion(command.sequence, completion) {
                    self.cancel_3d(command.sequence);
                    let error = error_response(&chain.input);
                    let written = write_response(mem, &output, &error).unwrap_or(0);
                    push_used(mem, queue.device, queue.num, head, written as u32);
                    completed = true;
                }
            } else {
                let written = write_response(mem, &output, &response).unwrap_or(0);
                push_used(mem, queue.device, queue.num, head, written as u32);
                completed = true;
            }
        }
        if completed {
            self.interrupt_status |= 1;
        }
        completed
    }
}

fn read_chain(mem: &PhysicalMemory, table: u64, count: u16, head: u16) -> CommandChain {
    let mut result = CommandChain {
        input: Vec::new(),
        output: Vec::new(),
        malformed: false,
    };
    let mut index = head;
    let mut saw_output = false;
    for _ in 0..count {
        let Some(desc) = read_desc(mem, table, count, index) else {
            result.malformed = true;
            break;
        };
        if desc.flags & DESC_F_INDIRECT != 0 {
            result.malformed = true;
            break;
        }
        if desc.flags & DESC_F_WRITE != 0 {
            saw_output = true;
            result.output.push(desc);
        } else if saw_output || !append_input(mem, desc, &mut result.input) {
            result.malformed = true;
        }
        if desc.flags & DESC_F_NEXT == 0 {
            return result;
        }
        index = desc.next;
    }
    result.malformed = true;
    result
}

fn read_desc(mem: &PhysicalMemory, table: u64, count: u16, index: u16) -> Option<Descriptor> {
    if index >= count {
        return None;
    }
    let base = table.checked_add(u64::from(index) * 16)?;
    Some(Descriptor {
        addr: mem.read(base, 8)?,
        len: mem.read(base.checked_add(8)?, 4)? as u32,
        flags: mem.read(base.checked_add(12)?, 2)? as u16,
        next: mem.read(base.checked_add(14)?, 2)? as u16,
    })
}

fn append_input(mem: &PhysicalMemory, desc: Descriptor, input: &mut Vec<u8>) -> bool {
    let Ok(len) = usize::try_from(desc.len) else {
        return false;
    };
    let Some(total) = input.len().checked_add(len) else {
        return false;
    };
    if total > MAX_COMMAND_BYTES || !mem.contains_range(desc.addr, len) {
        return false;
    }
    let old = input.len();
    input.resize(total, 0);
    mem.read_bytes(desc.addr, &mut input[old..]).is_some()
}

fn writable_regions(output: &[Descriptor]) -> Vec<WritableRegion> {
    output
        .iter()
        .map(|desc| WritableRegion {
            addr: desc.addr,
            len: desc.len,
        })
        .collect()
}

fn error_response(input: &[u8]) -> Vec<u8> {
    CtrlHeader::decode(input)
        .unwrap_or_default()
        .encode(RESP_ERR_INVALID_PARAMETER)
}
