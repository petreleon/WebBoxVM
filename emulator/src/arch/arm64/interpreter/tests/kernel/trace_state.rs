use std::collections::{HashMap, VecDeque};

pub(super) type EfiFrame = (u64, u64, u64, u64, u64, u64);

pub(super) struct EfiTrace {
    pub(super) fp_to_name: HashMap<u64, String>,
    pub(super) stack: Vec<EfiFrame>,
    pub(super) log: Vec<String>,
    pub(super) recent: VecDeque<String>,
    pub(super) file: Option<std::fs::File>,
}

impl EfiTrace {
    pub(super) fn new(bus: &super::SystemBus) -> Self {
        Self {
            fp_to_name: super::build_fp_to_name(bus),
            stack: Vec::new(),
            log: Vec::new(),
            recent: VecDeque::with_capacity(120),
            file: std::fs::File::create("/tmp/kernel_trace.txt").ok(),
        }
    }

    pub(super) fn resolve(&self, fp: u64) -> String {
        self.fp_to_name
            .get(&fp)
            .cloned()
            .unwrap_or_else(|| format!("EFI@{:#x}", fp))
    }
}
