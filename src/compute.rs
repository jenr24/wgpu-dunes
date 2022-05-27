
use bytemuck::bytes_of;
use rand::prelude::StdRng;
use rand::{thread_rng, Rng};
use wasm_bindgen::convert::{RefFromWasmAbi, WasmSlice};
use wasm_bindgen::prelude::*;
use wgpu::{
    include_wgsl, Adapter, BindGroup, Buffer, ComputePipeline, Device, Instance, Queue,
    ShaderModule,
};

#[wasm_bindgen]
pub struct ComputeRGBA {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    shader: ShaderModule,
    input: Buffer,
    output: Buffer,
    bind_group: BindGroup,
    pipeline: ComputePipeline,
    len: u32
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[wasm_bindgen]
pub struct Pixel {
    r: f32, g: f32, b: f32, a: f32
}

#[wasm_bindgen]
impl ComputeRGBA
{
    pub async fn new(data: &[Pixel]) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let adapter = instance.request_adapter(&Default::default()).await.unwrap();
        let features = adapter.features();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: features,
                    limits: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let shader = device.create_shader_module(
            &wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("compute.wgsl").into()),
            }
        );

        use wgpu::util::DeviceExt;
        let input = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::MAP_READ
                     | wgpu::BufferUsages::COPY_DST
                     | wgpu::BufferUsages::COPY_SRC,
            } 
        );

        let output = 
            device.create_buffer(
                &wgpu::BufferDescriptor {
                    label: None,
                    size: bytemuck::bytes_of(data).len() as u64,
                    usage: wgpu::BufferUsages::MAP_READ 
                         | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }
            );

        let bind_group_layout = 
            device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }
            );

        let compute_pipeline_layout =
            device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                }
            );

        let pipeline = 
            device.create_compute_pipeline(
                &wgpu::ComputePipelineDescriptor {
                    label: None,
                    layout: Some(&compute_pipeline_layout),
                    module: &shader,
                    entry_point: "main",
                }
            );

        let bind_group = 
            device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: input.as_entire_binding(),
                    }],
                }
            );

        let len = data.len as u32;

        Self {
            instance,
            adapter,
            device,
            queue,
            shader,
            input,
            output,
            bind_group,
            pipeline,
            len
        }
    }

    pub fn run_operation(&mut self) {
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.dispatch(self.len, 1, 1);
        drop(compute_pass);

        encoder.copy_buffer_to_buffer(
            &self.input, 0, 
            &self.output, 0, 
            self.len as u64
        );
        self.queue.submit(
            std::iter::once(encoder.finish())
        );
    }
    
    pub async fn slice_output(&self) -> Option<[Pixel]> {
        let slice = self.output.slice(..);
        if !slice.map_async(wgpu::MapMode::Read).await.is_ok() {
            return None;
        }

        let data_raw = &*slice.get_mapped_range();
        let data = *bytemuck::cast_slice(data_raw);
        Some(data)
    }
} 