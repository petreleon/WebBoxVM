use super::*;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn shell(script: &str) -> String {
    let mut process = Command::new("/bin/sh")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    process.stdin.take().unwrap().write_all(script.replace('\r', "\n").as_bytes()).unwrap();
    let result = process.wait_with_output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    String::from_utf8(result.stdout).unwrap()
}

fn execute(source: &str) -> String {
    let token = format!("{:x}{:x}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed));
    let scripts = Scripts::new(&token, &[source.into()]).unwrap();
    let result = shell(&scripts.commands[0]);
    std::fs::remove_file(format!("/tmp/webboxvm-guest-{token}-0.sh")).unwrap();
    result.replace(&token, "TOKEN")
}

#[test]
fn arbitrary_quotes_unicode_and_multiline_source_round_trip_through_real_shell() {
    let output =
        execute("name='ARM64'; printf 'Mesa 25.3.6 — %s\\n' \"$name\"\nprintf '%s\\n' \"single'quote\"");
    assert_eq!(
        output,
        "\nWEBBOXVM_GUEST_BEGIN:TOKEN:0\nMesa 25.3.6 — ARM64\nsingle'quote\n\nWEBBOXVM_GUEST_STATUS:TOKEN:0:0\n"
    );
}

#[test]
fn child_exit_and_errexit_propagate_into_guest_status() {
    assert!(execute("exit 7").ends_with("WEBBOXVM_GUEST_STATUS:TOKEN:0:7\n"));
    let output = execute("false\nprintf 'should not run\\n'");
    assert!(!output.contains("should not run"));
    assert!(output.ends_with("WEBBOXVM_GUEST_STATUS:TOKEN:0:1\n"));
}

#[test]
fn child_stdin_is_closed_and_cannot_consume_the_control_protocol() {
    let output = execute("if read word; then exit 9; fi\nprintf 'stdin closed\\n'");
    assert!(output.contains("\nstdin closed\n"));
    assert!(output.ends_with("WEBBOXVM_GUEST_STATUS:TOKEN:0:0\n"));
}

#[test]
fn decoder_failure_is_not_hidden_by_a_successful_final_printf() {
    let scripts = Scripts::new("ffff", &["printf 'should not run'".into()]).unwrap();
    let output = shell(&scripts.commands[0].replacen("base64 -d", "false", 1));
    std::fs::remove_file("/tmp/webboxvm-guest-ffff-0.sh").unwrap();
    assert!(!output.contains("should not run"));
    assert!(output.ends_with("WEBBOXVM_GUEST_STATUS:ffff:0:1\n"));
}

#[test]
fn input_caps_and_invalid_sources_are_checked_before_guest_execution() {
    for source in ["".to_string(), " \n".into(), "x\0y".into(), "x".repeat(INPUT_BYTES)] {
        assert!(Scripts::new("a", &[source]).is_err());
    }
    assert!(Scripts::new("unsafe'", &["true".into()]).is_err());
    assert!(Scripts::new("a", &[]).is_err());
    assert!(Scripts::new("a", &vec!["true".into(); 17]).is_err());
}

#[test]
fn echoed_input_cannot_contain_any_complete_protocol_marker() {
    let scripts = Scripts::new("abcd", &["printf 'hello\\n'".into()]).unwrap();
    for input in [&scripts.handshake, &scripts.commands[0], &scripts.finish] {
        for marker in ["READY", "BEGIN", "STATUS", "DONE"] {
            assert!(!input.contains(&format!("WEBBOXVM_GUEST_{marker}:abcd")));
        }
    }
}
