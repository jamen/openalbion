# OpenAlbion — agent guide & renderer plan

> **Note (2026-08-05):** the previous `AGENTS.md` was never committed and is gone from disk.
> This is a fresh, renderer-focused reconstitution. The def-compiler/architecture chapters of
> the old document were not recovered; re-add them here when that work resumes.

---

## 0. Where we are

Two renderer subsystems have been attempted — **sky** and **landscape**. Both render
*something*, neither renders *accurately*. The last ~12 commits are a tight loop of
`fix:` / `diagnostic:` / `debug:` on visual symptoms:

```
27d7fda debug: dump multiple LUT rows at once to verify indexing
25a8dcc diagnostic: log LUT colours each frame, clamp gradient alpha to 0.4
370aa0a fix: default LUT rows to 13-16 when ENVIRONMENT def returns zero
48ffc56 fix: LUT linear interpolation, dome top cap, production shader
23c2150 fix: rebuild dome as triangle-strip cylinder with proper UV wrap
a53d300 debug: UV visualisation shader, fan-only dome mesh
```

**Root cause of the loop:** we have been tuning *constants* inside an *architecture we
guessed at*, and validating by eye. When the picture still looks wrong there is no way to
tell whether the mesh, the data lookup, the shader, or the colour space is at fault — so
the next commit guesses again.

Meanwhile we already own four oracles that answer these questions exactly (§1). We were
not using them.

**The fix is procedural, not technical:** derive → verify → implement → review, one
mechanism at a time, with the ground truth quoted in the commit. §2 is the rules, §6 is the
tooling that makes off-screen verification possible, §7 is the review protocol.

---

## 1. The oracles

Ranked by authority. Higher entries override lower ones. Never guess when an oracle covers
the question.

| # | Resource | What it settles | Notes |
|---|---|---|---|
| 1 | `~/git/fable-reimpl/src/**/*.hpp` | Class layouts, field names, offsets, method signatures, **shader constant register maps** | Generated from CodeView debug info. **Authoritative** — if a decompiled body contradicts a header, the header wins. |
| 2 | `~/dl/Fable Shader Disassembly.md` | Exact per-vertex/per-pixel maths | 465 shaders, complete. Grouped by `## SHADERS_*`, each in a `<details><summary><strong>NAME</strong>` block. Provenance unverified — see §5 step 1. |
| 3 | `~/doc/.../Fable/Data/shaders/*.bbb` | The shaders as the game actually ships them: name, bytecode, source path | Compiled banks. Format decoded in §3.7. Lets us regenerate #2 ourselves. |
| 4 | `~/doc/.../Fable/Data/Defs/*.def` | Real authored data (rows, columns, textures, tuning) | Debug build ships text `.def`; retail ships only `CompiledDefs/*.bin`. |
| 5 | `~/git/fable-reimpl/src/**/*.cpp` | Algorithms, control flow, constants | Ghidra output — noisy. Trust literals and structure; be sceptical of types and register artifacts. |
| 6 | `~/git/fable-decomp` | Raw decomp inputs / sidecars | For when #5 is unreadable. |

Key fixture paths:

```
~/doc/Fable_Anniversary-2013-02-25/Fable/
  Data/Defs/environment.def                 # ENVIRONMENT + 38× ENVIRONMENT_THEME_DAY, as text
  Data/LightingTable/lighting_colours.tga   # 190 × 21, 24-bit — the colour LUT
  Data/shaders/{vs_*,ps_*,x*}.{bbb,dep,cks} # 51 shader banks + dep/checksum sidecars
  Data/CompiledDefs/{names.bin,game.bin}
  Data/Levels/FinalAlbion.wad
  Data/graphics/{graphics.big,pc/textures.big}
  Ego_d.exe + Ego_d.pdb                     # debug build + symbols
```

Renderer files worth knowing in the decomp:

```
fableengine/engine_sky_renderer.{hpp,cpp}        # CEngineSkyRenderer — the whole sky
fableengine/engine_environment.{hpp,cpp}         # engine-side environment component
fableengine/engine_landscape*.{hpp,cpp}          # 20 files: patches, layers, tesselation, LOD
fableengine/engine_surface_composition_manager.* # composited landscape surface textures
fableengine/engine_vs_layout_*.{hpp,cpp}         # ← shader constant register maps (§3.8)
bbblibrary/lib_shader_constant_register_layout.* # ← the named register slots
fablelib/environment_theme.{hpp,cpp}             # CBlendedEnvironmentTheme, lookup texture
fablelib/defs/environment_def.hpp                # CEnvironmentDef / CEnvironmentThemeDef
fablelib/defs/engine_sky_def.hpp                 # CSkyDef
```

---

## 2. Ground rules

1. **No invented constants.** Every magic number in the renderer must be traceable to a def
   field, a decomp literal, or a named shader constant register — cited in a comment. If a
   value genuinely cannot be sourced, mark it `// UNVERIFIED:` and log it in §9.
2. **One mechanism per change.** A commit implements one thing (a lookup, a mesh, a shader)
   and names its oracle in the message.
3. **Verify before rendering.** Data-layer correctness is checked with a test or a
   value-dump command — *not* by looking at the screen.
4. **The screen is the last check, never the first.** If the only evidence a change is right
   is "it looks better", it is not verified.
5. **Delete rather than tune.** A wrong mechanism tuned to look plausible is worse than an
   obviously missing one. Strip it.
6. **Jamen reviews the derivation before implementation** on each numbered step. §7.
7. **Coordinates are Z-up.** Settled — see §3.6.

---

## 3. Verified ground truth

Everything here was read out of the oracles during the 2026-08-05 review. Citations are exact.

