use super::super::VirglContext;
use super::decode::constant::Command;

pub(super) fn apply(context: &mut VirglContext, command: Command) {
    match command {
        Command::SetVertex(values) => context.set_vertex_constants(values),
        Command::SetFragment(values) => context.set_fragment_constants(values),
    }
}
