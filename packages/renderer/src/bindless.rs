//! One texture array for the whole renderer (AGENTS.md §12).
//!
//! Every pass used to own its texture binding: a `BindGroupLayout` for "one texture + one
//! sampler + a uniform", and a fresh `BindGroup` per material, per layer, per batch. That is
//! the ordinary wgpu pattern and it is not wrong — it just does not scale to one texture per
//! rasterized glyph (§13), and it gives the renderer nowhere to notice it has decoded the same
//! asset before. Measured, between a half and three quarters of a level's texture upload work
//! is a repeat of work already done (§12.1).
//!
//! So there is one `binding_array<texture_2d<f32>, N>` and one bind group, shared by every
//! pass, indexed per draw. A texture is registered by key; registering a key twice returns the
//! same index, which is the whole dedup.
//!
//! **Nothing uses this yet.** It is landed unused, deliberately, so the device features, the
//! limits and the array's validation are proven separately from any behavioural change
//! (§12.8 step 1).

// Step 1 lands the whole registry and migrates no pass, so most of the surface below has no
// caller yet. The allow goes away on its own as §12.8 steps 2–5 use it; if it is still needed
// when step 6 closes, something was built that nothing wanted.
#![allow(dead_code)]

use crate::texture::{linear_clamp_sampler, repeat_sampler};
use derive_more::{Display, Error};
use std::collections::HashMap;
use std::num::NonZeroU32;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Device, Extent3d, Queue,
    SamplerBindingType, ShaderStages, TexelCopyBufferLayout, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
    TextureViewDimension,
};

/// How many textures the array holds.
///
/// **Measured, not chosen** (AGENTS.md §12.3). Against the shipped data, one level's resident
/// set peaks at about 100 textures (LookoutPoint: 70 mesh + 20 ground + 5 blend tables + 2
/// sky), the largest `.wld` region touches 359 distinct meshes, and the whole 402-level world
/// touches 1,331. 4096 covers all of it with headroom, which is the point: binding arrays are
/// sized at layout-creation time and outgrowing one means rebuilding every pipeline built
/// against it, so the capacity is picked to make that impossible rather than to be tight.
///
/// The cost is one descriptor and one cloned `TextureView` handle per slot — on the order of
/// 128 KB — against a hardware ceiling of 1,015,808 on this machine's RADV and 1,000,000 on
/// Metal argument-buffers Tier 2.
pub const MAX_BINDLESS_TEXTURES: u32 = 4096;

/// The smallest array worth starting with: comfortably above §12.3's measured level peak, so
/// an adapter that cannot reach it cannot draw a level either. Metal argument-buffers **Tier
/// 1** reports 128 or even 31 elements and lands here.
pub const MIN_BINDLESS_TEXTURES: u32 = 256;

/// Which texture a slot holds. Registering the same key twice returns the same slot.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TextureKey {
    /// A global asset id, as `Material::base_texture_id` and `Graphic.BankIndex` give it
    /// (AGENTS.md §3.11). The same id from two meshes is one texture.
    Asset(u32),
    /// A table the renderer builds rather than reads — the five landscape direction blend
    /// tables (§3.4), keyed by `MappingDirection::index()`.
    BlendTable(u32),
    /// Sky texture slot 0 or 1. Resolved by name per keyframe rather than by asset id, so it
    /// gets its own key space until that path is asset-id-driven.
    Sky(u32),
}

/// A slot in the bindless array.
///
/// **Valid only until the next [`BindlessTextures::clear`]**, which is called on scene load.
/// That is the same lifetime every `GpuModel` holding one already has, so the index stays a
/// bare `u32` rather than carrying a generation (§12.4).
pub type BindlessIndex = u32;

#[derive(Debug, Display, Error)]
#[display("bindless texture array is full ({capacity} slots)")]
pub struct BindlessFull {
    pub capacity: u32,
}

