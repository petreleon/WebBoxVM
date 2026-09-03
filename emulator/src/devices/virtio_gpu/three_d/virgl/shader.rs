mod parse;

pub(in crate::devices::virtio_gpu::three_d::virgl) use parse::parse;

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
    VertexTextured,
    FragmentSolid([u32; 4]),
    FragmentTextured,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Shader {
    pub kind: ShaderKind,
    pub program: ShaderProgram,
}
