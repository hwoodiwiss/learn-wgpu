#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub colour: [f32; 4],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

pub const TRIANGLE: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        colour: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        colour: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        colour: [0.0, 0.0, 1.0, 1.0],
    },
];
pub const TRIANGLE_INDICES: &[u16] = &[
    0, 1, 2
];

pub const PENTAGON: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], colour: [0.5, 0.0, 0.5, 1.0] },
    Vertex { position: [-0.49513406, 0.06958647, 0.0], colour: [0.5, 0.0, 0.5, 1.0] },
    Vertex { position: [-0.21918549, -0.44939706, 0.0], colour: [0.5, 0.0, 0.5, 1.0] },
    Vertex { position: [0.35966998, -0.3473291, 0.0], colour: [0.5, 0.0, 0.5, 1.0] },
    Vertex { position: [0.44147372, 0.2347359, 0.0],colour: [0.5, 0.0, 0.5, 1.0] },
];

pub const PENTAGON_INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];
