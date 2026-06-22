use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    /// Get UART output.
    pub fn uart_output(&self) -> String {
        if let Some(ref boot) = self.boot {
            boot.uart_output()
        } else {
            self.machine.bus.uart.output_string()
        }
    }

    /// Get UART output length in bytes.
    pub fn uart_output_len(&self) -> usize {
        if let Some(ref boot) = self.boot {
            boot.uart_output_len()
        } else {
            self.machine.bus.uart.output.len()
        }
    }

    /// Get UART output since a byte offset.
    pub fn uart_output_since(&self, offset: usize) -> String {
        if let Some(ref boot) = self.boot {
            boot.uart_output_since(offset)
        } else {
            let output = &self.machine.bus.uart.output;
            String::from_utf8_lossy(&output[offset.min(output.len())..]).to_string()
        }
    }

    /// Send text to the guest UART receive path.
    pub fn send_uart_input(&mut self, input: &str) {
        if let Some(ref mut boot) = self.boot {
            boot.feed_uart_input(input);
        } else {
            self.machine.feed_uart_input(input);
        }
    }

    /// Send raw bytes to the guest UART receive path.
    pub fn send_uart_bytes(&mut self, input: Vec<u8>) {
        if let Some(ref mut boot) = self.boot {
            boot.feed_uart_bytes(&input);
        } else if !input.is_empty() {
            self.machine.feed_uart_bytes(&input);
        }
    }
}
