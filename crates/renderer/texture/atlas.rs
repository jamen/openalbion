use crate::gpu_buffer::{GpuBuffer, GpuBufferDescriptor};
use etagere::AtlasAllocator;
use std::any::type_name;

pub struct Atlas {
    pub texture: wgpu::Texture,
    pub sampler: wgpu::Sampler,
    pub alloc: AtlasAllocator,
    pub offsets: OffsetBuffer,
}

impl Atlas {
    const DEFAULT_TEXTURE_SIZE: u32 = 2048;
    const DEFAULT_OFFSET_CAPACITY: u64 = 64;

    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self::with_capacity(
            device,
            format,
            Self::DEFAULT_TEXTURE_SIZE,
            Self::DEFAULT_TEXTURE_SIZE,
            Self::DEFAULT_OFFSET_CAPACITY,
        )
    }

    fn with_capacity(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        offset_capacity: u64,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let alloc =
            AtlasAllocator::new([width.try_into().unwrap(), height.try_into().unwrap()].into());

        let offsets = OffsetBuffer::new(device, offset_capacity);

        Self {
            texture,
            sampler,
            alloc,
            offsets,
        }
    }

    pub fn create_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        size: &[u32; 3],
        data: &[u8],
    ) -> TextureId {
        debug_assert!(size[2] == 1);

        let bytes_per_row = size[0] * self.texture.format().block_size(None).unwrap();

        let width: i32 = size[0].try_into().unwrap();
        let height: i32 = size[1].try_into().unwrap();
        let allocation = self.alloc.allocate([width, height].into()).unwrap();

        let position: [u32; 2] = [
            allocation.rectangle.min.x.try_into().unwrap(),
            allocation.rectangle.min.y.try_into().unwrap(),
        ];

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: position[0],
                    y: position[1],
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(size[1]),
            },
            wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: size[2],
            },
        );

        self.offsets.write(
            device,
            queue,
            encoder,
            &[[position[0] as u16, position[1] as u16]],
        );

        TextureId { id: allocation.id }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct TextureId {
    id: etagere::AllocId,
}

type OffsetBufferInner = GpuBuffer<[u16; 2]>;

pub struct OffsetBuffer(OffsetBufferInner);

impl OffsetBuffer {
    pub fn new(device: &wgpu::Device, capacity: u64) -> Self {
        use wgpu::BufferUsages as U;

        Self(GpuBuffer::with_zeroes(
            &device,
            &GpuBufferDescriptor {
                label: Some(type_name::<OffsetBuffer>()),
                capacity,
                usage: U::UNIFORM | U::COPY_SRC | U::COPY_DST,
                mapped_at_creation: false,
            },
        ))
    }

    // pub fn as_entire_binding(&self) -> wgpu::BindingResource {
    //     self.0.inner().as_entire_binding()
    // }

    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        data: &[[u16; 2]],
    ) {
        self.0.write(device, queue, encoder, data)
    }
}