### 3.1 The environment colour LUT — *this is the big one*

`Data/LightingTable/lighting_colours.tga` is **190 × 21, 24-bit RGB** (12014 bytes = 18-byte
header + 190·21·3 + 26-byte TGA-2.0 footer). There is **no alpha channel**.

**Rows** come from `CEnvironmentDef` (`fablelib/defs/environment_def.hpp:874-918`), and
`environment.def` assigns them explicitly:

```
DiffuseLookupRow 0   AmbientLookupRow 1   CloudColourLookupRow 2   BacklightLookupRow 3
ReflectionLookupRow 4   MistEffectColourLookupRow 5   FogColourLookupRow 6
FogAlphaLookupRow 7   SunColourLookupRow 8   MoonColourLookupRow 9   StarsColourLookupRow 10
SunFlareColourLookupRow 11   LensFlareColourLookupRow 12
SkyGradientTopLookupRow 13   SkyGradientTopAlphaLookupRow 14
SkyGradientBottomLookupRow 15   SkyGradientBottomAlphaLookupRow 16
WaterColourLookupRow 17   SeaColourLookupRow 18
GlowThresholdColourLookupRow 19   GlowBloomColourLookupRow 20
```

21 rows, 0–20 — matches the texture height exactly. The `*AlphaLookupRow` rows are ordinary
RGB rows whose **R channel carries the alpha scalar**; the current code is right about that.

**Columns are NOT time-of-day.** From `fablelib/environment_theme.cpp:1862`
(`CEnvironmentThemeSetDay::Build`):

```c
BuildFromSource(..., theme_day_set_def->ColourLookupColumn + iVar2);   // iVar2 = keyframe index
```

and `BuildFromSource` (`:1448`) does, per property:

```c
LookupFloatColour(lookup_texture, &local_10, colour_lookup_x, environment_def->SkyGradientTopLookupRow)
```

`CEnvironmentLookupTexture::LookupFloatColour(long, long)` takes **two integers** — an
unfiltered texel fetch, not a sampled lookup.

> **column = `ENVIRONMENT_THEME_DAY.ColourLookupColumn` + keyframe index**

`environment.def` confirms it: 38 themes with columns 0, 10, 14, 18, 24, 32, 36, 55, 65, 79,
89, 93, 97, 101, 105, 109, 119, 129, 139, 143, 147, 157, 167, 176, 178, 182, 186 … each
theme owning a contiguous run sized to its keyframe count (2–8), the highest reaching
186 + keyframes ≈ 190. **Every theme owns its own slice of the texture.**

Time interpolation happens *after* the fetch:
`CEnvironmentThemeSetDay::GetBlendedTheme(result, time)` (`:1873`) finds the two keyframes
bracketing `time` and calls `CBlendedEnvironmentTheme::Blend(a, b, t)` over the whole
already-fetched colour set.

**Current code is wrong here.** `renderer/sky.rs:728` computes `fx = time_of_day / 24.0 *
width` and bilinearly interpolates along X — it walks across *other themes'* columns as the
clock advances and filters between unrelated themes. No row-index tweak or `* 0.4` clamp can
rescue that. This single bug accounts for most of the sky-colour thrash.

`CBlendedEnvironmentTheme::Colours[]` (`fablelib/environment_theme.hpp:873`) is indexed by a
stable enum, *not* by LUT row — read off `BuildFromSource`:

```
0 Diffuse  1 Ambient  2 Backlight  3 Reflection  4 MistEffect  5 FogColour
6 SunColour  7 CloudColour  8 MoonColour  9 StarsColour  10 SunFlare  11 LensFlare
12 SkyGradientTop  13 SkyGradientBottom  …
```

`ColourLookupColumn` is **already modelled** in
`~/git/fable-defs/packages/defs/src/def/environment_theme_day_set.rs:21` — the renderer just
never reads it.

### 3.2 Outer sky — geometry

`CEngineSkyRenderer::BuildOuterSkyMesh` (`engine_sky_renderer.cpp:545`), read literally:

- Apex vertex at `(0, 0, 7000)`, `DiffuseColour = 0`, UV `(-ε, -ε)` (`DELTA_NOTIONAL_ZERO`).
- 36 iterations (`uVar13 < 0x24`), each emitting **a pair**:
  - bottom: `(cos·6500, sin·6500, -500)`, `V1 = 1.0`, `DiffuseColour = 0xffffffff`
  - top:    `(cos·6500, sin·6500,  7000)`, `V1 = 0.0`, `DiffuseColour = 0`
  - `U1` starts at 0 and advances by `2.857142873108387e-2` = **1/35** per pair.
- Indices: a 36-triangle fan for the top cap, then a 36-quad wall, the final quad wrapping
  back to the first vertex pair.

It is a **cylinder with a flat cap**, not a hemisphere. Height is in **Z**. The U step of
1/35 over 36 pairs means U reaches exactly 1.0 on the last pair and the wrap quad spans
1.0 → 0.0 — that seam is original, not a bug to "fix".

Our `build_outer_sky_mesh` (`sky.rs:51`) already has 7000 / −500 / 6500 and the right vertex
colours, but parameterises U as `i/36`.

`BuildBaseBandMesh` (`:666`): centre vertex at `(0, 0, -10000)`, `DiffuseColour = 0`, plus a
36-vertex ring at `(cos·6500, sin·6500, -500)`, `DiffuseColour = 0xffffffff`, all UVs zero.
Current `build_base_band_mesh` matches.

### 3.3 Outer sky — shaders (exact)

`VSHADER_OUTER_SKY`:

