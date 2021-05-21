use wgpu::IndexFormat;
use wgpu::include_spirv;
use wgpu::util::DeviceExt;
use winit::event::ElementState;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;
use winit::{event::WindowEvent, window::Window};

use crate::vertex::PENTAGON;
use crate::vertex::PENTAGON_INDICES;
use crate::vertex::TRIANGLE_INDICES;
use crate::vertex::{Vertex, TRIANGLE};
fn rgb_to_normalized(r: u8, g: u8, b: u8) -> wgpu::Color {
    // Wish this could be const, but cant do fp arithmatic in const fn
    wgpu::Color {
        r: r as f64 / 255f64,
        g: g as f64 / 255f64,
        b: b as f64 / 255f64,
        a: 1.0,
    }
}

fn create_mesh_buffers(device: &wgpu::Device, vertices: &[Vertex], indices: &[u16]) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let num_indices = indices.len() as u32;

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsage::INDEX,
        });

        (vertex_buffer, index_buffer, num_indices)
    }

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    bg_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffers: Vec<wgpu::Buffer>,
    index_buffers: Vec<wgpu::Buffer>,
    num_indices_list: Vec<u32>,
    mesh_index: usize,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
            })
            .await
            .expect("Could not create adapter instance!");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .expect("Could not get device from adapter!");

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter
                .get_swap_chain_preferred_format(&surface)
                .expect("Could not get swapchain format!"),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // Cornflour blue, because I 'member XNA
        let bg_color = rgb_to_normalized(100, 149, 237);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let num_vertices = u16::max_value();
        let angle = std::f32::consts::PI * 2.0 / num_vertices as f32;
        let challenge_verts = (0..num_vertices)
            .map(|i| {
                let theta = angle * i as f32;
                Vertex {
                    position: [0.5 * theta.cos(), -0.5 * theta.sin(), 0.0],
                    colour: [(1.0 + theta.cos()) / 2.0, (1.0 + theta.sin()) / 2.0, 1.0, 1.0],
                }
            })
            .collect::<Vec<_>>();

        let num_triangles = num_vertices - 2;
        let challenge_indices = (1u16..num_triangles + 1)
            .into_iter()
            .flat_map(|i| vec![i + 1, i, 0])
            .collect::<Vec<_>>();

        let (triangle_vertex_buffer, triangle_index_buffer, triangle_num_indices) = create_mesh_buffers(&device, TRIANGLE, TRIANGLE_INDICES);
        let (pentagon_vertex_buffer, pentagon_index_buffer, pentagon_num_indices) = create_mesh_buffers(&device, PENTAGON, PENTAGON_INDICES);
        let (challenge_vertex_buffer, challenge_index_buffer, challenge_num_indices) = create_mesh_buffers(&device, &challenge_verts, &challenge_indices);
        let vs_module = device.create_shader_module(&include_spirv!("shaders/basic.vert.spv"));
        let fs_module = device.create_shader_module(&include_spirv!("shaders/basic.frag.spv"));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            bg_color,
            render_pipeline,
            vertex_buffers: vec![triangle_vertex_buffer, pentagon_vertex_buffer, challenge_vertex_buffer],
            index_buffers: vec![triangle_index_buffer, pentagon_index_buffer, challenge_index_buffer],
            num_indices_list: vec![triangle_num_indices, pentagon_num_indices, challenge_num_indices],
            mesh_index: 0,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = self.size.width;
        self.sc_desc.height = self.size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Space),
                    ..
                } => {

                    if self.mesh_index < self.vertex_buffers.len() - 1 {
                        self.mesh_index += 1;
                    } else {
                        self.mesh_index = 0;
                    }

                    true
                }
                _ => false

            }
            _ => false
        }
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Frame render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.bg_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffers[self.mesh_index].slice(..));
            render_pass.set_index_buffer(self.index_buffers[self.mesh_index].slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices_list[self.mesh_index], 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
