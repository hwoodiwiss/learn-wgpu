use std::{ops::Range, path::Path};

use anyhow::*;

use crate::vertex::Vertex;
use crate::texture::Texture;

use wgpu::util::DeviceExt;


#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
	position: [f32; 3],
	tex_coords: [f32; 2],
	normal: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                }
            ],
        }
    }
}

pub struct Material {
	pub name: String,
	pub diffuse_texture: Texture,
    pub bind_group: wgpu::BindGroup,
}


pub struct Mesh {
	pub name: String,
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: wgpu::Buffer,
	pub num_elements: u32,
	pub material: usize,
}

pub struct Model {
	pub meshes: Vec<Mesh>,
	pub materials: Vec<Material>,
}

impl Model {
	pub fn load<P: AsRef<Path>>(device: &wgpu::Device, queue: &wgpu::Queue, layout: &wgpu::BindGroupLayout, path: P) -> Result<Self> {
		let (obj_models, obj_materials) = tobj::load_obj(path.as_ref(), &tobj::LoadOptions {
		    triangulate: true,
		    single_index: true,
		    ..Default::default()
        })?;
        
        let obj_materials = obj_materials?;

        let containing_folder = path.as_ref().parent().context("Directory has no parent")?;

        let mut materials = Vec::new();

        for mat in obj_materials {
            let diffuse_path = mat.diffuse_texture;
            let diffuse_texture = Texture::load(device, queue, containing_folder.join(diffuse_path))?;
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
                label: None,
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
            });

            materials.push(Material {
                name: mat.name,
                diffuse_texture,
                bind_group,
            })
        };

        let mut meshes = Vec::new();

        for model in obj_models {
            let mut vertices = Vec::with_capacity(model.mesh.positions.len() / 3);
            for i in 0..model.mesh.positions.len() / 3 {
                vertices.push(ModelVertex {
                    position: [
                        model.mesh.positions[i * 3],
                        model.mesh.positions[i * 3 + 1],
                        model.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [model.mesh.texcoords[i * 2], model.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        model.mesh.normals[i * 3],
                        model.mesh.normals[i * 3 + 1],
                        model.mesh.normals[i * 3 + 2],
                    ]
                });
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", path.as_ref())),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsage::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", path.as_ref())),
                contents: bytemuck::cast_slice(&model.mesh.indices),
                usage: wgpu::BufferUsage::INDEX,
            });

            meshes.push(Mesh {
                name: model.name,
                vertex_buffer,
                index_buffer,
                num_elements: model.mesh.indices.len() as u32,
                material: model.mesh.material_id.unwrap_or(0),
            });
        }

        Ok(Self{
            meshes,
            materials,
        })
	}
}

pub trait DrawModel<'a, 'b>
where 'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh, material: &'b Material, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup);
    fn draw_mesh_instanced(&mut self, mesh: &'b Mesh, material: &'b Material, instances: Range<u32>, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup);

    fn draw_model(&mut self, model: &'b Model, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup);
    fn draw_model_instanced(&mut self, model: &'b Model, instances: Range<u32>, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup);
}

impl<'a, 'b> DrawModel<'a, 'b> for wgpu::RenderPass<'a>
where 'b: 'a
{
    fn draw_mesh(&mut self, mesh: &'b Mesh, material: &'b Material, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup) {
        self.draw_mesh_instanced(mesh, material, 0..1, uniforms, light);
    }

    fn draw_mesh_instanced(&mut self, mesh: &'b Mesh, material: &'b Material, instances: Range<u32>, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, &uniforms, &[]);
        self.set_bind_group(2, &light, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(&mut self, model: &'b Model, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup) {
        self.draw_model_instanced(model, 0..1, uniforms, light);
    }

    fn draw_model_instanced(&mut self, model: &'b Model, instances: Range<u32>, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instances.clone(), uniforms, light);
        }
    }
}

pub trait DrawLight<'a, 'b>
where 'b: 'a,
{
    fn draw_light_mesh(&mut self, mesh: &'b Mesh, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup);
    fn draw_light_mesh_instanced(&mut self, mesh: &'b Mesh, instances: Range<u32>, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup);

    fn draw_light_model(&mut self, model: &'b Model, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup);
    fn draw_light_model_instanced(&mut self, model: &'b Model, instances: Range<u32>, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup);
}

impl<'a, 'b> DrawLight<'a, 'b> for wgpu::RenderPass<'a>
where 'b: 'a,
 {
    fn draw_light_mesh(&mut self, mesh: &'b Mesh, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup) {
        self.draw_light_mesh_instanced(mesh, 0..1, uniforms, light);
    }

    fn draw_light_mesh_instanced(&mut self, mesh: &'b Mesh, instances: Range<u32>, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, uniforms, &[]);
        self.set_bind_group(1, light, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_light_model(&mut self, model: &'b Model, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup) {
        self.draw_light_model_instanced(model, 0..1, uniforms, light);
    }

    fn draw_light_model_instanced(&mut self, model: &'b Model, instances: Range<u32>, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup) {
        for mesh in &model.meshes {
            self.draw_light_mesh_instanced(mesh, instances.clone(), uniforms, light);
        }
    }
}