```asm
dp4 oPos.{x,y,z,w}, v0, c5..c8      ; CombinedProjectionMatrix
mul r0, v1, c93                     ; v1 = vertex diffuse, c93 = bottom gradient
add r1, c0.y, -v1                   ; c0.y = 1.0 (preset)
mul r1, r1, c92                     ; c92 = top gradient
add oD0, r0, r1                     ; oD0 = lerp(top, bottom, vertexColour)
mov oT0, v2
mov oT1, v2                         ; both stages sample the SAME uv
```

`PSHADER_OUTER_SKY`:

```asm
tex t0                              ; sky texture 0
tex t1                              ; sky texture 1
mov_sat r0, c0                      ; c0.w = the sky-texture blend factor
mov_sat r1, v0                      ; r1.w = saturate(oD0.a)
lrp r0, r0.w, t1, t0                ; lerp(tex0, tex1, saturate(c0.w))
lrp r0, r1.w, v0, r0                ; lerp(that, gradient, saturate(gradient.a))
```

(D3D `lrp dst, s0, s1, s2` = `s2 + s0·(s1 − s2)` = `lerp(s2, s1, s0)`. Note pixel-shader `c0`
is a *pixel* constant and unrelated to the vertex `c0` preset.)

Against `renderer/sky/outer_sky.wgsl`:

- ✅ vertex-side gradient lerp is correct.
- ❌ `mix(tex0, tex1, 0.5)` — the blend factor is **`c0.w`**, i.e.
  `CEnvironmentThemeDef::SkyTexture1Blend` (and the keyframe blend), not `0.5`.
- ❌ the `* 0.4` clamp on gradient alpha in `sky.rs:768` has no counterpart in the shader.
- ❌ the header comment claims "TEMPORARY DEBUG MODE: outputs UV coordinates as colours" —
  stale, the body does not do that. Actively misleading.

`VSHADER_SKY_BASE_BAND` is **`mov oD0, c92`** — flat top-gradient colour, no texture, no
vertex colour. We currently draw the base band through the *outer sky* pipeline, which
samples sky textures. Wrong shader.

`VSHADER_SKY_SPRITE` is `mov oD0, v1; mov oT0, v2` + the standard transform — matches our
`sky_sprite.wgsl`.

### 3.4 Landscape — the architecture is different from ours

`CLandscapeLayerMesh::CVertex` (`engine_landscape_layer_mesh.hpp:71`):

```cpp
unsigned char X;      // grid position within patch
unsigned char Y;
unsigned char Blend;  // this layer's alpha at this vertex
unsigned char CliffU; // texture coords
unsigned char CliffV;
```

The layer itself carries `ForegroundTextureIndex`, `BackgroundTextureIndex`,
`BumpMapTextureIndex`, `SelfIllumination`, `MappingDirection`
(`LANDSCAPE_TEXTURE_MAPPING_DIRECTION`), its own index/vertex buffers, and a `Next`
pointer — **a linked list of layers per patch**.

`VSHADER_LANDSCAPE_FOREGROUND`:

```asm
mov r0.xy, v0        ; v0.xy = XY position
mov r0.z,  v1.x      ; v1.x  = height        → position split across two streams
mov r0.w,  c0.y      ; 1.0 (preset)
add r1, r0, -c4      ; c4 = CameraPos — geometry is CAMERA-RELATIVE
dp4 oPos.{x..w}, r1, c5..c8

dp3 r4, v2, -c19     ; v2 = normal, c19 = light direction
max r4.x, r4.x, c0.x ; saturate positive part      (c0.x = 0.0)
min r4.y, r4.y, c0.x ; keep negative part
mul r4.x, r4.x, r4.x ; SQUARED n·l
mul r3, r4.x, c20            ; × diffuse colour
mad r3, -r4.y, c35, r3       ; + backlight colour × (−n·l)
add r3, r3, c3               ; + Ambient
mov oD0.xyz, r3

mov oT0.xy, v3.yzzz  ; UV = (v3.y, v3.z) = (CliffU, CliffV)
mul oD0.w, r4, v3.x  ; ALPHA = v3.x = Blend, × a distance fade
dp4 r5.x, r0, c40    ; oT1 = planar projection of world position
dp4 r5.y, r0, c41    ;      → the composited surface texture
```

`PSHADER_LANDSCAPE_FOREGROUND`:

```asm
tex t0                          ; layer texture, sampled at (CliffU, CliffV)
tex t1                          ; composited surface, sampled at the planar projection
mul_x2_sat r0.xyz, t1, v0       ; colour = surface × light × 2
mul_sat    r0.w,   t0.w, v0.w   ; alpha  = layerTexture.a × blend
```

So the real pipeline is:

1. A **background pass** (`PSHADER_LANDSCAPE_BACKGROUND`: `t0 × v0 ×2`).
2. **N alpha-blended foreground layer passes**, one per theme layer in the patch. The layer
   texture supplies the *mask* (alpha); the *colour* comes from the composited surface
   texture (`CEngineSurfaceCompositionManager`) via planar projection.
3. Lighting is `Ambient + saturate(n·l)²·Diffuse + max(−n·l,0)·Backlight`, all three colours
   from the environment LUT (rows 1, 0, 3).
4. UVs are **per-vertex `CliffU`/`CliffV`**, chosen by the layer's `MappingDirection` — *not*
   derived from world XZ, and *not* a slope-driven blend.

`renderer/terrain.wgsl` currently does a single pass with a 3-way `mix()` over
`theme_indices`, a hardcoded `light_dir = (0.4, 1.0, 0.3)`, `shade = 0.25 + diffuse*0.75`,
and an invented `slope_f = clamp((1 - normal.y)*6 - 0.5, 0, 1)` cliff blend. None of those
five mechanisms exists in the original. `terrain.rs:93` likewise invents `cliff_u` from an
`atan` of the height gradient.

