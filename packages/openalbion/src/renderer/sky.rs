use super::texture::{TextureUploadError, linear_clamp_sampler, upload_texture};
use bytemuck::{Pod, Zeroable};
use fable_data::{
    big::AssetMetadata,
    tga::{Tga, TgaError},
};
use std::any::type_name;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferUsages,
    CommandEncoder, Device, Extent3d, FragmentState, IndexFormat,
    MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, Queue,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType, ShaderModule,
    ShaderStages, TexelCopyBufferLayout, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode, include_wgsl,
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
pub(crate) struct SkyUniforms {
    view_proj: [[f32; 4]; 4],
    /// RGB from LUT row 13 (SkyGradientTop), A from row 14 (SkyGradientTopAlpha).
    zenith_color: [f32; 4],
    /// RGB from LUT row 15 (SkyGradientBottom), A from row 16 (SkyGradientBottomAlpha).
    horizon_color: [f32; 4],
}

fn build_outer_sky_mesh(segments: u32) -> (Vec<SkyVertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let dome_top_y: f32 = 7000.0;
    let dome_bottom_y: f32 = -500.0;
    let dome_radius: f32 = 6500.0;

    vertices.push(SkyVertex {
        position: [0.0, dome_top_y, 0.0],
        color: [0.0, 0.0, 0.0, 0.0],
        uv: [-1e-6, -1e-6],
    });

    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let (sin_a, cos_a) = (angle.sin(), angle.cos());
        let x = cos_a * dome_radius;
        let z = sin_a * dome_radius;
        let u = i as f32 / segments as f32;

        vertices.push(SkyVertex {
            position: [x, dome_bottom_y, z],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [u, 1.0],
        });

        vertices.push(SkyVertex {
            position: [x, dome_top_y, z],
            color: [0.0, 0.0, 0.0, 0.0],
            uv: [u, 0.0],
        });
    }

    for i in 0..segments {
        let center: u16 = 0;
        let bottom_curr: u16 = 1 + (i * 2) as u16;
        let top_curr: u16 = 2 + (i * 2) as u16;
        let bottom_next: u16 = 1 + (((i + 1) % segments) * 2) as u16;
        let top_next: u16 = 2 + (((i + 1) % segments) * 2) as u16;

        indices.extend_from_slice(&[center, bottom_curr, bottom_next]);

        indices.extend_from_slice(&[bottom_curr, bottom_next, top_curr]);
        indices.extend_from_slice(&[bottom_next, top_next, top_curr]);
    }

    (vertices, indices)
}

fn build_base_band_mesh(segments: u32) -> (Vec<SkyVertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let dome_radius: f32 = 6500.0;
    let dome_bottom_y: f32 = -500.0;
    let base_center_y: f32 = -10000.0;

    vertices.push(SkyVertex {
        position: [0.0, base_center_y, 0.0],
        color: [0.0, 0.0, 0.0, 0.0],
        uv: [0.0, 0.0],
    });

    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * dome_radius;
        let z = angle.sin() * dome_radius;
        vertices.push(SkyVertex {
            position: [x, dome_bottom_y, z],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 0.0],
        });
    }

    for i in 0..segments {
        indices.extend_from_slice(&[0, 1 + i as u16, 1 + ((i + 1) % segments) as u16]);
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
        let (vertices, indices) = build_outer_sky_mesh(36);
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
            zenith_color: [0.4, 0.6, 1.0, 0.0],
            horizon_color: [0.6, 0.7, 0.9, 1.0],
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

    pub fn update_uniforms(&self, queue: &Queue, uniforms: &SkyUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
    }
}

pub struct SkyBaseBand {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: BindGroup,
}

