use super::util;
use emulator::host::native::NativeVm;
use std::env;

pub(super) struct BootTestInput {
    shell_script: Option<String>,
    shell_sent: bool,
    prompt_marker: String,
    prompt_scan: usize,
    prompt_script: Vec<String>,
    prompt_index: usize,
}

impl BootTestInput {
    pub(super) fn from_env() -> Self {
        Self {
            shell_script: shell_script(),
            shell_sent: false,
            prompt_marker: prompt_marker(),
            prompt_scan: 0,
            prompt_script: prompt_script(),
            prompt_index: 0,
        }
    }

    pub(super) fn disables_uart_stop(&self) -> bool {
        self.shell_script.is_some() || !self.prompt_script.is_empty()
    }

    pub(super) fn maybe_feed(&mut self, ctx: &mut NativeVm, uart: &str) {
        self.maybe_feed_shell(ctx, uart);
        self.maybe_feed_prompt(ctx, uart);
    }

    fn maybe_feed_shell(&mut self, ctx: &mut NativeVm, uart: &str) {
        if self.shell_sent || !uart.contains("webboxvm# ") {
            return;
        }
        let Some(script) = self.shell_script.as_deref() else {
            return;
        };
        ctx.feed_uart_input(script);
        self.shell_sent = true;
        println!(
            "Fed {} bytes of UART input from BOOT_TEST_COMMANDS",
            script.len()
        );
    }

    fn maybe_feed_prompt(&mut self, ctx: &mut NativeVm, uart: &str) {
        if self.prompt_index >= self.prompt_script.len() {
            return;
        }
        let start = self.prompt_scan.min(uart.len());
        let Some(pos) = uart[start..].find(&self.prompt_marker) else {
            return;
        };
        self.prompt_scan = start + pos + self.prompt_marker.len();
        let script = &self.prompt_script[self.prompt_index];
        ctx.feed_uart_input(script);
        self.prompt_index += 1;
        println!(
            "Fed prompt reply {}/{} from BOOT_TEST_PROMPT_SCRIPT",
            self.prompt_index,
            self.prompt_script.len()
        );
    }
}

fn shell_script() -> Option<String> {
    env::var("BOOT_TEST_COMMANDS")
        .ok()
        .filter(|script| !script.is_empty())
        .map(|script| util::normalize_boot_test_commands(&script))
}

fn prompt_marker() -> String {
    env::var("BOOT_TEST_PROMPT_TEXT").unwrap_or_else(|_| "Prompt:".to_string())
}

fn prompt_script() -> Vec<String> {
    env::var("BOOT_TEST_PROMPT_SCRIPT")
        .ok()
        .filter(|script| !script.is_empty())
        .map(|script| {
            script
                .split(';')
                .map(util::normalize_boot_test_commands)
                .collect()
        })
        .unwrap_or_default()
}
