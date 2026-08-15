//! The four lighting registers every world pass reads, in one place.
//!
//! The landscape, the static meshes and the repeated meshes all light from the *same* four
//! shader constants, and that is not a coincidence to be tidied away — it is a property of the
//! original. `CalcSWLightingNoClip` (`fableengine/engine_lighting.cpp:2600`) computes
//! `Ambient + saturate(−L·n)²·Diffuse + max(L·n, 0)·Backlight`, which is
//! `VSHADER_STATIC_DIRLIGHT`'s expression over the same registers, not merely a similar one
//! (AGENTS.md §3.13).
//!
//! So the values belong to the *environment*, not to any one pass. AGENTS.md §5 step 2 says
//! the environment layer "lights meshes and terrain in one change" — which is only true if
//! there is one place to make it. This is that place (§12.7).

use bytemuck::{Pod, Zeroable};

/// `c3` / `c19` / `c20` / `c35` in the Lights register layout (AGENTS.md §3.8), laid out to
/// match `struct Lighting` in `lighting.wgsl`.
///
/// Every member is a `vec4`, so this drops into a pass's frame uniform at whatever offset the
/// four registers already occupied — embedding it changes no byte of any buffer.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct LightingUniforms {
    /// `c3` — `Ambient`.
    pub ambient: [f32; 4],
    /// `c19` — light[0]'s direction. Light 0 is the directional light and has no attenuation
    /// (`CShaderRenderManager::InitialiseLight`, `lib_shader_render_manager.cpp:1686`).
    pub light_dir: [f32; 4],
    /// `c20` — light[0]'s colour.
    pub diffuse: [f32; 4],
    /// `c35` — `LightGlobals`, the backlight.
    pub backlight: [f32; 4],
}

impl LightingUniforms {
    /// What every world pass runs with until the environment layer lands (AGENTS.md step 2).
    ///
    /// UNVERIFIED, and deliberately **inert rather than plausible** (§2 rule 5): an `Ambient`
    /// of 0.5 cancels the pixel shaders' `mul_x2` exactly, so surfaces show their textures at
    /// their authored colour with no directional term at all. The mechanism is fully wired —
    /// step 2 replaces these four values with rows 1/0/3 of the environment LUT and changes
    /// nothing else, in one edit rather than three kept in sync by hand.
    pub const NEUTRAL: LightingUniforms = LightingUniforms {
        // UNVERIFIED: neutral stand-in — see above. AGENTS.md §9.
        ambient: [0.5, 0.5, 0.5, 1.0],
        light_dir: [0.0, 0.0, -1.0, 0.0],
        diffuse: [0.0, 0.0, 0.0, 0.0],
        backlight: [0.0, 0.0, 0.0, 0.0],
    };
}

/// The WGSL counterpart, prepended to every shader that reads these registers so the two
/// layouts cannot drift apart.
pub(crate) const LIGHTING_WGSL: &str = include_str!("lighting.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    /// The layout the passes' frame uniforms assume: four `vec4`s, no padding, 64 bytes.
    #[test]
    fn matches_four_vec4_registers() {
        assert_eq!(size_of::<LightingUniforms>(), 64);
        assert_eq!(align_of::<LightingUniforms>(), 4);
    }
}