impl SkyBaseBand {
    pub fn new(device: &Device, uniform_layout: &SkyUniformBindGroupLayout) -> Self {
        let (vertices, indices) = build_base_band_mesh(36);
        let index_count = indices.len() as u32;
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("base_band_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("base_band_index_buffer"),
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
            zenith_color: [0.4, 0.6, 1.0, 0.0],
            horizon_color: [0.6, 0.7, 0.9, 1.0],
        };
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("base_band_uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("base_band_uniform_bind_group"),
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

    pub fn update_uniforms(&self, queue: &Queue, uniforms: &SkyUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
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
        // Pad to 256-aligned rows (required by wgpu write_texture)
        let row_bytes = (width * 4).max(1);
        let padded_row_bytes = row_bytes.div_ceil(256) * 256;
        let mut padded = vec![0u8; padded_row_bytes as usize * height as usize];
        for y in 0..height as usize {
            let src = y * row_bytes as usize;
            let dst = y * padded_row_bytes as usize;
            padded[dst..dst + row_bytes as usize]
                .copy_from_slice(&rgba_data[src..src + row_bytes as usize]);
        }

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
            &padded,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_row_bytes),
                rows_per_image: None,
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&TextureViewDescriptor::default());

        // Linear filtering for smooth time interpolation; clamp so times outside 0-24 don't wrap.
        let sampler = linear_clamp_sampler(device, "lighting_colours_sampler");

        Ok(Self { view, sampler })
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

use derive_more::{Display, Error};

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

pub struct OuterSkyPass {
    texture_layout: SkyTextureBindGroupLayout,
    lut_layout: LightingLutBindGroupLayout,
    pipeline: OuterSkyPipeline,
    dome: SkyDome,
    base_band: SkyBaseBand,
    sky_sampler: wgpu::Sampler,
    texture0: Option<TextureView>,
    texture1: Option<TextureView>,
    sky_textures_bind_group: Option<BindGroup>,
    lighting_lut: Option<BindGroup>,
    /// Flat RGBA pixel data from the LUT (190 × 21 pixels).
    /// Used to compute per-frame zenith/horizon gradient colours.
    lut_pixels: Vec<[f32; 4]>,
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
        let base_band = SkyBaseBand::new(device, &uniform_layout);

        let sky_sampler = linear_clamp_sampler(device, "sky_sampler");

        Self {
            texture_layout,
            lut_layout,
            pipeline,
            dome,
            base_band,
            sky_sampler,
            texture0: None,
            texture1: None,
            sky_textures_bind_group: None,
            lighting_lut: None,
            lut_pixels: Vec::new(),
        }
    }

    /// Set the primary sky texture (texture0).
    pub fn set_texture0(
        &mut self,
        device: &Device,
        queue: &Queue,
        asset_info: &AssetMetadata,
        asset_data: &[u8],
    ) -> Result<(), TextureUploadError> {
        self.texture0 = Some(upload_texture(device, queue, asset_info, asset_data)?);
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
    ) -> Result<(), TextureUploadError> {
        self.texture1 = Some(upload_texture(device, queue, asset_info, asset_data)?);
        self.rebuild_sky_bind_group(device);
        Ok(())
    }

    fn rebuild_sky_bind_group(&mut self, device: &Device) {
        let Some(tex0) = &self.texture0 else {
            self.sky_textures_bind_group = None;
            return;
        };

        // Fall back to texture0 for the blend slot until a second texture is set.
        let tex1_view = self.texture1.as_ref().unwrap_or(tex0);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sky_textures_bind_group"),
            layout: &self.texture_layout.0,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(tex0),
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
        let tga = Tga::parse(tga_bytes).map_err(LightingColoursError::Tga)?;
        let width = tga.width() as usize;
        let height = tga.height() as usize;
        let rgba = tga.to_rgba();

        // Store pixel data for CPU-side LUT lookups (used by update_uniforms).
        self.lut_pixels = rgba
            .chunks_exact(4)
            .map(|c| [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, c[3] as f32 / 255.0])
            .collect();
        tracing::info!("Stored LUT pixels: {}x{}", width, height);

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

        self.lighting_lut = Some(bind_group);

        Ok(())
    }

    fn lut_lookup(&self, row: usize, time_of_day: f32) -> [f32; 4] {
        if self.lut_pixels.is_empty() {
            return [0.5, 0.5, 0.5, 1.0];
        }
        let height = 21usize;
        let width = self.lut_pixels.len() / height;
        if width == 0 || row >= height {
            return [0.5, 0.5, 0.5, 1.0];
        }
        let u = (time_of_day / 24.0 * width as f32) as usize % width;
        self.lut_pixels[row * width + u]
    }

    pub fn update_uniforms(
        &self,
        queue: &Queue,
        view_proj: [[f32; 4]; 4],
        time_of_day: f32,
        _sky_blend: f32,
    ) {
        let zenith = self.lut_lookup(13, time_of_day);
        let zenith_alpha = self.lut_lookup(14, time_of_day);
        let horizon = self.lut_lookup(15, time_of_day);
        let horizon_alpha = self.lut_lookup(16, time_of_day);

        let uniforms = SkyUniforms {
            view_proj,
            zenith_color: [zenith[0], zenith[1], zenith[2], zenith_alpha[0]],
            horizon_color: [horizon[0], horizon[1], horizon[2], horizon_alpha[0]],
        };

        self.dome.update_uniforms(queue, &uniforms);
        self.base_band.update_uniforms(queue, &uniforms);
    }

    pub fn pass(&self, cmd: &mut CommandEncoder, target_texture_view: &TextureView) {
        let Some(sky_bind_group) = &self.sky_textures_bind_group else {
            tracing::debug!("Sky pass: no textures bind group — sky skipped");
            return;
        };
        let Some(lut_bind_group) = &self.lighting_lut else {
            tracing::debug!("Sky pass: no lighting LUT — sky skipped");
            return;
        };

        tracing::trace!(
            "Sky pass: dome {} indices, base_band {} indices",
            self.dome.index_count,
            self.base_band.index_count,
        );

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
        rpass.set_bind_group(2, lut_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.dome.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.dome.index_buffer.slice(..), IndexFormat::Uint16);
        rpass.draw_indexed(0..self.dome.index_count, 0, 0..1);

        rpass.set_bind_group(0, &self.base_band.uniform_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.base_band.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.base_band.index_buffer.slice(..), IndexFormat::Uint16);
        rpass.draw_indexed(0..self.base_band.index_count, 0, 0..1);
    }
}

fn sun_direction(time_of_day: f32) -> [f32; 3] {
    let t = time_of_day / 24.0;
    let azimuth = (t - 0.25) * std::f32::consts::TAU;
    let elevation = (t * std::f32::consts::PI).sin();
    let h = elevation.max(0.0).sqrt();
    [azimuth.cos() * h, elevation.max(0.0), azimuth.sin() * h]
}

fn moon_direction(time_of_day: f32) -> [f32; 3] {
    let d = sun_direction(time_of_day);
    [-d[0], -d[1], -d[2]]
}

fn build_sprite_quad(position: [f32; 3], size: f32) -> (Vec<SkyVertex>, Vec<u16>) {
    let dir = {
        let len = (position[0] * position[0] + position[1] * position[1] + position[2] * position[2])
            .sqrt();
        if len < 0.001 {
            return (vec![], vec![]);
        }
        [position[0] / len, position[1] / len, position[2] / len]
    };
    let dot = dir[1]; // dot(dir, (0,1,0))
    let ref_vec: [f32; 3] = if dot.abs() > 0.999 {
        [0.0, 0.0, 1.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let right = {
        let x = ref_vec[1] * dir[2] - ref_vec[2] * dir[1];
        let y = ref_vec[2] * dir[0] - ref_vec[0] * dir[2];
        let z = ref_vec[0] * dir[1] - ref_vec[1] * dir[0];
        let len = (x * x + y * y + z * z).sqrt();
        if len < 0.001 {
            return (vec![], vec![]);
        }
        [x / len, y / len, z / len]
    };
    let up = {
        let x = dir[1] * right[2] - dir[2] * right[1];
        let y = dir[2] * right[0] - dir[0] * right[2];
        let z = dir[0] * right[1] - dir[1] * right[0];
        [x, y, z]
    };

    let h = size * 0.5;
    let p = position;
    let vertices = vec![
        SkyVertex {
            position: [p[0] - right[0] * h - up[0] * h, p[1] - right[1] * h - up[1] * h, p[2] - right[2] * h - up[2] * h],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 0.0],
        },
        SkyVertex {
            position: [p[0] + right[0] * h - up[0] * h, p[1] + right[1] * h - up[1] * h, p[2] + right[2] * h - up[2] * h],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [1.0, 0.0],
        },
        SkyVertex {
            position: [p[0] + right[0] * h + up[0] * h, p[1] + right[1] * h + up[1] * h, p[2] + right[2] * h + up[2] * h],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [1.0, 1.0],
        },
        SkyVertex {
            position: [p[0] - right[0] * h + up[0] * h, p[1] - right[1] * h + up[1] * h, p[2] - right[2] * h + up[2] * h],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 1.0],
        },
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];
    (vertices, indices)
}

pub struct SkySpriteShader(ShaderModule);

impl SkySpriteShader {
    pub fn new(device: &Device) -> Self {
        Self(device.create_shader_module(include_wgsl!("sky/sky_sprite.wgsl")))
    }
}

pub struct SkySpriteTextureBindGroupLayout(BindGroupLayout);

impl SkySpriteTextureBindGroupLayout {
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

pub struct SkySpritePipelineLayout(PipelineLayout);

impl SkySpritePipelineLayout {
    pub fn new(
        device: &Device,
        uniform_layout: &SkyUniformBindGroupLayout,
        texture_layout: &SkySpriteTextureBindGroupLayout,
    ) -> Self {
        Self(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            bind_group_layouts: &[&uniform_layout.0, &texture_layout.0],
            immediate_size: 0,
        }))
    }
}

