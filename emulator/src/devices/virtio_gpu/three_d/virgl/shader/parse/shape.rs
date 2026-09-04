mod operation;
pub(super) use operation::Operation;

#[derive(Clone, Copy)]
struct Declaration<'a> {
    register: &'a str,
    semantic: Option<&'a str>,
}

/// A fail-closed TGSI text subset.
///
/// Invariant: every instruction is one of the five variants above and exactly
/// one `END` terminates the program. Declarations must not overlap; matchers
/// may ignore non-overlapping declarations that no supported operation reads.
/// The O(d²) overlap check runs only at bounded shader creation, never per draw.
pub(super) struct Shape<'a> {
    declarations: Vec<Declaration<'a>>,
    immediates: Vec<&'a str>,
    operations: Vec<Operation<'a>>,
}

pub(super) fn parse<'a>(lines: &[&'a str], stage: &str) -> Option<Shape<'a>> {
    if lines.first().copied() != Some(stage) {
        return None;
    }
    let mut declarations: Vec<Declaration<'a>> = Vec::new();
    let mut immediates: Vec<&str> = Vec::new();
    let mut operations = Vec::new();
    let mut ended = false;
    for line in lines.iter().skip(1).copied() {
        if ended {
            return None;
        }
        if let Some(declaration) = declaration(line) {
            if declarations.iter().any(|other| registers_overlap(other.register, declaration.register)) {
                return None;
            }
            declarations.push(declaration);
        } else if line.starts_with("IMM[") {
            let register = immediate_register(line)?;
            if immediates.iter().any(|other| immediate_register(other) == Some(register)) {
                return None;
            }
            immediates.push(line);
        } else {
            let operation = operation::parse(line)?;
            ended = operation == Operation::End;
            operations.push(operation);
        }
    }
    ended.then_some(Shape { declarations, immediates, operations })
}

impl Shape<'_> {
    pub(super) fn has_register(&self, wanted: &str) -> bool {
        self.declarations.iter().any(|declaration| register_matches(declaration.register, wanted))
    }

    pub(super) fn has_semantic(&self, wanted: &str, semantic: &str) -> bool {
        self.declarations.iter().any(|declaration| {
            register_matches(declaration.register, wanted) && declaration.semantic == Some(semantic)
        })
    }

    pub(super) fn has_optional_semantic(&self, wanted: &str, semantic: &str) -> bool {
        self.declarations.iter().any(|declaration| {
            register_matches(declaration.register, wanted)
                && declaration.semantic.is_none_or(|found| found == semantic)
        })
    }

    pub(super) fn immediate(&self, wanted: &str) -> Option<&str> {
        let mut matches = self.immediates.iter().copied().filter(|line| immediate_register(line) == Some(wanted));
        let found = matches.next()?;
        matches.next().is_none().then_some(found)
    }

    pub(super) fn operations(&self) -> &[Operation<'_>] {
        &self.operations
    }
}

pub(super) fn same(left: &str, right: &str) -> bool {
    strip_vector(left) == strip_vector(right)
}

pub(super) fn component_matches(actual: &str, register: &str, component: char) -> bool {
    actual
        .strip_suffix(&format!(".{component}"))
        .is_some_and(|base| same(base, register))
}

fn declaration(line: &str) -> Option<Declaration<'_>> {
    let mut parts = line.strip_prefix("DCL ")?.split(',').map(str::trim);
    Some(Declaration { register: parts.next()?.trim(), semantic: parts.next().filter(|part| !part.is_empty()) })
}

fn immediate_register(line: &str) -> Option<&str> {
    line.split_ascii_whitespace().next().filter(|register| register.starts_with("IMM["))
}

fn register_matches(actual: &str, wanted: &str) -> bool {
    if actual == wanted {
        return true;
    }
    let Some((actual_kind, actual_start, actual_end)) = indexed(actual) else { return false; };
    let Some((wanted_kind, wanted_start, wanted_end)) = indexed(wanted) else { return false; };
    actual_kind == wanted_kind && wanted_start == wanted_end && actual_start <= wanted_start && wanted_end <= actual_end
}

fn registers_overlap(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let Some((left_kind, left_start, left_end)) = indexed(left) else { return false; };
    let Some((right_kind, right_start, right_end)) = indexed(right) else { return false; };
    left_kind == right_kind && left_start <= right_end && right_start <= left_end
}

fn indexed(register: &str) -> Option<(&str, u32, u32)> {
    let (kind, indexes) = register.split_once('[')?;
    let indexes = indexes.strip_suffix(']')?;
    let (start, end) = match indexes.split_once("..") {
        Some((start, end)) => (start.parse().ok()?, end.parse().ok()?),
        None => { let index = indexes.parse().ok()?; (index, index) }
    };
    (start <= end).then_some((kind, start, end))
}

fn strip_vector(register: &str) -> &str {
    register.strip_suffix(".xyzw").unwrap_or(register)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::ShaderProgram;

    #[test]
    fn reordered_declarations_keep_supported_program_semantics() {
        let vertex = ["VERT", "DCL OUT[0], POSITION", "DCL IN[0], POSITION", "MOV OUT[0].xyzw, IN[0].xyzw", "END"];
        assert_eq!(super::super::vertex::parse(&vertex), Some(ShaderProgram::VertexPassthrough));
        let fragment = ["FRAG", "DCL OUT[0], COLOR", "DCL TEMP[0]", "DCL SVIEW[0], 2D, FLOAT", "DCL IN[0], GENERIC[0], LINEAR", "DCL SAMP[0]", "TEX TEMP[0].xyzw, IN[0].xyzw, SAMP[0], 2D", "MOV OUT[0].xyzw, TEMP[0].xyzw", "END"];
        assert_eq!(super::super::fragment::parse(&fragment), Some(ShaderProgram::FragmentTextured));
    }

    #[test]
    fn unknown_operations_are_rejected_before_classification() {
        let lines = ["FRAG", "DCL OUT[0], COLOR", "DP3 OUT[0], IN[0], IN[1]", "END"];
        assert!(parse(&lines, "FRAG").is_none());
    }

    #[test]
    fn overlapping_declarations_are_rejected() {
        let lines = ["FRAG", "DCL OUT[0], COLOR", "DCL OUT[0], POSITION", "MOV OUT[0], IMM[0]", "END"];
        assert!(parse(&lines, "FRAG").is_none());
    }
}