The `LEV` side is fine: `LevHeightCell` already carries `ground_theme: (u8,u8,u8)` and
`ground_theme_strength: (u8,u8)` (`fable-data/src/lev.rs:196`) — three layers with two
weights, which maps directly onto per-layer `Blend`.

### 3.5 Colour space — a systemic bug affecting everything

- Textures upload as `Bc1/2/3RgbaUnorm` (`renderer/texture.rs:31`) — **linear**, no sRGB decode.
- The render target view is created with `format.add_srgb_suffix()` (`renderer.rs:210`) —
  **sRGB**, so the GPU applies a linear→sRGB *encode* on write.

Every colour is therefore encode-without-decode: brightened and desaturated in the shadows.
The original is D3D9 with no sRGB framebuffer and raw texture sampling.

**The faithful choice is to drop `add_srgb_suffix()`** and keep `Unorm` textures — sample raw,
write raw, exactly like the original. (The "modern-correct" alternative — `UnormSrgb` textures
+ linear math + sRGB target — would look *different* from the game and is the wrong goal.)
One line, changes sky/terrain/models simultaneously, so it must land *before* any colour
tuning or every subsequent judgement is made through a broken filter.

### 3.6 Coordinate system — **settled: Z-up**

The game is Z-up: `BuildOuterSkyMesh` puts the dome apex at `Z = 7000`, and
`VSHADER_LANDSCAPE_FOREGROUND` builds position as `(v0.x, v0.y, v1.x)` with height third.

**Decision (2026-08-05, Jamen): keep the game's native Z-up in every mesh, def value and
data-layer coordinate. Convert exactly once, in the view matrix.** Per-subsystem conversion
(as in `dedca35 "fix: convert sky dome/sprite coords from Z-up to Y-up"`) is a permanent
source of sign and axis bugs and makes decomp constants un-copy-pasteable.

Practically: `camera.rs` builds the view matrix with `up = +Z`; `glam::Mat4::perspective_rh`
still applies; the wgpu NDC handedness fix stays where it already is. Nothing else flips.

### 3.7 The `.bbb` shader banks — investigation result

Jamen's instinct was half right: `.bbb` **is** a Big-Blue-Box bank container, not text. But
these particular banks contain compiled shaders. Format, decoded from `vs_format.bbb` (251
bytes, the smallest):

```
00  "BBBB"                       magic
04  u32 0x73                     \ header/section sizes
08  u32 0x6b                     /
0c  u32 1                        entry count
10  u32 4, u32 1, u32 1
1c  u32 0x4c                     offset to ...
20  u32 0x20                     bytecode length (32 bytes)
30  01 01 fe ff                  ← D3D token 0xfffe0101 = vs_1_1
    1f 00 00 00 05 00 00 80 …    dcl / mov instructions
    ff ff 00 00                  END token
70  u32 19  "VERTEX_FORMAT_DUMMY"        ← length-prefixed SHADER NAME
    u32 hash
    u32 100 "\FableTLC\…\FableEngine\shaders\vertex\dummy.vsh"   ← SOURCE PATH
```

51 banks: 25 `vs_*`/`ps_pixel` (PC) plus `x`-prefixed Xbox twins. Sidecars: `.dep` = a
`BDEP`-magic dependency list, `.cks` = checksums.

**No `CTAB`** anywhere in the banks — the shaders were assembled from `.vsh`/`.psh` asm, not
compiled by `fxc` with a constant table, so the bytecode carries **no named constants**.

But the `.dep` files name the includes the asm used:

```
vertex_shader_config.h
vertex_shader_constants_basic.h
vertex_shader_constants_lights.h
vertex_shader_constants_lights_bones.h
```

Those headers don't ship — **but their C++ counterparts are in the decomp**, and they carry
the same register assignments. That is §3.8, and it is the real prize.

**What the banks are still worth:**
- They let us regenerate the disassembly ourselves and confirm
  `~/dl/Fable Shader Disassembly.md` is complete and faithful (its provenance is unknown).
- They give the authoritative `NAME → bytecode → source path` mapping, so a shader's group
  and role are unambiguous.
- The source paths reveal the engine's own shader taxonomy (`shaders/vertex/*.vsh`).

**Verdict:** worth a small extractor (§5 step 1) — half a day, and it de-risks oracle #2 for
every subsequent subsystem. Not worth more than that.

### 3.8 The shader constant register map — named, with exact numbers

`bbblibrary/lib_shader_constant_register_layout.hpp` declares
`CVertexShaderConstantLayout` as a **named register map**:

```
Reserved  User  Compression  Ambient  FogColour  CameraPos
LightSize  LightArray  LightGlobals  LightAttenuations
CombinedProjectionMatrix  FogTransform  WorldMatrix  SphereMapMatrix
TextureTransform[4]  BoneMatrices
ShadowTextureTransform  ShadowFadeTransform  ShadowFadeFactors
ShadowedSpotlightFalloffTransform  ShadowedSpotlightAttenuation  ShadowedSpotlightColour
ScreenSpaceTransformScale  ScreenSpaceTransformOffset
ReservedPresets: CArray<CVertexShaderConstant>
```

Each is a `CShaderConstantRange { Offset, Count }`. The base constructor zeroes them; each
engine subclass fills in the real numbers. From `engine_vs_layout_basic.cpp:109-182`:

