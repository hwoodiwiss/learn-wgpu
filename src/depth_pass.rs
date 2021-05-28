
use wgpu::util::DeviceExt;

use crate::{texture::{Texture}, vertex::{Vertex, PLANE, PLANE_INDICES}};

pub struct DepthPass {
	pub texture: Texture,
	bind_group_layout: wgpu::BindGroupLayout,
	bind_group: wgpu::BindGroup,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	num_depth_indices: u32,
	render_pipeline: wgpu::RenderPipeline,
}

impl DepthPass {
    pub fn new(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
     	let texture = Texture::create_depth_texture(device, sc_desc, "depth_texture");

     	let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("depth_pass.bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: true,
                    },
                    count: None,
                }
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("depth_pass.bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                }
            ],
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("depth_pass.vertices"),
            contents: bytemuck::cast_slice(PLANE),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("depth_pass.vertices"),
            contents: bytemuck::cast_slice(PLANE_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: Some("depth_pass.pipeline_layout"),
            bind_group_layouts: &[
            	&bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("depth_pass.shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/depth_texture.wgsl").into()),
            flags: wgpu::ShaderFlags::all(),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: Some("depth_pass.pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[Vertex::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                clamp_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
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
        });

        Self {
        	texture,
        	bind_group_layout,
        	bind_group,
        	vertex_buffer,
        	index_buffer,
        	num_depth_indices: PLANE_INDICES.len() as u32,
        	render_pipeline,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) {
    	self.texture = Texture::create_depth_texture(device, sc_desc, "depth_texture");
    	self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("depth_pass.bind_group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.texture.sampler),
                }
            ],
        });
    }

    pub fn render(&self, frame: &wgpu::SwapChainTexture, encoder: &mut wgpu::CommandEncoder) {
    	let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    	    label: Some("depth_pass"),
    	    color_attachments: &[
    	    	wgpu::RenderPassColorAttachment {
    	        	view: &frame.view,
	    	        resolve_target: None,
    		        ops: wgpu::Operations {
    		            load: wgpu::LoadOp::Load,
    		            store: true,
    		        },
    	    	}
    	    ],
    	    depth_stencil_attachment: None,
    	});

    	render_pass.set_pipeline(&self.render_pipeline);

    	render_pass.set_bind_group(0, &self.bind_group, &[]);

    	render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    	render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

    	render_pass.draw_indexed(0..self.num_depth_indices, 0, 0..1);
    }

}