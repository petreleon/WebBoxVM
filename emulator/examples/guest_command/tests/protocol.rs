use super::*;

fn protocol(required: &[&str], count: usize) -> Protocol {
    Protocol::new(
        "abcd".into(),
        "BOOT_READY".into(),
        count,
        required.iter().map(|s| (*s).into()).collect(),
        4096,
    )
}

fn started(required: &[&str]) -> Protocol {
    let mut p = protocol(required, 1);
    p.feed(b"BOOT_READY\r\n").unwrap();
    assert!(matches!(p.take_action(), Some(Action::Handshake)));
    p.feed(b"printf 'WEBBOXVM_GUEST_%s:%s:%s' READY abcd $r\r\n").unwrap();
    p.feed(b"WEBBOXVM_GUEST_READY:abcd:0\n").unwrap();
    assert!(matches!(p.take_action(), Some(Action::Command(0))));
    p.feed(b"WEBBOXVM_GUEST_BEGIN:abcd:0\n").unwrap();
    p
}

fn finish(p: &mut Protocol) -> Result<(), String> {
    p.feed(b"WEBBOXVM_GUEST_STATUS:abcd:0:0\n")?;
    assert!(matches!(p.take_action(), Some(Action::Finish)));
    p.feed(b"WEBBOXVM_GUEST_DONE:abcd\n")
}

#[test]
fn partial_utf8_and_markers_are_preserved_across_every_chunk_boundary() {
    let output = "Mesa 25.3.6 — ARM64\r\n";
    for split in 0..=output.len() {
        let mut p = started(&["Mesa 25.3.6 — ARM64"]);
        p.feed(&output.as_bytes()[..split]).unwrap();
        p.feed(&output.as_bytes()[split..]).unwrap();
        finish(&mut p).unwrap();
        assert!(p.finished());
    }
    let mut p = started(&[]);
    for byte in b"WEBBOXVM_GUEST_STATUS:abcd:0:0\n" {
        p.feed(&[*byte]).unwrap();
    }
    assert!(matches!(p.take_action(), Some(Action::Finish)));
    for byte in b"WEBBOXVM_GUEST_DONE:abcd\n" {
        p.feed(&[*byte]).unwrap();
    }
    assert!(p.finished());
}

#[test]
fn failures_and_malformed_or_duplicate_markers_cannot_be_success() {
    for line in [
        "WEBBOXVM_GUEST_STATUS:abcd:0:7",
        "WEBBOXVM_GUEST_STATUS:abcd:0:x",
        "WEBBOXVM_GUEST_STATUS:abcd:00:0",
        "WEBBOXVM_GUEST_STATUS:abcd:0:00",
        "WEBBOXVM_GUEST_STATUS:ffff:0:0",
        "WEBBOXVM_GUEST_DONE:abcd",
        " WEBBOXVM_GUEST_STATUS:abcd:0:0",
        "WEBBOXVM_GUEST_STATUS:abcd:0:0\t",
    ] {
        assert!(started(&[]).feed(format!("{line}\n").as_bytes()).is_err(), "{line}");
    }
    let mut p = started(&[]);
    assert!(p.feed(b"WEBBOXVM_GUEST_STATUS:abcd:0:0\nWEBBOXVM_GUEST_STATUS:abcd:0:0\n").is_err());
}

#[test]
fn required_output_cannot_come_from_boot_echo_or_substrings() {
    let mut p = protocol(&["Mesa 25.3.6"], 1);
    p.feed(b"Mesa 25.3.6\nBOOT_READY\n").unwrap();
    p.take_action();
    p.feed(b"WEBBOXVM_GUEST_READY:abcd:0\n").unwrap();
    p.take_action();
    p.feed(b"WEBBOXVM_GUEST_BEGIN:abcd:0\nMesa 25.3.60\nmissing Mesa 25.3.6\n").unwrap();
    assert!(finish(&mut p).is_err());
    let mut p = started(&["Mesa 25.3.6"]);
    p.feed(b"Mesa 25.3.6\r\r\n").unwrap();
    assert!(finish(&mut p).is_err());
}

#[test]
fn nonzero_echo_disable_status_fails_before_any_command() {
    let mut p = protocol(&[], 1);
    p.feed(b"BOOT_READY\n").unwrap();
    p.take_action();
    assert!(p.feed(b"WEBBOXVM_GUEST_READY:abcd:1\n").is_err());
}

#[test]
fn panic_invalid_utf8_and_trailing_partial_failures_are_rejected() {
    for bad in [
        b"Kernel panic - not syncing\n".as_slice(),
        b"Oops: 96000004\n",
        b"BUG: bad state\n",
        b"Unable to handle kernel paging\n",
        b"\xff\n",
    ] {
        assert!(started(&[]).feed(bad).is_err());
    }
    for bad in [b"Kernel panic - not syncing".as_slice(), b"\xff", b"WEBBOXVM_GUEST_STATUS:abcd:0:0"] {
        let mut p = started(&[]);
        p.feed(b"WEBBOXVM_GUEST_STATUS:abcd:0:0\n").unwrap();
        p.take_action();
        let mut bytes = b"WEBBOXVM_GUEST_DONE:abcd\n".to_vec();
        bytes.extend_from_slice(bad);
        assert!(p.feed(&bytes).is_err());
    }
}

#[test]
fn uart_cap_and_missing_terminated_status_never_produce_success() {
    let mut p = Protocol::new("abcd".into(), "READY".into(), 1, vec![], 4);
    p.feed(b"1234").unwrap();
    assert!(p.feed(b"5").is_err());
    let mut p = started(&[]);
    p.feed(b"WEBBOXVM_GUEST_STATUS:abcd:0:0").unwrap();
    assert!(!p.finished());
    assert!(p.take_action().is_none());
}

#[test]
fn multiple_commands_require_ordered_host_actions_and_zero_statuses() {
    let mut p = protocol(&["first", "second"], 2);
    p.feed(b"BOOT_READY\n").unwrap();
    p.take_action();
    p.feed(b"WEBBOXVM_GUEST_READY:abcd:0\n").unwrap();
    p.take_action();
    p.feed(b"WEBBOXVM_GUEST_BEGIN:abcd:0\nfirst\nWEBBOXVM_GUEST_STATUS:abcd:0:0\n").unwrap();
    assert!(matches!(p.take_action(), Some(Action::Command(1))));
    p.feed(b"WEBBOXVM_GUEST_BEGIN:abcd:1\nsecond\nWEBBOXVM_GUEST_STATUS:abcd:1:0\n").unwrap();
    assert!(matches!(p.take_action(), Some(Action::Finish)));
    p.feed(b"WEBBOXVM_GUEST_DONE:abcd\n").unwrap();
    assert!(p.finished());
}