| Register(s) | Name | Source |
|---|---|---|
| `c0` | preset `(0.0, 1.0, 2.0, 0.5)` | `engine_vs_layout_basic.cpp` presets |
| `c1` | preset `(256, 256, 256, 256)` | ″ |
| `c2` | `FogTransform` | `.Offset = 2, .Count = 1` |
| `c3` | `Ambient` | `.Offset = 3, .Count = 1` |
| `c4` | `CameraPos` | `.Offset = 4, .Count = 1` |
| `c5`–`c8` | `CombinedProjectionMatrix` | `.Offset = 5, .Count = 4` |
| `c9`–`c10` | `SphereMapMatrix` | `.Offset = 9, .Count = 2` |
| `c11`–`c14` | `ShadowTextureTransform` | `.Offset = 0xb, .Count = 4` |
| `c15` | `ShadowFadeTransform` | `.Offset = 0xf` |
| `c16` | `ShadowFadeFactors` | `.Offset = 0x10` |
| `c18` | `FogColour` | `.Offset = 0x12, .Count = 1` |
| `c19`–`c95` | `User` | `.Offset = 0x13, .Count = 0x4d` |

**This cross-verifies against the disassembly on six registers independently.** In
`VSHADER_LANDSCAPE_FOREGROUND`: `c0.x`=0.0 and `c0.y`=1.0 used as clamp bounds ✓;
`c4` subtracted from position ✓ CameraPos; `c5..c8` the transform ✓; `c3` added as the base
lighting term ✓ Ambient; `c2` feeding `oFog` ✓ FogTransform; `c18.w` in the fog `mad` ✓
FogColour. In `VSHADER_OUTER_SKY`: `c0.y` = 1.0 ✓, `c92`/`c93` land inside `User` ✓.

Two independent oracles agreeing on six registers is exactly the standard of evidence §2
asks for. **We are no longer guessing at any of `c0`–`c18`.**

The `Lights` subclass (`engine_vs_layout_lights.cpp`) overrides for the landscape/mesh
shaders: `LightArray.Count = 0xc` with `LightSize = 2` (→ 6 lights × 2 registers),
`LightGlobals.Count = 1`, `ShadowedSpotlightFalloffTransform` at `c15`–`c17`,
`ShadowedSpotlightAttenuation` at `c36`, `ShadowedSpotlightColour` at `c37`,
`User` at `c38`–`c95`.

**Open, and the first task of step 2:** the exact `LightArray` / `LightGlobals` /
`LightAttenuations` offsets. The strong hypothesis is `LightArray.Offset = 19`, giving
`c19` = light[0] direction and `c20` = light[0] colour (matching the `dp3 v2, -c19` /
`mul r3, r4.x, c20` pair exactly), with `c35` the backlight global. Read the full
constructor body to confirm — do not assume.

---

## 4. Assessment of the current renderer

`packages/openalbion/src/renderer/` — 4686 lines.

| Area | Verdict |
|---|---|
| `renderer.rs` pass/pipeline plumbing, `texture.rs`, `depth.rs` | **Keep.** Structurally fine. |
| Sky dome & base-band mesh geometry | **Keep**, fix U step to 1/35, interleave to match. |
| Outer sky vertex-side gradient lerp | **Keep** — matches `VSHADER_OUTER_SKY`. |
| `sky.rs:728 lut_lookup` (time→column, bilinear) | **Delete.** Wrong axis, wrong filtering (§3.1). |
| `sky.rs:768` `* 0.4` alpha clamp | **Delete.** Pure fudge. |
| `sky.rs:771` per-frame `tracing::info!` row dump | **Delete.** |
| `outer_sky.wgsl` `mix(tex0, tex1, 0.5)` | **Replace** with the real blend constant. |
| `outer_sky.wgsl` stale "DEBUG MODE" header | **Delete.** Actively misleading. |
| `sky.rs:842 sun_direction` / `moon_direction` | **Delete.** Invented orbit maths; replace from `RenderSun`/`RenderMoon` (`engine_sky_renderer.cpp:1617`/`:1812`). |
| Base band drawn through the outer-sky pipeline | **Fix.** Needs its own `mov oD0, c92` shader. |
| `sky/inner_sky.wgsl`, `sky_base_band.wgsl`, `sky_screen_space_sprite.wgsl`, `sky_star_field.wgsl` | **Delete.** All four are 0 bytes. |
| `terrain.wgsl` 3-way `mix`, hardcoded `light_dir`, `slope_f` | **Delete.** No counterpart in the original (§3.4). |
| `terrain.rs:93` `cliff_u` from `atan` of gradient | **Delete.** |
| `terrain.rs` `HEIGHT_SCALE = 2048.0`, `CELL_SIZE = 1.0` | **Mark `UNVERIFIED`.** |
| `files.rs load_lut_rows` / `load_sky_def` | **Refactor.** Each re-opens and re-parses `names.bin` + `game.bin` per call. |
| `Renderer::set_lut_rows(top, top_alpha, bottom, bottom_alpha)` | **Delete.** Signature encodes the wrong model. |

Note the pattern: in both subsystems the geometry is roughly right and the **data lookup is
invented**. That is where the effort goes.

---

## 5. The plan

Each step: derivation posted for review → implementation → verification artefact → commit
citing the oracle.

### Step 0 — Strip back and level the ground *(no new features)*

- 0.1 Remove everything in the "Delete" column of §4.
- 0.2 Drop `add_srgb_suffix()` from the surface view (§3.5). **Its own commit**, so the
  visual delta is attributable.
- 0.3 Convert to native Z-up (§3.6): view matrix `up = +Z`, remove per-subsystem flips.
- 0.4 Mark surviving unsourced constants `// UNVERIFIED:` and list them in §9.

*Exit:* builds clean; sky is an untextured gradient or nothing; terrain is untextured-lit.
Ugly is fine — nothing invented remains.

### Step 1 — Tooling and oracle hardening *(before more rendering)*

This is the step that stops the loop from recurring. See §6 for the design of each.

