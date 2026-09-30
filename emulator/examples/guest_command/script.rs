use super::budget::INPUT_BYTES;

pub enum Action {
    Handshake,
    Command(usize),
    Finish,
}

pub struct Scripts {
    pub handshake: String,
    pub commands: Vec<String>,
    pub finish: String,
}

impl Scripts {
    pub fn new(token: &str, commands: &[String]) -> Result<Self, String> {
        if commands.is_empty() || commands.len() > 16 {
            return Err("invalid guest command count".into());
        }
        if token.is_empty() || token.len() > 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("protocol token must be bounded hexadecimal text".into());
        }
        let handshake =
            format!("stty -echo; r=$?; printf '\\nWEBBOXVM_GUEST_%s:%s:%s\\n' READY {token} \"$r\"\r");
        let finish = format!("printf '\\nWEBBOXVM_GUEST_%s:%s\\n' DONE {token}\r");
        let mut scripts = Vec::new();
        for (index, command) in commands.iter().enumerate() {
            if command.trim().is_empty() || command.contains('\0') {
                return Err("guest command files must be nonempty UTF-8 without NUL".into());
            }
            let path = format!("/tmp/webboxvm-guest-{token}-{index}.sh");
            let delimiter = format!("WEBBOXVM_{token}_{index}_EOF");
            scripts.push(format!(
                "base64 -d >{path} <<'{delimiter}'\r{}{delimiter}\rr=$?; \
                 printf '\\nWEBBOXVM_GUEST_%s:%s:%s\\n' BEGIN {token} {index}; \
                 if test \"$r\" -eq 0; then sh -e {path} </dev/null; r=$?; fi; \
                 printf '\\nWEBBOXVM_GUEST_%s:%s:%s:%s\\n' STATUS {token} {index} \"$r\"\r",
                base64_lines(command.as_bytes())
            ));
        }
        let size = scripts.iter().map(String::len).sum::<usize>() + handshake.len() + finish.len();
        if size > INPUT_BYTES {
            return Err(format!("guest input exceeds {INPUT_BYTES} bytes"));
        }
        Ok(Self { handshake, commands: scripts, finish })
    }

    pub fn input(&self, action: Action) -> &str {
        match action {
            Action::Handshake => &self.handshake,
            Action::Command(index) => &self.commands[index],
            Action::Finish => &self.finish,
        }
    }
}

fn base64_lines(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let bits = u32::from(chunk[0]) << 16
            | u32::from(*chunk.get(1).unwrap_or(&0)) << 8
            | u32::from(*chunk.get(2).unwrap_or(&0));
        encoded.push(TABLE[((bits >> 18) & 63) as usize] as char);
        encoded.push(TABLE[((bits >> 12) & 63) as usize] as char);
        encoded.push(if chunk.len() > 1 { TABLE[((bits >> 6) & 63) as usize] as char } else { '=' });
        encoded.push(if chunk.len() > 2 { TABLE[(bits & 63) as usize] as char } else { '=' });
    }
    encoded.as_bytes().chunks(76).map(|line| format!("{}\r", std::str::from_utf8(line).unwrap())).collect()
}

#[cfg(test)]
#[path = "tests/script.rs"]
mod tests;