pub struct BindlessTextures {
    /// Length `capacity`. Every slot is always a valid view — unregistered ones hold
    /// [`Self::fallback`] — so the bind group is always *fully* bound and
    /// `PARTIALLY_BOUND_BINDING_ARRAY`, which Metal does not support, is never needed.
    slots: Vec<TextureView>,
    fallback: TextureView,
    index_of: HashMap<TextureKey, BindlessIndex>,
    next_free: u32,
    capacity: u32,
    /// Set by `register`, cleared by `rebuild_if_dirty`. Registration never touches the GPU;
    /// the bind group is rebuilt at controlled sync points instead, because a `BindGroup` is
    /// immutable and every new texture means rebuilding the whole thing (§12.2).
    dirty: bool,
    layout: BindGroupLayout,
    bind_group: BindGroup,
    /// Wrap + trilinear + anisotropy 4, for anything projected onto a surface.
    repeat_sampler: wgpu::Sampler,
    /// Clamp, no mip, no anisotropy, for anything that is a lookup rather than a surface.
    clamp_sampler: wgpu::Sampler,
}

impl BindlessTextures {
    /// The capacity to build for on `adapter`: [`MAX_BINDLESS_TEXTURES`], or what the adapter
    /// allows if that is less. `None` when the adapter cannot reach
    /// [`MIN_BINDLESS_TEXTURES`], which is a refusal rather than a silent downgrade — a
    /// smaller array than a level needs would fail later, at a draw, instead of here.
    pub fn capacity_for(adapter_limit: u32) -> Option<u32> {
        let capacity = MAX_BINDLESS_TEXTURES.min(adapter_limit);
        (capacity >= MIN_BINDLESS_TEXTURES).then_some(capacity)
    }

    pub fn new(device: &Device, queue: &Queue, capacity: u32) -> Self {
        let fallback = create_fallback_view(device, queue);
        let slots = vec![fallback.clone(); capacity as usize];

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bindless_textures"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        // Every texture the renderer uploads is a filterable float 2D
                        // texture — Bc1/Bc2/Bc3, Rgba8Unorm and R8Unorm all qualify, and so
                        // do §13's R8 glyph bitmaps. Differing formats, sizes and mip counts
                        // between entries are fine; a differing *sample type* or view
                        // dimension is not, and would need its own binding rather than this
                        // array (AGENTS.md §12.2).
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(capacity),
                },
                // Two ordinary samplers, deliberately not a `binding_array<sampler, N>`:
                // sampler choice is per pass and statically known in each shader, and these
                // two are exactly the set the four existing passes already use — the model,
                // local-detail and terrain-ground samplers are all `repeat_sampler`, and the
                // sky and terrain-blend samplers are both `linear_clamp_sampler`. An array
                // would buy nothing and cost a second limit and Metal's much lower
                // sampler-array ceilings (§12.4).
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let repeat_sampler = repeat_sampler(device, "bindless_repeat_sampler");
        let clamp_sampler = linear_clamp_sampler(device, "bindless_clamp_sampler");
        let bind_group = build_bind_group(
            device,
            &layout,
            &slots,
            &repeat_sampler,
            &clamp_sampler,
        );

        tracing::info!("Bindless textures: {capacity} slots");