- 1.1 `fool shaderbank` — parse `.bbb`, list `name / version / source path`, disassemble
  bytecode. Diff the output against `~/dl/Fable Shader Disassembly.md` to certify oracle #2.
- 1.2 `openalbion shot` — **headless deterministic capture**. Fixed camera, fixed time, no
  window, renders offscreen and writes a PNG.
- 1.3 `openalbion probe` — dump every uniform/constant fed to each pass for one frame, each
  value tagged with its provenance (`SkyGradientTop ← lighting_colours.tga (col 4, row 13)`).
- 1.4 Golden-image test harness over 1.2, with a perceptual-diff threshold.
- 1.5 Complete the constant register map (§3.8 open item) and commit it as a Rust module of
  named constants, so shaders reference `regs::AMBIENT` not `c3`.

*Exit:* we can produce a reproducible PNG and a reproducible value dump from a single
command, and the shader oracle is self-verified.

### Step 2 — The environment layer *(data only, no rendering)*

A faithful port of `CEnvironmentThemeSetDay` / `CBlendedEnvironmentTheme` into
`fable-data/src/environment.rs`:

- 2.1 `LookupTexture` — load `lighting_colours.tga`, expose `fetch(column, row) -> [f32; 3]`.
  **Integer fetch, no filtering** (§3.1).
- 2.2 `EnvironmentRows` — all 21 `*LookupRow` fields from the `ENVIRONMENT` def.
- 2.3 `BlendedTheme { time_of_day, colours, scalers, texture_sets }` with the `Colours[]`
  ordering from `BuildFromSource`.
- 2.4 `ThemeSetDay::build(...)` — one `BlendedTheme` per keyframe at column
  `ColourLookupColumn + keyframe_index`.
- 2.5 `ThemeSetDay::blended_at(time)` — bracket and lerp, mirroring `GetBlendedTheme`.

*Verification:* `openalbion probe --theme ENVIRONMENT_THEME1 --time 6.0` prints every colour
with provenance; unit tests pin a few times against texels read straight out of the TGA.
**Checked numerically, never on screen.**

### Step 3 — Outer sky, faithfully

- 3.1 Rebuild the mesh to match `BuildOuterSkyMesh` exactly (interleaved pairs, U step 1/35,
  Z-up). Assert vertex/index counts against the decomp's loop bounds in a test.
- 3.2 Rewrite `outer_sky.wgsl` as a literal transcription of `VSHADER_OUTER_SKY` +
  `PSHADER_OUTER_SKY`, with each WGSL line commented with the asm it came from (§6.6).
- 3.3 Feed `c92`/`c93` from step 2's `BlendedTheme`.
- 3.4 Feed the pixel-shader `c0.w` from the real texture blend factor.
- 3.5 Resolve and upload `SkyTexture0` / `SkyTexture1` per keyframe.
- 3.6 Land the first golden images: sky at 00:00, 06:00, 12:00, 18:00.

### Step 4 — Base band, sun, moon, stars, clouds

- 4.1 Base band: own pipeline, `mov oD0, c92`.
- 4.2 Sun/moon: port `RenderSun` (`:1617`) and `RenderMoon` (`:1812`); read `CSkyDef`
  (`fablelib/defs/engine_sky_def.hpp`) for the real orbit/size/texture fields.
- 4.3 Star field: `BuildStarFieldVB` (`:3559`) + `RenderStarField` (`:3824`).
- 4.4 Clouds: `BuildCloudMesh` (`:434`) + `RenderClouds` (`:2636`) — largest remaining sky
  piece; defer until 4.1–4.3 land.

### Step 5 — Landscape, re-architected

- 5.1 **Design review first.** Post the layer-mesh architecture (§3.4) and agree how far to
  go: full per-patch layer meshes + surface composition, or a simplified equivalent. This is
  the step most likely to be deliberately scoped down — decide explicitly rather than drift.
- 5.2 Port `CEngineLandscapeMeshBuilder` to produce per-layer meshes with real `Blend` /
  `CliffU` / `CliffV` and `MappingDirection`.
- 5.3 Transcribe the `_FOREGROUND` and `_BACKGROUND` shader pairs.
- 5.4 Wire lighting constants (`Ambient`, light dir/colour, backlight) from step 2.
- 5.5 Surface composition (`CEngineSurfaceCompositionManager`) — likely the simplification point.

---

## 6. Tooling

The workflow in §2 demands "verify off-screen". That is only realistic with tooling. Ranked
by leverage for *this* project's actual problem, which is **accuracy, not performance**.

### 6.1 Headless deterministic capture — *highest leverage, build first*

```
openalbion shot --scene sky --level FinalAlbion --theme ENVIRONMENT_THEME1 \
                --time 06:00 --camera 1024,1024,120 --look 0,1,0 \
                --size 1280x720 --out shots/sky-0600.png
```

wgpu renders offscreen to a `TextureUsages::COPY_SRC` colour target and reads back via a
mapped buffer — no window, no compositor, no winit event loop. Requirements: fixed camera
(no free-fly), fixed clock (no `delta_time` advance), fixed asset order.

Why this matters more than anything else here:
- **Reproducible.** The same command gives the same pixels, so "does this commit change the
  picture?" becomes a `cmp`, not a memory of what it looked like yesterday.
- **Shareable.** I can attach a PNG to a review comment. Right now Jamen has to run the app
  to see what I did.
- **Diffable.** Before/after pairs make the sRGB change (§3.5) and every colour change legible.
- **Testable.** It is the substrate for 6.3.

Build it in step 1, not later. Every subsequent step gets cheaper.

### 6.2 Value probe with provenance

```
openalbion probe --theme ENVIRONMENT_THEME1 --time 06:00
```

