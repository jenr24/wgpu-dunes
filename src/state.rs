use std::iter;

use wgpu::{SurfaceTexture, Instance, Surface, Adapter, Device, Queue, RenderPipeline, ShaderModule, SurfaceConfiguration, ComputePipeline};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder}, dpi::PhysicalSize,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub enum Error {
    AdapterRequestError,
    DeviceRequestError(wgpu::RequestDeviceError)
}

impl From<wgpu::RequestDeviceError> for Error {
    fn from(err: wgpu::RequestDeviceError) ->  Self {
        Error::DeviceRequestError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
}

async fn get_adapter(instance: &Instance, surface: &Surface) -> Option<Adapter> {
    instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
}

async fn get_device_and_queue(adapter: &Adapter) -> Result<(Device, Queue)> {
    adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            ).await.map_err(|e| Error::DeviceRequestError(e))
}

fn create_compute_pipeline(name: &str, device: &Device, shader: &ShaderModule) -> ComputePipeline {
    let mut name_new = String::new();
    name_new.push_str(name);
    name_new.push_str("_layout");

    let compute_pipeline_layout = 
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&name_new),
            bind_group_layouts: &[],
            push_constant_ranges: &[]
        }
    );

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(name),
        layout: Some(&compute_pipeline_layout),
        entry_point: "main",
        module: shader,
    })
}

fn create_render_pipeline(device: &Device, shader: &ShaderModule, config: &SurfaceConfiguration) -> RenderPipeline {
    let render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

    use crate::vertex::Vertex;

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main", // 1.
            buffers: &[Vertex::desc()], // 2.
        },
        fragment: Some(wgpu::FragmentState { // 3.
            module: &shader,
            entry_point: "fs_main",
            targets: &[wgpu::ColorTargetState { // 4.
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList, // 1.
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // 2.
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None, // 1.
        multisample: wgpu::MultisampleState {
            count: 1, // 2.
            mask: !0, // 3.
            alpha_to_coverage_enabled: false, // 4.
        },
        multiview: None, // 5.
    })
}

fn initialize_application_state(
    size: PhysicalSize<u32>, 
    surface: Surface,
    config: SurfaceConfiguration, 
    device: Device, 
    queue: Queue
) -> Result<State> {
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let render_pipeline = create_render_pipeline(&device, &shader, &config);

    use wgpu::util::DeviceExt;
    use crate::vertex;
    let vertex_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertex::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        }
    );

    let num_vertices = vertex::VERTICES.len() as u32;


    Ok(State {
        surface,
        device,
        queue,
        config,
        size,
        render_pipeline,
        vertex_buffer,
        num_vertices
    })
}

impl State {
    pub async fn new(window: &Window) -> Result<Self> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = get_adapter(&instance, &surface).await.ok_or(Error::AdapterRequestError)?;
        let (device, queue) = get_device_and_queue(&adapter).await?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        initialize_application_state(size, surface, config, device, queue)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    #[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        let output: SurfaceTexture = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        // This is what [[location(0)]] in the fragment shader targets
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(
                                    wgpu::Color {
                                        r: 0.1,
                                        g: 0.2,
                                        b: 0.3,
                                        a: 1.0,
                                    }
                                ),
                                store: true,
                            }
                        }
                    ],
                    depth_stencil_attachment: None,
                }
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}