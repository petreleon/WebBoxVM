pub fn merge_bootargs(base: &str, extra: &str) -> String {
    let extra = normalize_extra_bootargs(extra);
    if extra.is_empty() {
        base.to_string()
    } else if base.is_empty() {
        extra
    } else {
        format!("{base} {extra}")
    }
}

fn normalize_extra_bootargs(extra: &str) -> String {
    extra.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_bootargs_ignores_empty_extra_args() {
        assert_eq!(merge_bootargs("root=/dev/vdb3", " \n\t"), "root=/dev/vdb3");
    }

    #[test]
    fn merge_bootargs_normalizes_extra_arg_spacing() {
        assert_eq!(
            merge_bootargs("root=/dev/vdb3", "  ftrace_filter=__x64_sys_close   quiet "),
            "root=/dev/vdb3 ftrace_filter=__x64_sys_close quiet"
        );
    }
}
