use std::any::type_name;

use crate::gpu_buffer::{GpuBuffer, GpuBufferDescriptor};

pub struct Buffers {
    pub positions: PositionsBuffer,
    pub normals: NormalsBuffer,
    pub indices: IndicesBuffer,
    pub ranges: RangesBuffer,
}

impl Buffers {
    const DEFAULT_CAPACITY: u64 = 2048;

    pub fn new(
        device: &wgpu::Device,
        vertex_capacity: u64,
        index_capacity: u64,
        range_capacity: u64,
    ) -> Self {
        Self {
            positions: PositionsBuffer::new(device, vertex_capacity),
            normals: NormalsBuffer::new(device, vertex_capacity),
            indices: IndicesBuffer::new(device, index_capacity),
            ranges: RangesBuffer::new(device, range_capacity),
        }
    }
}

pub struct PositionsBuffer(pub PositionsBufferInner);

type PositionsBufferInner = GpuBuffer<[f32; 4]>;

impl PositionsBuffer {
    pub fn new(device: &wgpu::Device, capacity: u64) -> Self {
        use wgpu::BufferUsages as U;

        Self(GpuBuffer::new(
            &device,
            &GpuBufferDescriptor {
                label: Some(type_name::<PositionsBuffer>()),
                capacity,
                usage: U::VERTEX | U::COPY_DST | U::COPY_SRC,
                mapped_at_creation: false,
            },
        ))
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.0.inner().as_entire_binding()
    }

    // pub fn item_size() -> u64 {
    //     PositionsBufferInner::item_size()
    // }
}

pub struct NormalsBuffer(pub NormalsBufferInner);

type NormalsBufferInner = GpuBuffer<[f32; 4]>;

impl NormalsBuffer {
    pub fn new(device: &wgpu::Device, capacity: u64) -> Self {
        use wgpu::BufferUsages as U;

        Self(GpuBuffer::new(
            &device,
            &GpuBufferDescriptor {
                label: Some(type_name::<NormalsBuffer>()),
                capacity,
                usage: U::VERTEX | U::COPY_DST | U::COPY_SRC,
                mapped_at_creation: false,
            },
        ))
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.0.inner().as_entire_binding()
    }

    // pub fn item_size() -> u64 {
    //     NormalsBuffer::item_size()
    // }
}

pub struct IndicesBuffer(pub IndicesBufferInner);

type IndicesBufferInner = GpuBuffer<[f32; 4]>;

impl IndicesBuffer {
    pub fn new(device: &wgpu::Device, capacity: u64) -> Self {
        use wgpu::BufferUsages as U;

        Self(GpuBuffer::new(
            &device,
            &GpuBufferDescriptor {
                label: Some(type_name::<IndicesBuffer>()),
                capacity,
                usage: U::INDEX | U::COPY_DST | U::COPY_SRC,
                mapped_at_creation: false,
            },
        ))
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.0.inner().as_entire_binding()
    }

    // pub fn item_size() -> u64 {
    //     IndicesBuffer::item_size()
    // }
}

pub struct RangesBuffer(pub RangesBufferInner);

type RangesBufferInner = GpuBuffer<[f32; 4]>;

impl RangesBuffer {
    pub fn new(device: &wgpu::Device, capacity: u64) -> Self {
        use wgpu::BufferUsages as U;

        Self(GpuBuffer::new(
            &device,
            &GpuBufferDescriptor {
                label: Some(type_name::<RangesBuffer>()),
                capacity,
                usage: U::INDEX | U::COPY_DST | U::COPY_SRC,
                mapped_at_creation: false,
            },
        ))
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.0.inner().as_entire_binding()
    }

    // pub fn item_size() -> u64 {
    //     RangesBuffer::item_size()
    // }
}
