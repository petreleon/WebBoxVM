//! Bounded native guest command execution; no GPU completion is synthesized.

#[path = "guest_command/mod.rs"]
mod guest_command;

fn main() -> Result<(), String> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments == ["--help"] {
        println!("{}", guest_command::Options::usage());
        return Ok(());
    }
    let options = guest_command::Options::parse(arguments.into_iter())?;
    guest_command::run(options)
}
