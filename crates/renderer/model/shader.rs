macro_rules! impl_shader {
    ($path:expr, $name:ident) => {
        pub struct $name(wgpu::ShaderModule);

        impl $name {
            pub fn new(device: &wgpu::Device) -> Self {
                Self(device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(std::any::type_name::<$name>()),
                    source: wgpu::ShaderSource::Wgsl(include_str!($path).into()),
                }))
            }
        }

        impl AsRef<wgpu::ShaderModule> for $name {
            fn as_ref(&self) -> &wgpu::ShaderModule {
                &self.0
            }
        }
    };
}

impl_shader!("shader/color.wgsl", ColorShader);
