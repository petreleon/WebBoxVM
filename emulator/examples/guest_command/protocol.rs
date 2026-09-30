use super::script::Action;

#[derive(Debug, PartialEq, Eq)]
enum Phase {
    Boot,
    Ready,
    Begin(usize),
    Command(usize),
    Done,
    Finished,
}

pub struct Protocol {
    token: String,
    ready: String,
    count: usize,
    required: Vec<String>,
    phase: Phase,
    action: Option<Action>,
    pending: Vec<u8>,
    command_output: String,
    completed_output: String,
    bytes: usize,
    cap: usize,
}

impl Protocol {
    pub fn new(token: String, ready: String, count: usize, required: Vec<String>, cap: usize) -> Self {
        Self {
            token,
            ready,
            count,
            required,
            phase: Phase::Boot,
            action: None,
            pending: Vec::new(),
            command_output: String::new(),
            completed_output: String::new(),
            bytes: 0,
            cap,
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.bytes = self.bytes.checked_add(bytes.len()).ok_or("UART byte count overflow")?;
        if self.bytes > self.cap {
            return Err(format!("UART output exceeds {} bytes", self.cap));
        }
        for &byte in bytes {
            if byte == b'\n' {
                let mut line = std::mem::take(&mut self.pending);
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                let line = std::str::from_utf8(&line).map_err(|_| "guest UART is not UTF-8")?;
                self.line(line)?;
            } else {
                self.pending.push(byte);
            }
        }
        if fatal(&self.pending) {
            return Err("guest kernel failure in partial UART line".into());
        }
        if let Err(error) = std::str::from_utf8(&self.pending) {
            if error.error_len().is_some() || self.finished() {
                return Err("guest UART is not UTF-8".into());
            }
        }
        if self.finished() && reserved(&String::from_utf8_lossy(&self.pending)) {
            return Err("trailing partial guest protocol line".into());
        }
        Ok(())
    }

    fn line(&mut self, line: &str) -> Result<(), String> {
        if fatal(line.as_bytes()) {
            return Err(format!("guest kernel failure: {line}"));
        }
        if self.phase == Phase::Boot && line == self.ready {
            self.phase = Phase::Ready;
            self.action = Some(Action::Handshake);
            return Ok(());
        }
        if line.starts_with("WEBBOXVM_GUEST_") {
            return self.marker(line);
        }
        if reserved(line) {
            return Err(format!("decorated guest protocol line: {line}"));
        }
        if matches!(self.phase, Phase::Command(_)) {
            self.command_output.push_str(line);
            self.command_output.push('\n');
        }
        Ok(())
    }

    fn marker(&mut self, line: &str) -> Result<(), String> {
        if self.action.is_some() {
            return Err("guest replied before the next host action".into());
        }
        let parts: Vec<_> = line.split(':').collect();
        if parts.get(1) != Some(&self.token.as_str()) {
            return Err(format!("unexpected guest protocol line: {line}"));
        }
        match (&self.phase, parts.as_slice()) {
            (Phase::Ready, ["WEBBOXVM_GUEST_READY", _, "0"]) => {
                self.phase = Phase::Begin(0);
                self.action = Some(Action::Command(0));
            }
            (Phase::Ready, ["WEBBOXVM_GUEST_READY", _, status]) => {
                return Err(format!("guest could not disable terminal echo: status {status}"));
            }
            (Phase::Begin(index), ["WEBBOXVM_GUEST_BEGIN", _, ordinal]) if *ordinal == index.to_string() => {
                self.phase = Phase::Command(*index);
            }
            (Phase::Command(index), ["WEBBOXVM_GUEST_STATUS", _, ordinal, status])
                if *ordinal == index.to_string() =>
            {
                if *status != "0" {
                    return Err(format!("guest command {index} failed with status {status}"));
                }
                self.completed_output.push_str(&self.command_output);
                self.command_output.clear();
                let next = index + 1;
                if next < self.count {
                    self.phase = Phase::Begin(next);
                    self.action = Some(Action::Command(next));
                } else {
                    self.phase = Phase::Done;
                    self.action = Some(Action::Finish);
                }
            }
            (Phase::Done, ["WEBBOXVM_GUEST_DONE", _]) => {
                for expected in &self.required {
                    if !self.completed_output.split_terminator('\n').any(|line| line == expected) {
                        return Err(format!("required guest command output missing: {expected:?}"));
                    }
                }
                self.phase = Phase::Finished;
            }
            _ => return Err(format!("malformed, duplicate or out-of-order guest protocol line: {line}")),
        }
        Ok(())
    }

    pub fn take_action(&mut self) -> Option<Action> {
        self.action.take()
    }
    pub fn finished(&self) -> bool {
        self.phase == Phase::Finished
    }
    pub fn phase(&self) -> String {
        format!("{:?}", self.phase)
    }
}

fn fatal(bytes: &[u8]) -> bool {
    [b"Kernel panic".as_slice(), b"Oops:", b"BUG:", b"Unable to handle kernel"]
        .iter()
        .any(|pattern| bytes.windows(pattern.len()).any(|window| window == *pattern))
}

fn reserved(line: &str) -> bool {
    ["READY", "BEGIN", "STATUS", "DONE"]
        .iter()
        .any(|suffix| line.contains(&format!("WEBBOXVM_GUEST_{suffix}")))
}

#[cfg(test)]
#[path = "tests/protocol.rs"]
mod tests;