```
sky.uniforms.zenith_color   = (0.478, 0.561, 0.729, 0.647)
  ├ rgb  ← lighting_colours.tga texel (col 4, row 13)   [SkyGradientTopLookupRow]
  │        col = ColourLookupColumn(0) + keyframe(4), keyframes 3..4 blended t=0.50
  └ a    ← lighting_colours.tga texel (col 4, row 14).r [SkyGradientTopAlphaLookupRow]
sky.pixel.c0.w              = 0.500   ← ENVIRONMENT_THEME1.Time[4].SkyTexture1Blend
```

This is what makes rule 3 enforceable. Every number the GPU sees, traced back to the byte it
came from. When the picture is wrong, this tells you in one command whether the bug is in
the data path or the shader — the exact distinction we could not make during the fix loop.

### 6.3 Golden-image tests

Commit reference PNGs under `tests/golden/`. A test runs 6.1 for each scene/time and compares
with a perceptual threshold (a small crate, or a hand-rolled per-channel mean+max delta —
avoid an exact `cmp`, GPU rasterisation differs slightly across drivers).

Rules that keep this from becoming a nuisance:
- Goldens are **regenerated deliberately**, via `just bless`, never automatically.
- A commit that changes a golden must say why in the message.
- Keep the set small (~8 images). Golden suites rot when they are large.

Critically: **a golden is not proof of correctness, only of stability.** It catches "step 5
silently broke the sky", which is exactly the regression class we have been eating. Accuracy
still comes from the oracles.

### 6.4 RenderDoc — already installed

`renderdoccmd` and `qrenderdoc` are on PATH. wgpu works under RenderDoc on the Vulkan backend:

```
WGPU_BACKEND=vulkan renderdoccmd capture -c shots/frame cargo run -p openalbion -- ...
```

Use it for the class of bug the probe cannot see: wrong bind group, wrong vertex layout,
wrong blend state, unexpected texture format or mip, geometry off-screen. Concretely, it
would have settled "is the dome even facing the camera?" in one capture instead of the three
commits `23c2150`, `a53d300`, `48ffc56`.

Set `label:` on every pipeline, buffer and pass — many already are — because those strings
are what makes a capture readable.

Also available and cheaper: `wgpu::Trace::Directory(path)` on `DeviceDescriptor` (wgpu 28,
behind the `trace` feature) records an API-call trace without RenderDoc.

### 6.5 Tracy — **defer, and here's why**

Tracy is excellent, and it is the wrong tool for the current problem. Our failures are
"the sky is the wrong colour", not "the frame takes 40 ms". Adding a profiler now costs
integration time and buys nothing that moves accuracy forward, and profiling a renderer that
is still architecturally wrong measures the wrong thing.

**The trigger to add it** is step 5.2: once landscape draws N alpha-blended layer passes per
patch across a streamed patch grid, draw-call count and buffer churn become real, and
per-patch/per-layer timing becomes the thing you need to see. At that point it is a ~1 day job:

- `profiling` crate with the `profile-with-tracy` feature, or `tracing-tracy` to bridge the
  `tracing` spans already in the codebase.
- `wgpu-profiler` for GPU-side timestamp scopes, which is the half that actually matters for
  a renderer.

Noted here so it is a scheduled decision, not a forgotten one.

### 6.6 Shader transcription convention

Every WGSL file that transcribes an original shader carries the disassembly in its header and
cites it per line:

```wgsl
// Transcription of VSHADER_OUTER_SKY (vs_1_1) + PSHADER_OUTER_SKY (ps_1_1).
// Source: ~/dl/Fable Shader Disassembly.md, SHADERS_SKY
// Registers per CEngineVSConstantLayoutBasic (engine_vs_layout_basic.cpp:109-182).
//
//   dp4 oPos.x, v0, c5      ...
//   mul r0, v1, c93
//   add r1, c0.y, -v1
//   ...

out.clip_position = u.combined_projection * vec4(in.position, 1.0);  // dp4 oPos, v0, c5..c8
out.diffuse = in.color * u.gradient_bottom                           // mul r0, v1, c93
            + (vec4(1.0) - in.color) * u.gradient_top;               // add r1, c0.y, -v1 / mul r1, r1, c92
```

Cheap, and it makes the shader reviewable by diffing against the asm rather than by reading
WGSL and hoping. A `just shader-check` can assert the header block still matches the oracle.

### 6.7 Logging

`tracing` + `tracing-subscriber` with `env-filter` are already in `openalbion`. Conventions:

- **Per-frame logging is banned at `info`.** `sky.rs:771` currently dumps six LUT rows every
  frame — that is what a `probe` command is for.
- Target-scoped filters: `RUST_LOG=openalbion::renderer::sky=debug`.
- `info` = lifecycle (asset loaded, pipeline built, level parsed). `debug` = one-shot
  resolution decisions with values. `trace` = per-frame, off by default.
- Log **decisions with provenance**, not observations:
  `"sky gradient top ← col 4 row 13 (theme ENVIRONMENT_THEME1 kf 4)"`, not
  `"gradient = (0.47, 0.56, 0.73)"`.
- `fool` uses `log` + a hand-rolled logger while `openalbion` uses `tracing`. Unify on
  `tracing` when convenient — low priority, but it stops probe output diverging in format.

### 6.8 Tests

| Layer | What it tests | Example |
|---|---|---|
| Parser | Data reads back correctly | `lev`, `tga`, `.bbb` round-trips (mostly exist) |
| **Oracle-pinned** | A ported algorithm matches decomp literals | LUT column = `ColourLookupColumn + kf`; sky mesh has exactly 73 vertices / 216 indices |
| **Provenance** | Values reach the GPU from the right byte | probe output snapshot-tested |
| Golden image | Nothing regressed visually | 6.3 |

