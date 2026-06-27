use bytemuck::{Pod, Zeroable};
use fable_data::{
    big::{AssetMetadata, ExtraMetadata},
    texture::{BcnEncoding, Texture, TextureError},
    tga::{Tga, TgaError},
};
use std::any::type_name;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferUsages,
    CommandEncoder, Device, Extent3d, FragmentState, IndexFormat, MultisampleState, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor, ShaderModule, ShaderStages,
    TexelCopyBufferLayout, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension, VertexAttribute,
    VertexBufferLayout, VertexState, VertexStepMode, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct SkyVertex {
    position: [f32; 3],
    color: [f32; 4],
    uv: [f32; 2],
}

impl SkyVertex {
    const ATTRIBS: [VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2];

    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct SkyUniforms {
    view_proj: [[f32; 4]; 4],
    /// Time of day in hours (0.0 to 24.0).
    /// Shader normalizes this to 0.0-1.0 for LUT sampling.
    time_of_day: f32,
    /// Blend factor between sky_texture_0 and sky_texture_1 (0.0 to 1.0).
    sky_blend: f32,
    /// Padding to align to 16 bytes.
    _padding: [f32; 2],
}

fn generate_skydome_mesh(segments: u32, rings: u32) -> (Vec<SkyVertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let horizon_color = [1.0, 0.7, 0.5, 0.3];
    let zenith_color = [0.8, 0.85, 1.0, 0.0];

    for ring in 0..=rings {
        let elevation = (ring as f32 / rings as f32) * std::f32::consts::FRAC_PI_2;
        let y = elevation.sin();
        let xz_radius = elevation.cos();
        let v = ring as f32 / rings as f32;
        let t = (v * 2.0).min(1.0);
        let t = t * t;
        let color = [
            horizon_color[0] * (1.0 - t) + zenith_color[0] * t,
            horizon_color[1] * (1.0 - t) + zenith_color[1] * t,
            horizon_color[2] * (1.0 - t) + zenith_color[2] * t,
            horizon_color[3] * (1.0 - t) + zenith_color[3] * t,
        ];

        for seg in 0..=segments {
            let azimuth = (seg as f32 / segments as f32) * std::f32::consts::TAU;
            let x = xz_radius * azimuth.cos();
            let z = xz_radius * azimuth.sin();
            let u = seg as f32 / segments as f32;

            vertices.push(SkyVertex {
                position: [x, y, z],
                color,
                uv: [u, v],
            });
        }
    }

    for ring in 0..rings {
        for seg in 0..segments {
            let current = ring * (segments + 1) + seg;
            let next = current + segments + 1;

            indices.push(current as u16);
            indices.push((current + 1) as u16);
            indices.push(next as u16);

            indices.push((current + 1) as u16);
            indices.push((next + 1) as u16);
            indices.push(next as u16);
        }
    }

    (vertices, indices)
}

pub struct SkyDome {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: BindGroup,
}

impl SkyDome {
    pub fn new(device: &Device, uniform_layout: &SkyUniformBindGroupLayout) -> Self {
        let (vertices, indices) = generate_skydome_mesh(32, 16);
        let index_count = indices.len() as u32;

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("sky_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("sky_index_buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let uniforms = SkyUniforms {
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            time_of_day: 12.0, // Default to noon
            sky_blend: 0.0,
            _padding: [0.0; 2],
        };

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("sky_uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sky_uniform_bind_group"),
            layout: &uniform_layout.0,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    pub fn update_uniforms(
        &self,
        queue: &Queue,
        view_proj: [[f32; 4]; 4],
        time_of_day: f32,
        sky_blend: f32,
    ) {
        let uniforms = SkyUniforms {
            view_proj,
            time_of_day,
            sky_blend,
            _padding: [0.0; 2],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
}

/// Lighting colours lookup table texture for GPU-based time-of-day rendering.
///
/// This is a 190×21 pixel texture where:
/// - X-axis (U coordinate) = time of day (0.0 = midnight, 1.0 = next midnight)
/// - Y-axis (V coordinate) = color property row
///
/// In shaders, sample using:
/// ```wgsl
/// let u = time_of_day / 24.0;
/// let v = (ROW + 0.5) / 21.0;  // +0.5 centers in texel
/// let color = textureSample(lighting_lut, lut_sampler, vec2(u, v));
/// ```
///
/// Row indices (V = (row + 0.5) / 21.0):
/// - 0: Diffuse light color
/// - 1: Ambient light color
/// - 2: Cloud color
/// - 3: Backlight color
/// - 6: Fog color
/// - 8: Sun color
/// - 9: Moon color
/// - 10: Stars color
/// - 11: Sun flare color
/// - 12: Lens flare color
/// - 13: Sky gradient top color
/// - 14: Sky gradient top alpha
/// - 15: Sky gradient bottom color
/// - 16: Sky gradient bottom alpha
pub struct LightingColoursTexture {
    texture: wgpu::Texture,
    view: TextureView,
    sampler: wgpu::Sampler,
}

/// Row indices for the lighting colours lookup texture.
/// Use in shader as: `v = (ROW + 0.5) / 21.0`
#[allow(dead_code)]
pub mod lighting_row {
    /// Normalized V coordinate for a row index.
    /// Centers the sample in the texel to avoid row bleeding.
    pub const fn v_coord(row: u32) -> f32 {
        (row as f32 + 0.5) / 21.0
    }

    pub const DIFFUSE: u32 = 0;
    pub const AMBIENT: u32 = 1;
    pub const CLOUD_COLOUR: u32 = 2;
    pub const BACKLIGHT: u32 = 3;
    pub const FOG_COLOUR: u32 = 6;
    pub const SUN_COLOUR: u32 = 8;
    pub const MOON_COLOUR: u32 = 9;
    pub const STARS_COLOUR: u32 = 10;
    pub const SUN_FLARE_COLOUR: u32 = 11;
    pub const LENS_FLARE_COLOUR: u32 = 12;
    pub const SKY_GRADIENT_TOP: u32 = 13;
    pub const SKY_GRADIENT_TOP_ALPHA: u32 = 14;
    pub const SKY_GRADIENT_BOTTOM: u32 = 15;
    pub const SKY_GRADIENT_BOTTOM_ALPHA: u32 = 16;
}

impl LightingColoursTexture {
    /// Load from raw TGA file bytes.
    pub fn from_tga_bytes(
        device: &Device,
        queue: &Queue,
        tga_bytes: &[u8],
    ) -> Result<Self, LightingColoursError> {
        let tga = Tga::parse(tga_bytes).map_err(LightingColoursError::Tga)?;

        let width = tga.width();
        let height = tga.height();
        let rgba_data = tga.to_rgba();

        tracing::info!(
            "Lighting colours LUT loaded: {}x{} (time samples × color rows)",
            width,
            height,
        );

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("lighting_colours_lut"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            &rgba_data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: None,
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&TextureViewDescriptor::default());

        // Linear filtering for smooth time interpolation.
        // Clamp to edge so times outside 0-24 don't wrap weirdly.
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("lighting_colours_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}

#[derive(Debug, Display, Error)]
pub enum LightingColoursError {
    #[display("TGA parse error: {_0}")]
    Tga(TgaError),
}

fn dxt_to_bcn_encoding(dxt: u16) -> BcnEncoding {
    match dxt {
        1 | 33 => BcnEncoding::Bc1,
        3 | 34 => BcnEncoding::Bc2,
        5 | 35 => BcnEncoding::Bc3,
        _ => BcnEncoding::Bc1,
    }
}

use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum SkyTextureError {
    NotATexture,
    #[display("UnsupportedDxtFormat({_0})")]
    UnsupportedDxtFormat(#[error(not(source))] u16),
    Parse(TextureError),
}

pub struct SkyUniformBindGroupLayout(BindGroupLayout);

impl SkyUniformBindGroupLayout {
    pub fn new(device: &Device) -> Self {
        Self(device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(type_name::<Self>()),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        }))
    }
}

/// Bind group layout for sky textures (two textures for blending + shared sampler).
pub struct SkyTextureBindGroupLayout(BindGroupLayout);

impl SkyTextureBindGroupLayout {
    pub fn new(device: &Device) -> Self {
        Self(device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(type_name::<Self>()),
            entries: &[
                // Sky texture 0 (primary)
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sky texture 1 (for blending)
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Shared sampler
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        }))
    }
}

/// Bind group layout for the lighting colours LUT texture.
pub struct LightingLutBindGroupLayout(BindGroupLayout);

impl LightingLutBindGroupLayout {
    pub fn new(device: &Device) -> Self {
        Self(device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(type_name::<Self>()),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        }))
    }
}

pub struct OuterSkyShader(ShaderModule);

impl OuterSkyShader {
    pub fn new(device: &Device) -> Self {
        Self(device.create_shader_module(include_wgsl!("sky/outer_sky.wgsl")))
    }
}

pub struct OuterSkyPipelineLayout(PipelineLayout);

impl OuterSkyPipelineLayout {
    pub fn new(
        device: &Device,
        uniform_layout: &SkyUniformBindGroupLayout,
        texture_layout: &SkyTextureBindGroupLayout,
        lut_layout: &LightingLutBindGroupLayout,
    ) -> Self {
        Self(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            bind_group_layouts: &[&uniform_layout.0, &texture_layout.0, &lut_layout.0],
            immediate_size: 0,
        }))
    }
}

pub struct OuterSkyPipeline(RenderPipeline);

impl OuterSkyPipeline {
    pub fn new(
        device: &Device,
        layout: &OuterSkyPipelineLayout,
        shader: &OuterSkyShader,
        target_format: TextureFormat,
    ) -> Self {
        Self(device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(type_name::<Self>()),
            layout: Some(&layout.0),
            vertex: VertexState {
                module: &shader.0,
                entry_point: Some("vs_main"),
                buffers: &[SkyVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader.0,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(target_format.into())],
            }),
            primitive: PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        }))
    }
}

