use crate::bus::messages::ArcisVertex;
use bytemuck::{Pod, Zeroable};
use eframe::egui_wgpu;
use eframe::wgpu;
use eframe::wgpu::util::DeviceExt;
use std::sync::Arc;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CameraUniform {
    yaw: f32,
    pitch: f32,
    zoom: f32,
    opacity: f32,
}

const SHADER_WGSL: &str = r#"
struct CameraUniform {
    yaw: f32,
    pitch: f32,
    zoom: f32,
    opacity: f32,
};

@group(0) @binding(0)
var<uniform> cam: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let cy = cos(cam.yaw);
    let sy = sin(cam.yaw);
    let cx = cos(cam.pitch);
    let sx = sin(cam.pitch);

    // Rotação orbital em torno do eixo Y (yaw) e X (pitch)
    let x1 = model.position.x * cy - model.position.z * sy;
    let z1 = model.position.x * sy + model.position.z * cy;
    let y2 = model.position.y * cx - z1 * sx;
    let z2 = model.position.y * sx + z1 * cx;

    out.clip_position = vec4<f32>(x1, y2, z2 * 0.5 + 0.5, cam.zoom);
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, cam.opacity);
}
"#;

pub struct ArcisRenderPipeline {
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

impl ArcisRenderPipeline {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Arcis Core Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_WGSL.into()),
        });

        let camera_uniform = CameraUniform {
            yaw: 0.6,
            pitch: 0.7,
            zoom: 1.8,
            opacity: 1.0,
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Arcis Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Arcis Camera BindGroupLayout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Arcis Camera BindGroup"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Arcis Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Arcis Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<ArcisVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            camera_buffer,
            camera_bind_group,
        }
    }
}

pub struct ArcisGpuData {
    pub data_ptr: usize,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

pub struct WgpuCallback {
    pub vertices: Arc<Vec<ArcisVertex>>,
    pub indices: Arc<Vec<u32>>,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub opacity: f32,
}

impl egui_wgpu::CallbackTrait for WgpuCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        // Atualiza a câmera e opacidade a cada frame
        if let Some(pipeline) = resources.get::<ArcisRenderPipeline>() {
            let uniform = CameraUniform {
                yaw: self.yaw,
                pitch: self.pitch,
                zoom: self.zoom,
                opacity: self.opacity,
            };
            queue.write_buffer(&pipeline.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
        }

        if self.vertices.is_empty() || self.indices.is_empty() {
            return Vec::new();
        }

        let current_ptr = Arc::as_ptr(&self.vertices) as usize;
        let mut needs_upload = true;
        if let Some(gpu_data) = resources.get::<ArcisGpuData>() {
            if gpu_data.data_ptr == current_ptr {
                needs_upload = false;
            }
        }

        if needs_upload {
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Arcis Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Arcis Index Buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            resources.insert(ArcisGpuData {
                data_ptr: current_ptr,
                vertex_buffer,
                index_buffer,
                num_indices: self.indices.len() as u32,
            });
        }

        Vec::new()
    }

    fn paint(
        &self,
        _info: eframe::egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {
        if let (Some(pipeline), Some(gpu_data)) = (
            callback_resources.get::<ArcisRenderPipeline>(),
            callback_resources.get::<ArcisGpuData>(),
        ) {
            render_pass.set_pipeline(&pipeline.pipeline);
            render_pass.set_bind_group(0, &pipeline.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, gpu_data.vertex_buffer.slice(..));
            render_pass.set_index_buffer(gpu_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..gpu_data.num_indices, 0, 0..1);
        }
    }
}