pub struct SkySpritePipeline(RenderPipeline);

impl SkySpritePipeline {
    pub fn new(
        device: &Device,
        layout: &SkySpritePipelineLayout,
        shader: &SkySpriteShader,
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
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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

struct SpriteMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: BindGroup,
}

impl SpriteMesh {
    fn new(
        device: &Device,
        uniform_layout: &SkyUniformBindGroupLayout,
        position: [f32; 3],
        size: f32,
    ) -> Self {
        let (vertices, indices) = build_sprite_quad(position, size);

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("sprite_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("sprite_index_buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let uniforms = SkyUniforms {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            zenith_color: [0.0; 4],
            horizon_color: [0.0; 4],
        };

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("sprite_uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sprite_uniform_bind_group"),
            layout: &uniform_layout.0,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    fn update(&self, queue: &Queue, position: [f32; 3], size: f32, view_proj: [[f32; 4]; 4]) {
        let (vertices, _) = build_sprite_quad(position, size);
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        let uniforms = SkyUniforms {
            view_proj,
            zenith_color: [0.0; 4],
            horizon_color: [0.0; 4],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
}

pub struct SkySpritePass {
    pipeline: SkySpritePipeline,
    texture_layout: SkySpriteTextureBindGroupLayout,
    sprite_sampler: wgpu::Sampler,
    sun_mesh: SpriteMesh,
    moon_mesh: SpriteMesh,
    sun_texture_bind_group: Option<BindGroup>,
    moon_texture_bind_group: Option<BindGroup>,
}

impl SkySpritePass {
    pub fn new(device: &Device, surface_format: TextureFormat) -> Self {
        let uniform_layout = SkyUniformBindGroupLayout::new(device);
        let texture_layout = SkySpriteTextureBindGroupLayout::new(device);
        let shader = SkySpriteShader::new(device);
        let layout = SkySpritePipelineLayout::new(device, &uniform_layout, &texture_layout);
        let pipeline = SkySpritePipeline::new(device, &layout, &shader, surface_format);

        let sun_mesh = SpriteMesh::new(device, &uniform_layout, [0.0, 7000.0, 0.0], 600.0);
        let moon_mesh = SpriteMesh::new(device, &uniform_layout, [0.0, 7000.0, 0.0], 400.0);
        let sprite_sampler = linear_clamp_sampler(device, "sprite_sampler");

        Self {
            pipeline,
            texture_layout,
            sprite_sampler,
            sun_mesh,
            moon_mesh,
            sun_texture_bind_group: None,
            moon_texture_bind_group: None,
        }
    }

    pub fn set_sun_texture(
        &mut self,
        device: &Device,
        queue: &Queue,
        asset_info: &AssetMetadata,
        asset_data: &[u8],
    ) -> Result<(), TextureUploadError> {
        let view = upload_texture(device, queue, asset_info, asset_data)?;
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sun_texture_bind_group"),
            layout: &self.texture_layout.0,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sprite_sampler),
                },
            ],
        });
        self.sun_texture_bind_group = Some(bind_group);
        Ok(())
    }

