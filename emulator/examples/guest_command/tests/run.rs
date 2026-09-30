use super::*;
use std::collections::VecDeque;

struct Fixture {
    output: Vec<u8>,
    pending: VecDeque<Vec<u8>>,
    inputs: Vec<String>,
    failure: &'static [u8],
    respond: bool,
}

impl Guest for Fixture {
    fn run(&mut self, _: usize) -> usize {
        if let Some(bytes) = self.pending.pop_front() {
            self.output.extend_from_slice(&bytes);
        }
        0
    }
    fn output(&self) -> &[u8] {
        &self.output
    }
    fn input(&mut self, text: &str) {
        self.inputs.push(text.into());
        if !self.respond {
            return;
        }
        let reply = match self.inputs.len() {
            1 => b"WEBBOXVM_GUEST_READY:abcd:0\n".to_vec(),
            2 => b"WEBBOXVM_GUEST_BEGIN:abcd:0\nMesa 25.3.6\nWEBBOXVM_GUEST_STATUS:abcd:0:0\n".to_vec(),
            3 => [b"WEBBOXVM_GUEST_DONE:abcd\n".as_slice(), self.failure].concat(),
            _ => panic!("extra host action"),
        };
        self.pending.push_back(reply);
    }
    fn pc(&self) -> u64 {
        0
    }
}

fn fixture(failure: &'static [u8], respond: bool) -> Fixture {
    Fixture {
        output: Vec::new(),
        pending: VecDeque::from([b"BOOT_READY\n".to_vec()]),
        inputs: Vec::new(),
        failure,
        respond,
    }
}

fn run_fixture(vm: &mut Fixture) -> (Result<(), String>, Vec<u8>, Budget) {
    let mut budget = Budget::new(10, 1, 1).unwrap();
    let mut protocol = Protocol::new("abcd".into(), "BOOT_READY".into(), 1, vec!["Mesa 25.3.6".into()], 4096);
    let scripts = Scripts::new("abcd", &["printf 'Mesa 25.3.6\\n'".into()]).unwrap();
    let mut receipt = Vec::new();
    let result = execute(vm, &mut budget, &mut protocol, &scripts, &mut receipt, &Instant::now());
    (result, receipt, budget)
}

#[test]
fn runner_requires_guest_status_and_the_control_shell_round_trip() {
    let mut vm = fixture(b"", true);
    let (result, receipt, budget) = run_fixture(&mut vm);
    result.unwrap();
    assert_eq!(receipt, vm.output);
    assert_eq!(vm.inputs.len(), 3);
    assert!(vm.inputs[0].starts_with("stty -echo"));
    assert!(vm.inputs[1].contains("sh -e"));
    assert!(vm.inputs[2].contains("DONE abcd"));
    assert_eq!((budget.steps, budget.chunks), (0, 4));
}

#[test]
fn final_chunk_failure_is_saved_and_prevents_success() {
    for failure in [b"Kernel panic - not syncing".as_slice(), b"WEBBOXVM_GUEST_DONE:abcd\n", b"\xff"] {
        let mut vm = fixture(failure, true);
        let (result, receipt, _) = run_fixture(&mut vm);
        assert!(result.is_err());
        assert_eq!(receipt, vm.output);
    }
}

#[test]
fn missing_guest_response_exhausts_chunks_even_with_zero_instructions() {
    let mut vm = fixture(b"", false);
    let (result, receipt, budget) = run_fixture(&mut vm);
    assert!(result.unwrap_err().contains("budget exhausted"));
    assert_eq!((budget.steps, budget.chunks), (0, 10));
    assert_eq!(vm.inputs.len(), 1);
    assert_eq!(receipt, b"BOOT_READY\n");
}
