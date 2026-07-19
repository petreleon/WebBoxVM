pub(super) const KNOWN_SERIAL_GETTY_EXEC_STARTS: [&[u8]; 2] = [
    br#"-/sbin/agetty -o '-- \\u' --noreset --noclear --keep-baud 115200,57600,38400,9600 - ${TERM}"#,
    br#"-/sbin/agetty -o '-p -- \\u' --keep-baud 115200,57600,38400,9600 - $TERM"#,
];

#[derive(Default)]
struct RequiredDirectives {
    unit_section: usize,
    service_section: usize,
    install_section: usize,
    description: usize,
    device_binding: usize,
    exec_start: usize,
    service_type: usize,
    standard_input: usize,
    standard_output: usize,
    tty_path: usize,
    wanted_by: usize,
}

pub(super) fn usable_serial_getty_template(data: &[u8]) -> bool {
    let mut section = b"".as_slice();
    let mut required = RequiredDirectives::default();
    for line in data.split(|byte| *byte == b'\n').map(trim_ascii) {
        if line.ends_with(b"\\") {
            return false;
        }
        if line.is_empty() || line.starts_with(b"#") {
            continue;
        }
        if line.starts_with(b"[") && line.ends_with(b"]") {
            section = &line[1..line.len() - 1];
            match section {
                b"Unit" => required.unit_section += 1,
                b"Service" => required.service_section += 1,
                b"Install" => required.install_section += 1,
                _ => return false,
            }
            continue;
        }
        let Some((key, value)) = split_directive(line) else {
            return false;
        };
        if !known_directive(section, key, value) {
            return false;
        }
        match (section, key) {
            (b"Unit", b"Description") => required.description += 1,
            (b"Unit", b"BindsTo") => required.device_binding += 1,
            (b"Service", b"ExecStart") => required.exec_start += 1,
            (b"Service", b"Type") => required.service_type += 1,
            (b"Service", b"StandardInput") => required.standard_input += 1,
            (b"Service", b"StandardOutput") => required.standard_output += 1,
            (b"Service", b"TTYPath") => required.tty_path += 1,
            (b"Install", b"WantedBy") => required.wanted_by += 1,
            _ => {}
        }
    }
    [
        required.unit_section,
        required.service_section,
        required.install_section,
        required.description,
        required.device_binding,
        required.exec_start,
        required.service_type,
        required.standard_input,
        required.standard_output,
        required.tty_path,
        required.wanted_by,
    ]
    .into_iter()
    .all(|count| count == 1)
}

fn split_directive(line: &[u8]) -> Option<(&[u8], &[u8])> {
    let equals = line.iter().position(|byte| *byte == b'=')?;
    let key = trim_ascii(&line[..equals]);
    let value = trim_ascii(&line[equals + 1..]);
    (!key.is_empty() && !value.is_empty()).then_some((key, value))
}

fn known_directive(section: &[u8], key: &[u8], value: &[u8]) -> bool {
    match (section, key) {
        (b"Unit", b"Description") => value == b"Serial Getty on %I",
        (b"Unit", b"Documentation") => matches!(
            value,
            b"man:agetty(8) man:systemd-getty-generator(8)"
                | b"https://0pointer.de/blog/projects/serial-console.html"
        ),
        (b"Unit", b"BindsTo") => value == b"dev-%i.device",
        (b"Unit", b"After") => matches!(
            value,
            b"dev-%i.device systemd-user-sessions.service plymouth-quit-wait.service getty-pre.target"
                | b"rc-local.service"
        ),
        (b"Unit", b"Before") => matches!(value, b"getty.target" | b"rescue.service"),
        (b"Unit", b"IgnoreOnIsolate") => value == b"yes",
        (b"Unit", b"Conflicts") => value == b"rescue.service",
        (b"Service", b"ExecStart") => KNOWN_SERIAL_GETTY_EXEC_STARTS.contains(&value),
        (b"Service", b"Type") => value == b"idle",
        (b"Service", b"Restart") => value == b"always",
        (b"Service", b"UtmpIdentifier") => value == b"%I",
        (b"Service", b"StandardInput" | b"StandardOutput") => value == b"tty",
        (b"Service", b"TTYPath") => value == b"/dev/%I",
        (b"Service", b"TTYReset" | b"TTYVHangup") => value == b"yes",
        (b"Service", b"KillMode") => value == b"process",
        (b"Service", b"IgnoreSIGPIPE") => value == b"no",
        (b"Service", b"SendSIGHUP") => value == b"yes",
        (b"Service", b"ImportCredential") => matches!(
            value,
            b"tty.serial.%I.agetty.*:agetty."
                | b"tty.serial.%I.login.*:login."
                | b"agetty.*"
                | b"login.*"
                | b"shell.*"
        ),
        (b"Install", b"WantedBy") => value == b"getty.target",
        _ => false,
    }
}

fn trim_ascii(mut data: &[u8]) -> &[u8] {
    while data.first().is_some_and(u8::is_ascii_whitespace) {
        data = &data[1..];
    }
    while data.last().is_some_and(u8::is_ascii_whitespace) {
        data = &data[..data.len() - 1];
    }
    data
}