    pub fn set_moon_texture(
        &mut self,
        device: &Device,
        queue: &Queue,
        asset_info: &AssetMetadata,
        asset_data: &[u8],
    ) -> Result<(), TextureUploadError> {
        let view = upload_texture(device, queue, asset_info, asset_data)?;
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("moon_texture_bind_group"),
            layout: &self.texture_layout.0,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sprite_sampler),
                },
            ],
        });
        self.moon_texture_bind_group = Some(bind_group);
        Ok(())
    }

    pub fn update(
        &self,
        queue: &Queue,
        view_proj: [[f32; 4]; 4],
        time_of_day: f32,
    ) {
        let sun_dir = sun_direction(time_of_day);
        let moon_dir = moon_direction(time_of_day);
        let distance = 2000.0f32;
        let sun_pos = [
            sun_dir[0] * distance,
            sun_dir[1] * distance,
            sun_dir[2] * distance,
        ];
        let moon_pos = [
            moon_dir[0] * distance,
            moon_dir[1] * distance,
            moon_dir[2] * distance,
        ];

        tracing::trace!(
            "Sprite update: time={:.2}, sun_dir=({:.2},{:.2},{:.2}), moon_dir=({:.2},{:.2},{:.2})",
            time_of_day, sun_dir[0], sun_dir[1], sun_dir[2],
            moon_dir[0], moon_dir[1], moon_dir[2],
        );

        self.sun_mesh
            .update(queue, sun_pos, 500.0, view_proj);
        self.moon_mesh
            .update(queue, moon_pos, 350.0, view_proj);
    }

    pub fn pass(&self, cmd: &mut CommandEncoder, target_texture_view: &TextureView) {
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

        if let Some(bind_group) = &self.sun_texture_bind_group {
            rpass.set_bind_group(0, &self.sun_mesh.uniform_bind_group, &[]);
            rpass.set_bind_group(1, bind_group, &[]);
            rpass.set_vertex_buffer(0, self.sun_mesh.vertex_buffer.slice(..));
            rpass.set_index_buffer(self.sun_mesh.index_buffer.slice(..), IndexFormat::Uint16);
            rpass.draw_indexed(0..self.sun_mesh.index_count, 0, 0..1);
        }

        if let Some(bind_group) = &self.moon_texture_bind_group {
            rpass.set_bind_group(0, &self.moon_mesh.uniform_bind_group, &[]);
            rpass.set_bind_group(1, bind_group, &[]);
            rpass.set_vertex_buffer(0, self.moon_mesh.vertex_buffer.slice(..));
            rpass.set_index_buffer(self.moon_mesh.index_buffer.slice(..), IndexFormat::Uint16);
            rpass.draw_indexed(0..self.moon_mesh.index_count, 0, 0..1);
        }
    }
}
