#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::devices::virtio_gpu::three_d::virgl::shader::parse) enum Operation<'a> {
    Mov(&'a str, &'a str),
    Add(&'a str, &'a str, &'a str),
    Mul(&'a str, &'a str, &'a str),
    Dp4(&'a str, &'a str, &'a str),
    Tex(&'a str, &'a str, &'a str, &'a str),
    End,
}

pub(super) fn parse(line: &str) -> Option<Operation<'_>> {
    let mut words = line
        .split(|character: char| character.is_ascii_whitespace() || character == ',')
        .filter(|word| !word.is_empty());
    let opcode = words.next()?;
    let arguments: Vec<_> = words.collect();
    match (opcode, arguments.as_slice()) {
        ("MOV", [destination, source]) => Some(Operation::Mov(destination, source)),
        ("ADD", [destination, left, right]) => Some(Operation::Add(destination, left, right)),
        ("MUL", [destination, left, right]) => Some(Operation::Mul(destination, left, right)),
        ("DP4", [destination, left, right]) => Some(Operation::Dp4(destination, left, right)),
        ("TEX", [destination, coordinates, sampler, target]) => Some(Operation::Tex(destination, coordinates, sampler, target)),
        ("END", []) => Some(Operation::End),
        _ => None,
    }
}