/// Bound lighting LUT with its bind group.
pub struct BoundLightingLut {
    pub lut: LightingColoursTexture,
    pub bind_group: BindGroup,
}

/// A loaded sky texture (without its own bind group - bind groups are created in SkyPass).
pub struct LoadedSkyTexture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
}

impl LoadedSkyTexture {
    pub fn load(
        device: &Device,
        queue: &Queue,
        asset_info: &AssetMetadata,
        asset_data: &[u8],
    ) -> Result<Self, SkyTextureError> {
        use SkyTextureError as E;

        let texture_extras = match &asset_info.extras {
            Some(ExtraMetadata::Texture(extras)) => extras,
            _ => return Err(E::NotATexture),
        };

        let width = texture_extras.width as u32;
        let height = texture_extras.height as u32;

        let format = match texture_extras.dxt_compression {
            1 | 33 => TextureFormat::Bc1RgbaUnorm,
            3 | 34 => TextureFormat::Bc2RgbaUnorm,
            5 | 35 => TextureFormat::Bc3RgbaUnorm,
            other => return Err(E::UnsupportedDxtFormat(other)),
        };

        let mut input = asset_data;
        let texture_data = Texture::parse(
            &mut input,
            width as usize,
            height as usize,
            texture_extras.depth as usize,
            texture_extras.top_mip_map_size as usize,
            dxt_to_bcn_encoding(texture_extras.dxt_compression),
        )
        .map_err(E::Parse)?;

        let bcn_data = texture_data.get_top_mip_bcn_image().map_err(E::Parse)?;

        tracing::info!(
            "Sky texture loaded: {} ({}x{}, {:?})",
            asset_info.symbol_name,
            width,
            height,
            format,
        );

        let texture = device.create_texture(&TextureDescriptor {
            label: Some(&asset_info.symbol_name),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let block_size = match format {
            TextureFormat::Bc1RgbaUnorm => 8,
            TextureFormat::Bc2RgbaUnorm | TextureFormat::Bc3RgbaUnorm => 16,
            _ => unreachable!(),
        };
        let blocks_wide = (width + 3) / 4;
        let bytes_per_row = blocks_wide * block_size;

        queue.write_texture(
            texture.as_image_copy(),
            bcn_data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: None,
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&TextureViewDescriptor::default());

        Ok(Self { texture, view })
    }
}

pub struct OuterSkyPass {
    texture_layout: SkyTextureBindGroupLayout,
    lut_layout: LightingLutBindGroupLayout,
    pipeline: OuterSkyPipeline,
    dome: SkyDome,
    sky_sampler: wgpu::Sampler,
    texture0: Option<LoadedSkyTexture>,
    texture1: Option<LoadedSkyTexture>,
    sky_textures_bind_group: Option<BindGroup>,
    lighting_lut: Option<BoundLightingLut>,
}

impl OuterSkyPass {
    pub fn new(device: &Device, surface_format: TextureFormat) -> Self {
        let shader = OuterSkyShader::new(device);
        let uniform_layout = SkyUniformBindGroupLayout::new(device);
        let texture_layout = SkyTextureBindGroupLayout::new(device);
        let lut_layout = LightingLutBindGroupLayout::new(device);
        let layout =
            OuterSkyPipelineLayout::new(device, &uniform_layout, &texture_layout, &lut_layout);
        let pipeline = OuterSkyPipeline::new(device, &layout, &shader, surface_format);
        let dome = SkyDome::new(device, &uniform_layout);

        let sky_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("sky_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        Self {
            texture_layout,
            lut_layout,
            pipeline,
            dome,
            sky_sampler,
            texture0: None,
            texture1: None,
            sky_textures_bind_group: None,
            lighting_lut: None,
        }
    }

    /// Set the primary sky texture (texture0).
    pub fn set_texture0(
        &mut self,
        device: &Device,
        queue: &Queue,
        asset_info: &AssetMetadata,
        asset_data: &[u8],
    ) -> Result<(), SkyTextureError> {
        let texture = LoadedSkyTexture::load(device, queue, asset_info, asset_data)?;
        self.texture0 = Some(texture);
        self.rebuild_sky_bind_group(device);
        Ok(())
    }

    /// Set the secondary sky texture for blending (texture1).
    pub fn set_texture1(
        &mut self,
        device: &Device,
        queue: &Queue,
        asset_info: &AssetMetadata,
        asset_data: &[u8],
    ) -> Result<(), SkyTextureError> {
        let texture = LoadedSkyTexture::load(device, queue, asset_info, asset_data)?;
        self.texture1 = Some(texture);
        self.rebuild_sky_bind_group(device);
        Ok(())
    }

    fn rebuild_sky_bind_group(&mut self, device: &Device) {
        let Some(tex0) = &self.texture0 else {
            self.sky_textures_bind_group = None;
            return;
        };

        let tex1_view = self
            .texture1
            .as_ref()
            .map(|t| &t.view)
            .unwrap_or(&tex0.view);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sky_textures_bind_group"),
            layout: &self.texture_layout.0,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&tex0.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(tex1_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&self.sky_sampler),
                },
            ],
        });

        self.sky_textures_bind_group = Some(bind_group);
    }

    pub fn set_lighting_lut(
        &mut self,
        device: &Device,
        queue: &Queue,
        tga_bytes: &[u8],
    ) -> Result<(), LightingColoursError> {
        let lut = LightingColoursTexture::from_tga_bytes(device, queue, tga_bytes)?;

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("lighting_lut_bind_group"),
            layout: &self.lut_layout.0,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(lut.view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(lut.sampler()),
                },
            ],
        });

        self.lighting_lut = Some(BoundLightingLut { lut, bind_group });

        Ok(())
    }

    pub fn update_uniforms(
        &self,
        queue: &Queue,
        view_proj: [[f32; 4]; 4],
        time_of_day: f32,
        sky_blend: f32,
    ) {
        self.dome
            .update_uniforms(queue, view_proj, time_of_day, sky_blend);
    }

    pub fn pass(&self, cmd: &mut CommandEncoder, target_texture_view: &TextureView) {
        let Some(sky_bind_group) = &self.sky_textures_bind_group else {
            return;
        };

        let Some(lut) = &self.lighting_lut else {
            return;
        };

        let mut rpass = cmd.begin_render_pass(&RenderPassDescriptor {
            label: Some(type_name::<Self>()),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        rpass.set_pipeline(&self.pipeline.0);
        rpass.set_bind_group(0, &self.dome.uniform_bind_group, &[]);
        rpass.set_bind_group(1, sky_bind_group, &[]);
        rpass.set_bind_group(2, &lut.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.dome.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.dome.index_buffer.slice(..), IndexFormat::Uint16);
        rpass.draw_indexed(0..self.dome.index_count, 0, 0..1);
    }
}
