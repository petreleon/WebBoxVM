use super::*;

fn parse(extra: &[&str]) -> Result<Options, String> {
    let args = [
        "--kernel",
        "Image",
        "--initrd",
        "initrd.cpio",
        "--command-file",
        "probe.sh",
        "--uart-log",
        "new.log",
    ];
    Options::parse(args.into_iter().chain(extra.iter().copied()).map(str::to_string))
}

#[test]
fn default_cli_preserves_the_standard_bounds() {
    let options = parse(&[]).unwrap();
    assert_eq!(
        (options.max_steps, options.chunk_steps, options.timeout_secs, options.uart_bytes),
        (MAX_STEPS, CHUNK_STEPS, TIMEOUT_SECS, UART_BYTES)
    );
    assert_eq!(options.ready_marker, "WEBBOXVM_GUEST_READY");
}

#[test]
fn cli_accepts_explicit_fixture_scripts_disk_and_assertions() {
    let options = parse(&[
        "--command-file",
        "drivers.sh",
        "--disk",
        "fixture.wbdisk",
        "--require",
        "Mesa 25.3.6",
        "--ready-marker",
        "FIXTURE_READY",
        "--timeout-secs",
        "30",
    ])
    .unwrap();
    assert_eq!(options.command_files.len(), 2);
    assert_eq!(options.disk.unwrap(), PathBuf::from("fixture.wbdisk"));
    assert_eq!(options.required, ["Mesa 25.3.6"]);
    assert_eq!(options.timeout_secs, 30);
}

#[test]
fn bad_cli_values_are_rejected_without_booting() {
    for args in [
        &["--kernel", "other"][..],
        &["--wat", "value"],
        &["--uart-bytes", "0"],
        &["--timeout-secs", "901"],
        &["--max-steps", "20000000001"],
        &["--chunk-steps", "2000001"],
        &["--require", ""],
        &["--require", "first\nsecond"],
        &["--ready-marker", "READY\r"],
        &["--bootargs", "bad\0args"],
        &["--max-steps", "x"],
        &["--disk"],
    ] {
        assert!(parse(args).is_err(), "{args:?}");
    }
    assert!(Options::parse(std::iter::empty()).is_err());
}

#[test]
fn excessive_script_counts_are_rejected() {
    let mut args = vec!["--command-file", "next.sh"].repeat(16);
    assert!(parse(&args).is_err());
    args.pop();
    assert!(parse(&args).is_err());
}
