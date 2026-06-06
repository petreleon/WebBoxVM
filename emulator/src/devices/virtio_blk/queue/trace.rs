use std::env;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;

const DEFAULT_TRACE_LIMIT: u32 = 4096;

pub(super) fn trace_request(id: &[u8], req_type: u32, sector: u64, bytes: u32, status: u8) {
    if !trace_enabled() {
        return;
    }
    let index = trace_counter().fetch_add(1, Ordering::Relaxed);
    if index >= trace_limit() {
        return;
    }
    eprintln!(
        "VIRTIO_BLK req={} dev={} type={} sector={} bytes={} status={}",
        index,
        id_text(id),
        req_name(req_type),
        sector,
        bytes,
        status
    );
}

fn trace_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| env::var_os("WEBBOXVM_TRACE_VIRTIO_BLK").is_some())
}

fn trace_limit() -> u32 {
    static LIMIT: OnceLock<u32> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        env::var("WEBBOXVM_TRACE_VIRTIO_BLK_LIMIT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_TRACE_LIMIT)
    })
}

fn trace_counter() -> &'static AtomicU32 {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    &COUNTER
}

fn id_text(id: &[u8]) -> String {
    let end = id.iter().position(|byte| *byte == 0).unwrap_or(id.len());
    String::from_utf8_lossy(&id[..end]).to_string()
}

fn req_name(req_type: u32) -> &'static str {
    match req_type {
        0 => "IN",
        1 => "OUT",
        4 => "FLUSH",
        8 => "GET_ID",
        _ => "UNKNOWN",
    }
}
