mod parse;

pub(in crate::devices::virtio_gpu::three_d::virgl) use parse::parse;

pub(super) const MAX_SHADER_TEXT_BYTES: usize = 4 * 1024;
pub(super) const MAX_TGSI_TOKENS: u32 = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::devices::virtio_gpu) enum ShaderKind {
    Vertex,
    Fragment,
}

impl ShaderKind {
    const VERTEX_PIPE_TYPE: u32 = 0;
    const FRAGMENT_PIPE_TYPE: u32 = 1;

    pub(in crate::devices::virtio_gpu) fn from_pipe_type(value: u32) -> Option<Self> {
        match value {
            Self::VERTEX_PIPE_TYPE => Some(Self::Vertex),
            Self::FRAGMENT_PIPE_TYPE => Some(Self::Fragment),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::devices::virtio_gpu) enum ShaderProgram {
    VertexPassthrough,
    VertexGeneric,
    FragmentSolid([u32; 4]),
    FragmentVertexColor,
    FragmentTextured,
    FragmentTexturedMultiply,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Shader {
    pub kind: ShaderKind,
    pub program: ShaderProgram,
}

impl Shader {
    pub(super) const fn tgsi_token_count(self) -> u32 {
        match self.program {
            ShaderProgram::VertexPassthrough => 11,
            ShaderProgram::VertexGeneric => 17,
            ShaderProgram::FragmentSolid(_) => 14,
            ShaderProgram::FragmentVertexColor => 11,
            ShaderProgram::FragmentTextured => 25,
            ShaderProgram::FragmentTexturedMultiply => 31,
        }
    }
}
