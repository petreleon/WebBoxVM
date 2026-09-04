use super::super::VirglContext;
use super::decode::constant::Command;

pub(super) fn apply(context: &mut VirglContext, command: Command) {
    match command {
        Command::SetFragment(values) => context.set_fragment_constants(values),
    }
}