        Self {
            slots,
            fallback,
            index_of: HashMap::new(),
            next_free: 0,
            capacity,
            dirty: false,
            layout,
            bind_group,
            repeat_sampler,
            clamp_sampler,
        }
    }

    /// The layout every pipeline that draws through the array is built against.
    pub fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }

    /// Give `key` a slot, or return the one it already has.
    ///
    /// Touches no GPU state — the caller batches registrations and calls
    /// [`Self::rebuild_if_dirty`] once (§12.2, §13.3).
    pub fn register(
        &mut self,
        key: TextureKey,
        view: TextureView,
    ) -> Result<BindlessIndex, BindlessFull> {
        if let Some(&index) = self.index_of.get(&key) {
            return Ok(index);
        }
        if self.next_free >= self.capacity {
            return Err(BindlessFull {
                capacity: self.capacity,
            });
        }

        let index = self.next_free;
        self.next_free += 1;
        self.slots[index as usize] = view;
        self.index_of.insert(key, index);
        self.dirty = true;
        Ok(index)
    }

    pub fn index_of(&self, key: TextureKey) -> Option<BindlessIndex> {
        self.index_of.get(&key).copied()
    }

    /// Drop every registration and release the textures with it.
    ///
    /// **Every [`BindlessIndex`] handed out before this call is stale afterwards.** The
    /// registry is level-scoped by design (§12.4): it is cleared alongside the models it holds
    /// textures for, so the indices die with the `GpuModel`s that carry them. Without this a
    /// scene load would leak every previous level's textures *and* burn their slots.
    pub fn clear(&mut self) {
        if self.next_free == 0 {
            return;
        }
        for slot in &mut self.slots[..self.next_free as usize] {
            *slot = self.fallback.clone();
        }
        self.index_of.clear();
        self.next_free = 0;
        self.dirty = true;
    }

    /// Rebuild the bind group if anything was registered since the last call.
    ///
    /// A `BindGroup` is immutable, so this is the only way a new texture becomes visible to a
    /// shader. It is cheap — `capacity` view handles, not `capacity` copies of data — but not
    /// free, which is why `register` does not do it.
    pub fn rebuild_if_dirty(&mut self, device: &Device) -> &BindGroup {
        if self.dirty {
            self.bind_group = build_bind_group(
                device,
                &self.layout,
                &self.slots,
                &self.repeat_sampler,
                &self.clamp_sampler,
            );
            self.dirty = false;
            tracing::debug!(
                "Bindless bind group rebuilt: {}/{} slots registered",
                self.next_free,
                self.capacity,
            );
        }
        &self.bind_group
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    /// Slots handed out so far. `(registered, capacity)`.
    pub fn stats(&self) -> (u32, u32) {
        (self.next_free, self.capacity)
    }
}

fn build_bind_group(
    device: &Device,
    layout: &BindGroupLayout,
    slots: &[TextureView],
    repeat: &wgpu::Sampler,
    clamp: &wgpu::Sampler,
) -> BindGroup {
    let views: Vec<&TextureView> = slots.iter().collect();
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("bindless_textures"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureViewArray(&views),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(repeat),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::Sampler(clamp),
            },
        ],
    })
}

/// The 1×1 opaque white texture every unregistered slot points at.
///
/// White rather than transparent or magenta on purpose: it is also what a material with no
/// diffuse map samples, and roughly a quarter of `graphics.big`'s materials have none
/// (AGENTS.md §3.11), so "no texture" draws the material's own colour rather than a hole.
fn create_fallback_view(device: &Device, queue: &Queue) -> TextureView {
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("bindless_fallback_white"),
        size: Extent3d {
            width: 1,
            height: 1,
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
        &[255, 255, 255, 255],
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: None,
        },
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    texture.create_view(&TextureViewDescriptor::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_clamps_to_the_adapter_and_refuses_below_the_floor() {
        // The normal case: the adapter allows far more than we want.
        assert_eq!(BindlessTextures::capacity_for(1_015_808), Some(4096));
        // A tighter adapter gets what it has, so long as a level still fits.
        assert_eq!(BindlessTextures::capacity_for(512), Some(512));
        assert_eq!(
            BindlessTextures::capacity_for(MIN_BINDLESS_TEXTURES),
            Some(MIN_BINDLESS_TEXTURES)
        );
        // Metal argument-buffers Tier 1 reports 128 or 31. Refused, rather than downgraded
        // into a failure at some later draw.
        assert_eq!(BindlessTextures::capacity_for(128), None);
        assert_eq!(BindlessTextures::capacity_for(0), None);
    }
}