The middle two are new and are the ones that would have caught the LUT bug. An oracle-pinned
test reads like:

```rust
// engine_sky_renderer.cpp:596 — radius literal 6.5e3, 36 segments (uVar13 < 0x24)
assert_eq!(mesh.vertices.len(), 1 + 36 * 2);
assert!((mesh.vertices[1].position.xy().length() - 6500.0).abs() < 0.01);
```

### 6.9 Justfile

There isn't one. `fable-reimpl` has a good one. Add:

```
just shot <scene> <time>   # 6.1
just probe <theme> <time>  # 6.2
just golden                # run golden tests
just bless                 # regenerate goldens (deliberate)
just capture <scene>       # launch under RenderDoc
just shader-check          # 6.6 header/oracle consistency
just oracle <NAME>         # grep a shader out of the disassembly
```

---

## 7. Keeping Jamen in the loop

The failure mode was me making a chain of small guesses and Jamen only seeing the result.
The counter is to make the *derivation* the reviewable artefact, not the diff.

**Per step, in order:**

1. **Derivation note** — before any code. What the question is, the quoted asm / decomp /
   def lines, and what I conclude. Short (§3-style). Jamen confirms or corrects the reading.
   *This is the gate; nothing is implemented before it passes.*
2. **Implementation** — one mechanism, matching the approved derivation.
3. **Evidence** — probe output and/or a `shot` PNG pair (before/after). Attached, not described.
4. **Commit** — message names the oracle:
   `sky: fetch gradient at ColourLookupColumn+keyframe (environment_theme.cpp:1862)`

**Escalate immediately, don't guess, when:**
- Two oracles disagree.
- The decomp body is too mangled to read confidently.
- A step turns out to need an architectural choice not covered by the plan (step 5.1 is a
  known one; there will be others).
- A value cannot be sourced and would have to be invented.

**The tripwire:** if I am about to change a number to make the picture look better, stop.
That is the failure mode this document exists to prevent. Find the oracle, or escalate.

**Cadence:** one step per working session, ending with a written summary of what landed, what
the evidence was, and what the next derivation note will cover. Steps 0 and 1 are mechanical
and can run with a lighter touch; step 2 onward gets the full protocol.

---

## 8. Useful commands

```bash
# find a shader in the disassembly
grep -n "VSHADER_OUTER_SKY" -A 20 ~/dl/"Fable Shader Disassembly.md"

# list a decomp class's members/methods
sed -n '/^class CEngineSkyRenderer/,/^};/p' \
  ~/git/fable-reimpl/src/fableengine/engine_sky_renderer.hpp

# list functions in a decomp source
grep -n "^// qualified:" ~/git/fable-reimpl/src/fableengine/engine_sky_renderer.cpp

# extract one function body
sed -n '/qualified: CEngineSkyRenderer::BuildOuterSkyMesh/,/^}/p' \
  ~/git/fable-reimpl/src/fableengine/engine_sky_renderer.cpp

# shader constant register assignments
grep -n "Offset = \|\.Count = " ~/git/fable-reimpl/src/fableengine/engine_vs_layout_*.cpp

# real authored data
grep -n "ColourLookupColumn" \
  ~/doc/Fable_Anniversary-2013-02-25/Fable/Data/Defs/environment.def

# shader bank contents
od -A d -t x1z ~/doc/Fable_Anniversary-2013-02-25/Fable/Data/shaders/vs_sky.bbb | head -40
```

---

## 9. Unverified constants

Track anything that could not be sourced. Empty is the goal.

| Location | Value | Status |
|---|---|---|
| `renderer/terrain.rs` | `HEIGHT_SCALE = 2048.0` | unsourced — real scale is in `engine_landscape*.cpp` |
| `renderer/terrain.rs` | `CELL_SIZE = 1.0` | unsourced — ″ |
| `renderer/terrain.rs` | `texture_scale = 0.0625` | unsourced; moot once layer meshes supply `CliffU`/`CliffV` (step 5.2) |
| `renderer/terrain.wgsl` | placeholder light dir + `0.25/0.75` shade | placeholder; real form is `Ambient + saturate(n·l)²·Diffuse + max(−n·l,0)·Backlight` (step 5.4) |
| `renderer/model.wgsl` | placeholder light dir + `0.3/0.7` shade | placeholder; models not yet in scope |
| `renderer/model.rs` | `ALPHA_CUTOFF = 0.5` | unsourced |
| — | `LightArray` / `LightGlobals` offsets in the Lights layout | §3.8 open item, step 1.5 |

Retired from this table: the sky dome's `36` segments (`engine_sky_renderer.cpp:616`,
`while (uVar13 < 0x24)`) and its `7000` / `−500` / `6500` extents are sourced and cited
in place.

---

## 10. History

- **2026-08-05** — **Step 0 complete** on branch `renderer-refocus`: invented mechanisms
  stripped (`dc165b1`), raw non-sRGB colour space (`8e59ee5`), native Z-up (`99f0d8d`),
  unverified constants audited (§9). Sky now renders its raw texture with zeroed
  gradients; landscape renders untextured flat-lit. Next: step 1 (tooling).
- **2026-08-05** — Renderer review. Identified the LUT column model (§3.1), the landscape
  layer architecture (§3.4) and the sRGB mismatch (§3.5) as the three root causes behind the
  sky/terrain fix loop. Decoded the `.bbb` shader bank format (§3.7) and recovered the named
  shader constant register map (§3.8), cross-verified against the disassembly on six
  registers. Z-up settled. Document reconstituted; plan in §5 and tooling in §6 pending review.
