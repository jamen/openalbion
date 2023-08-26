use std::any::type_name;

pub struct BindGroupLayouts {
    pub color: ColorBindGroupLayout,
}

impl BindGroupLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        let color = ColorBindGroupLayout::new(&device);
        Self { color }
    }
}

pub struct ColorBindGroupLayout(wgpu::BindGroupLayout);

impl ColorBindGroupLayout {
    pub fn new(device: &wgpu::Device) -> Self {
        Self(
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(type_name::<ColorBindGroupLayout>()),
                entries: &[],
            }),
        )
    }
}

impl AsRef<wgpu::BindGroupLayout> for ColorBindGroupLayout {
    fn as_ref(&self) -> &wgpu::BindGroupLayout {
        &self.0
    }
}
