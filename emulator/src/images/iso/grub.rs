#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootSpec {
    pub kernel_path: String,
    pub initrd_paths: Vec<String>,
    pub bootargs: String,
}

pub fn parse_grub_boot_spec(config: &str, exists: impl Fn(&str) -> bool) -> Option<BootSpec> {
    let mut pending: Option<(String, String)> = None;

    for line in config.lines() {
        let tokens = shell_words(strip_comment(line));
        if tokens.is_empty() {
            continue;
        }

        match tokens[0].as_str() {
            "linux" | "linuxefi" | "linux16" => {
                if tokens.len() < 2 {
                    pending = None;
                    continue;
                }
                let kernel_path = clean_grub_path(&tokens[1]);
                if exists(&kernel_path) {
                    pending = Some((kernel_path, tokens[2..].join(" ")));
                } else {
                    pending = None;
                }
            }
            "initrd" | "initrdefi" | "initrd16" => {
                let Some((kernel_path, bootargs)) = pending.take() else {
                    continue;
                };
                let initrd_paths: Vec<String> = tokens[1..]
                    .iter()
                    .map(|path| clean_grub_path(path))
                    .filter(|path| exists(path))
                    .collect();
                if !initrd_paths.is_empty() {
                    return Some(BootSpec {
                        kernel_path,
                        initrd_paths,
                        bootargs,
                    });
                }
            }
            _ => {}
        }
    }

    None
}

fn strip_comment(line: &str) -> &str {
    line.split_once('#')
        .map(|(before, _)| before)
        .unwrap_or(line)
        .trim()
}

fn clean_grub_path(path: &str) -> String {
    let mut path = path.trim();
    if path.starts_with('(') {
        if let Some(end) = path.find(')') {
            path = &path[end + 1..];
        }
    }
    if path.is_empty() {
        "/".to_string()
    } else if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn shell_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if Some(ch) == quote {
            quote = None;
            continue;
        }
        if quote.is_none() && (ch == '\'' || ch == '"') {
            quote = Some(ch);
            continue;
        }
        if quote.is_none() && ch.is_whitespace() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push(ch);
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_first_existing_linux_initrd_pair() {
        let config = r#"
menuentry 'Install' {
    linux /install.a64/vmlinuz root=/dev/ram0 quiet
    initrd /install.a64/initrd.gz
}
"#;
        let spec = parse_grub_boot_spec(config, |path| {
            matches!(path, "/install.a64/vmlinuz" | "/install.a64/initrd.gz")
        })
        .unwrap();

        assert_eq!(spec.kernel_path, "/install.a64/vmlinuz");
        assert_eq!(spec.initrd_paths, vec!["/install.a64/initrd.gz"]);
        assert_eq!(spec.bootargs, "root=/dev/ram0 quiet");
    }

    #[test]
    fn strips_grub_device_prefix() {
        let config = "linux ($root)/casper/vmlinuz ---\ninitrd ($root)/casper/initrd\n";
        let spec = parse_grub_boot_spec(config, |path| {
            matches!(path, "/casper/vmlinuz" | "/casper/initrd")
        })
        .unwrap();

        assert_eq!(spec.kernel_path, "/casper/vmlinuz");
        assert_eq!(spec.initrd_paths, vec!["/casper/initrd"]);
    }
}
