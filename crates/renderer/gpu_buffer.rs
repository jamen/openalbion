use bytemuck::Pod;
use std::{marker::PhantomData, mem::size_of};

/// A growable and type-specified `wgpu::Buffer`
pub struct GpuBuffer<T: Pod> {
    buffer: wgpu::Buffer,
    len: u64,
    marker: PhantomData<Vec<T>>,
}

pub struct GpuBufferDescriptor<L> {
    pub label: L,
    pub usage: wgpu::BufferUsages,
    pub capacity: u64,
    pub mapped_at_creation: bool,
}

impl<T: Pod> GpuBuffer<T> {
    pub fn new(device: &wgpu::Device, descriptor: &GpuBufferDescriptor<wgpu::Label>) -> Self {
        use wgpu::BufferUsages as U;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: descriptor.label,
            size: descriptor.capacity * Self::item_size(),
            usage: descriptor.usage | U::COPY_SRC | U::COPY_DST,
            mapped_at_creation: descriptor.mapped_at_creation,
        });

        Self {
            len: 0,
            buffer,
            marker: Default::default(),
        }
    }

    pub fn with_zeroes(
        device: &wgpu::Device,
        descriptor: &GpuBufferDescriptor<wgpu::Label>,
    ) -> Self {
        let mut buf = Self::new(device, descriptor);
        buf.len = descriptor.capacity;
        buf
    }

    pub fn capacity(&self) -> u64 {
        self.buffer.size() / Self::item_size()
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub const fn item_size() -> u64 {
        size_of::<T>() as u64
    }

    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        data: &[T],
    ) {
        // Grow buffer
        let data_len = data.len() as u64;
        let new_len = self.len() + data_len;
        if new_len > self.capacity() {
            self.grow(device, encoder, new_len);
        }
        // Write data
        let offset = self.len() * Self::item_size();
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(&data));
        self.len += data_len;
    }

    pub fn write_at(&mut self, queue: &wgpu::Queue, offset: u64, data: &[T]) {
        debug_assert!(self.len() >= offset + data.len() as u64);
        // Write data
        let offset = offset * Self::item_size();
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(&data));
    }

    fn grow(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, new_len: u64) {
        let next_capacity = new_len.next_power_of_two();
        let next_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            usage: self.buffer.usage(),
            size: next_capacity * Self::item_size(),
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(
            &self.buffer,
            0,
            &next_buffer,
            0,
            self.len * Self::item_size(),
        );
        self.buffer = next_buffer;
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn inner(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    // pub fn slice<R: RangeBounds<u64>>(&self, range: R) -> wgpu::BufferSlice<'_> {
    //     use Bound::{Excluded, Included, Unbounded};
    //     let start = match range.end_bound() {
    //         Unbounded => Unbounded,
    //         Excluded(&x) => Excluded(x * Self::item_size()),
    //         Included(&x) => Included(x * Self::item_size()),
    //     };
    //     let end = match range.end_bound() {
    //         Unbounded => Excluded(self.len() * Self::item_size()),
    //         Excluded(&x) => {
    //             debug_assert!(x <= self.len());
    //             Excluded(x * Self::item_size())
    //         }
    //         Included(&x) => {
    //             debug_assert!(x < self.len());
    //             Excluded(x * Self::item_size())
    //         }
    //     };
    //     self.buffer.slice((start, end))
    // }
}

impl<T: Pod> AsRef<wgpu::Buffer> for GpuBuffer<T> {
    fn as_ref(&self) -> &wgpu::Buffer {
        self.inner()
    }
}
