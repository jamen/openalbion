use std::mem::size_of;

use glam::{Vec2, Vec3};
use range_alloc::RangeAllocator;
use thiserror::Error;

use crate::renderer::Base;

pub const POSITION_SIZE: usize = size_of::<Vec3>();
// pub const NORMAL_SIZE: usize = size_of::<Vec3>();
// pub const TANGENT_SIZE: usize = size_of::<Vec3>();
pub const UV_0_SIZE: usize = size_of::<Vec2>();
pub const INDEX_SIZE: usize = size_of::<u32>();

const DEFAULT_SIZE: usize = 65536;

pub struct Megabuffer {
    buffers: MeshBuffers,
    vertex_alloc: RangeAllocator<usize>,
    index_alloc: RangeAllocator<usize>,
}

impl Megabuffer {
    pub fn new(base: &Base) -> Result<Self, MeshManagerError> {
        let buffers = MeshBuffers::new(base);
        let vertex_alloc = RangeAllocator::new(0..DEFAULT_SIZE);
        let index_alloc = RangeAllocator::new(0..DEFAULT_SIZE);
        Ok(Self {
            buffers,
            vertex_alloc,
            index_alloc,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum MeshManagerError {}

pub struct MeshBuffers {
    positions: wgpu::Buffer,
    // normals: wgpu::Buffer,
    // tangents: wgpu::Buffer,
    uv_0: wgpu::Buffer,
    indices: wgpu::Buffer,
}

impl MeshBuffers {
    pub fn new(base: &Base) -> Self {
        let positions = base.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("positions"),
            size: (POSITION_SIZE * DEFAULT_SIZE) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        // let normals = base.device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("normals"),
        //     size: (NORMAL_SIZE * DEFAULT_SIZE) as u64,
        //     usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        //     mapped_at_creation: false,
        // });

        // let tangents = base.device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("tangents"),
        //     size: (TANGENT_SIZE * DEFAULT_SIZE) as u64,
        //     usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        //     mapped_at_creation: false,
        // });

        let uv_0 = base.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UV 0"),
            size: (UV_0_SIZE * DEFAULT_SIZE) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let indices = base.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("indices"),
            size: (INDEX_SIZE * DEFAULT_SIZE) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
            mapped_at_creation: false,
        });

        MeshBuffers {
            positions,
            // normals,
            // tangents,
            uv_0,
            indices,
        }
    }
}
