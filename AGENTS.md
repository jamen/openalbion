# OpenAlbion — agent guide & renderer plan

> **Note (2026-08-05):** the previous `AGENTS.md` was never committed and is gone from disk.
> This is a fresh, renderer-focused reconstitution. The def-compiler/architecture chapters of
> the old document were not recovered; re-add them here when that work resumes.

---

## 0. Where we are

> **Session summary — 2026-08-08, branch `renderer-refocus`.**
> **Landed:** the renderer is its own crate with a data-only input boundary (§11), and
> `openalbion` is a plain binary again — one game binary, no library, no harness.
> **Abandoned: the mirror.** Comparing screenshots against the original engine needed
> more setup than it was worth. `packages/mirror` and `scenes.toml` are deleted, and
> §5 step 1, §6 and §7.1 are **history, not plan** — read them for what they established,
> not for what to do next.
> **Still standing:** step 0 (strip-back, raw sRGB, Z-up), and every finding in §3. The
> oracles (§1) and the ground rules (§2) are unchanged and are now the *only* verification
> route: derive from the decomp, check numerically, then look at the screen.
> **Corrected along the way:** FOV is horizontal data, not a fitted vertical constant
> (§3.10); the colour LUT is indexed by theme column, not time (§3.1); the landscape is
> N alpha-blended layer passes, not a single 3-way mix (§3.4).
> **Next renderer work:** step 2, the environment layer. It was never blocked on capture.


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

### 3.9 The original engine is controllable — the comparison surface

`~/Fable` is not a plain retail install. It is an instrumented dev setup, and it changes
what verification is possible.

**It runs here.** Fable: The Lost Chapters (Steam 204030) with Proton 10.0 / Experimental /
Hotfix, and `steamapps/compatdata/204030`, `204030-debug`, `204030-debug2` prefixes already
exist. `Fable.exe` is **PE32 (32-bit x86)**.

**Console command script.** `~/Fable/user.ini` is executed as console commands at boot
(`SetMaxAnisotropy(4); RunScript("joystick.ini"); ActivateQuest("Gameflow");`). Commands
found in `Fable.exe` include:

| Command | Use |
|---|---|
| `EnableSky`, `EnableLandscape`, `EnableWater`, `EnableWeather`, `EnableShadows`, `EnableFlareSprites` | **per-subsystem isolation** |
| `EnableScreenEffect{ColourFilter,GlowRenderer,OutlineGlow,RadialBlurRenderer}` | disable post effects |
| `DebugCamera`, `PauseTime`, `SetTimeOfDay` | deterministic framing |

These correspond to `CEngineComponent::GetConsoleEnableFunctionName()` — every engine
component has one. **Subsystem isolation is the single most useful lever here:** with
`EnableLandscape(0)`, `EnableWater(0)`, `EnableWeather(0)`, `EnableShadows(0)` the original
renders sky only, which is directly comparable to what we have today. Without it we would be
diffing our stub landscape against the real one and drowning in noise.

**Scripted camera and clock.** `FableScriptExtender.dll` (Jamen's, integrated with
`~/git/EgoCore`) binds the game's script API to Lua. Relevant bindings, from the DLL's
symbol strings:

```
CameraMoveToPosAndLookAtPos      CameraMoveBetweenLookFromAndLookTo
CameraMoveToPosAndLookAtThing    CameraMoveBetweenLookingAt
CameraUseCameraPoint             CameraDefault / CameraResetToViewBehindHero
SetTimeOfDay / GetTimeOfDay      SetTimeAsStopped
```

So a Lua script can place the camera at an exact position looking at an exact point, freeze
the clock at an exact time of day, and hold it. **That is the basis of the testbed:** both
engines can be given the same camera and the same clock, by construction rather than by eye.

**Capture.** `grim` (Wayland) and OBS are installed. RenderDoc 1.45 is installed but is
**x64-only and supports Vulkan/GL/GLES** — it cannot attach to a 32-bit PE, so capturing
Fable's D3D9 (even DXVK-translated) is not available without a 32-bit capture library.
Numeric extraction should therefore go through the ASI/hook route that is already proven in
this setup (`DebugThingListHook.asi` and friends), not RenderDoc.

### 3.10 Camera and projection — FOV is data, and it is *horizontal*

`CCamera` (`bbblibrary/lib_camera.hpp:1005`) carries `HorizontalFOV`, `VerticalFOV` and a
`FOVFlags`. The two-argument constructor (`lib_camera.cpp:145`) sets `FOVFlags = 1` and
leaves `VerticalFOV` at 0 unless a positive vertical FOV is supplied, in which case
`FOVFlags = 3`. So the normal case is **horizontal FOV only, vertical derived**.

`CEngineCamera` builds the projection (`fableengine/engine_camera.cpp:781-793`):

```c
this->AspectRatio = aspect_size.X / aspect_size.Y;
if (desc->Use2DFOV == false) {
    fVar8 = _CItan();
    this->HomogenousViewScaleX = 1.0 / (float)fVar8;
    fVar1 = (1.0 / (float)fVar8) * (aspect_size.X / aspect_size.Y);
} else {
    fVar8 = _CItan();  this->HomogenousViewScaleX = 1.0 / (float)fVar8;
    fVar8 = _CItan();  fVar1 = 1.0 / (float)fVar8;
}
this->HomogenousViewScaleY = fVar1;
```

then scales view-space X by `ScaleX` and Y by `ScaleY`. A standard right-handed perspective
has `m00 = f/aspect`, `m11 = f` with `f = 1/tan(fovY/2)` — the same `m11 = m00 * aspect`
relation — so equating them gives

> **`fovY = 2 · atan( tan(HFOV/2) / aspect )`**

**The value is data.** `camera_mode.def` gives every `CAMERA_MODE` an `FOV`;
`CAMERA_MODE_TEMPLATE` defaults to **70.0** and individual modes range 70–90. The field is
already modelled in `~/git/fable-defs/packages/defs/src/def/camera_mode.rs:15`.

At 70° horizontal and 16:9 the vertical FOV is **~43°**, not 70°. We were feeding the def's
number straight into `perspective_rh` as a *vertical* FOV, giving a far too wide view — so
this was never a constant to fit against a reference (as §7.1 originally proposed), it was a
constant to read. `Camera::fov_y` (`openalbion/src/camera.rs`) derives it, with the
derivation as a unit test; `Camera::fov_h` holds the horizontal value.

Two residuals, both flagged on `Camera::fov_y`:
- The argument to `_CItan()` is register-passed and invisible in the decomp, so half-angle vs
  full-angle is inferred rather than read. Half-angle is near-certain: full-angle would make
  `ScaleX = 1/tan(70°) = 0.36`, an implausibly wide view, where half-angle gives 1.43.
  A reference capture would settle it definitively — but see step 1: there is no capture
  route any more, so this stands on the arithmetic alone.
- *Which* camera mode is active in a given shot is not yet established, so `fov_h` defaults
  to `CAMERA_MODE_TEMPLATE`'s 70 and should be read from `camera_mode.def` per mode once the
  camera system lands.

`Use2DFOV` selects the second path, where vertical FOV is independent — that is the 2D/UI
camera (`ENGINE.FOV_2D`) and does not apply to the world camera.

---

## 4. Assessment of the current renderer

> **Historical (2026-08-05).** This audit drove step 0, which is complete — every
> "Delete" row has been actioned. Paths are pre-split: the renderer now lives in
> `packages/renderer/src/` (§11), and `files.rs` in each binary. Kept because the
> *pattern* it names is the thing to keep watching.

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

### Step 1 — ~~The mirror~~ **ABANDONED (2026-08-08)**

Building a comparison testbed against the original engine — drive retail or the dev build
headlessly, park the camera, freeze the clock, capture a reference frame, diff ours against
it — turned out to need far more setup than it returned. `packages/mirror` and `scenes.toml`
are deleted. §6 and §7.1 remain as a record of what was learned; **neither is a plan.**

What was real and is kept:

- **1.1 landed and then some** — the lib/bin split became the crate split in §11.
- **1.7 is still open** but no longer blocking: the `LightArray` / `LightGlobals` register
  offsets (§3.8). Nothing renders lighting yet, so it is due when step 5.4 needs it.

**What replaces it.** Nothing symmetrical, deliberately. Verification goes back to §2's
own rules, which never depended on the harness: derive from the oracles, check the numbers
in a test or a dump, *then* look at the screen. §6.9's test table still applies — the
oracle-pinned and provenance layers are ordinary unit tests, and the crate boundary in §11
is what makes them possible without a GPU or a Fable install.

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
  **Observed (2026-08-06, via the since-deleted mirror harness):** our apex UV is `(0.5, 0.0)`
  where the original is `(-DELTA_NOTIONAL_ZERO, -DELTA_NOTIONAL_ZERO)` — effectively `(0, 0)`
  under clamp addressing (`engine_sky_renderer.cpp:581`). The cap renders as a flat disc
  either way (V is 0 across it in both), but the sampled column differs. Fix here, with the
  rest of the mesh.
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

## 6. ~~The mirror~~ — comparing against the original *(historical)*

> **Abandoned 2026-08-08 — see step 1.** Kept for §6.6–§6.9, which are still live: the
> Tracy trigger, the shader transcription convention, the logging rules and the test
> table do not depend on the harness. Everything in §6.1–§6.5 describes tooling that no
> longer exists.

The goal was **not** pixel-parity. Jamen wants room to improve the graphics, while not
drifting far from the original's look. So the harness must distinguish *deliberate
divergence* from *regression*, and it must be honest about which it is looking at.

### 6.1 Two tiers of ground truth

**Tier 1 — images.** A screenshot of the original at a known camera, clock and subsystem
mask, compared perceptually against our render of the same manifest scene. Answers "does it
look right". Cheap, reliable, available today.

**Tier 2 — numbers.** The actual shader constants the original feeds the GPU for that frame:
`c92`/`c93` at a given theme and time, the dome's vertex buffer, the blend/depth/cull state.
This is strictly better than tier 1 because it verifies the *data path* before a single pixel
is drawn — exactly what rule 3 asks for. It would have caught the LUT column bug instantly.

Tier 2 is **not** available through RenderDoc: `Fable.exe` is 32-bit PE and the installed
RenderDoc is x64/Vulkan-only. The realistic route is the ASI/hook infrastructure already
proven in this setup (`DebugThingListHook.asi`, FSE) — a small 32-bit plugin that hooks
`IDirect3DDevice9::SetVertexShaderConstantF` and logs `(register, values)` around the sky
draw. Treat this as a **spike with its own go/no-go**, not a dependency: tier 1 plus an
FSE-driven camera is the reliable path, and tier 2 is a large bonus if it lands.

### 6.2 The metric

Compare at reduced resolution — downscale both images before measuring — so anti-aliasing,
texture filtering and driver differences do not dominate.

Then split the judgement in two:

- **Gate on structure.** MS-SSIM or a gradient-domain metric over luminance. This measures
  geometry, silhouettes, horizon placement, layout. A structural mismatch is a *bug*: the
  dome is the wrong shape, the horizon is at the wrong height, the terrain is at the wrong
  scale. There is no legitimate reason for structure to drift.
- **Report, do not gate, on colour.** Per-channel mean and histogram distance. Colour is
  where deliberate improvement lives, so a threshold here would fight the goal. Track it, plot
  it over time, and let a human judge.

This split is the whole answer to "not 1:1, but not too far": structure is held tightly,
appearance is held loosely and visibly.

**Implemented** in `packages/mirror/src/compare.rs`:

```
mirror compare <scene>                        # reference/<scene>.png vs out/<scene>.png
mirror compare --a X.png --b Y.png            # any two images (before/after regressions)
```

Prints structure (gated, non-zero exit on failure) and colour (reported), and writes a
contact sheet — `a | b | SSIM heatmap` — so a human sees *where* the difference is, not just
how much. Structure is SSIM over Rec.601 luma, 8×8 windows at stride 4, computed on images
box-downscaled to 512px on the long side; colour is measured at full resolution, since
downscaling would hide banding and gradient errors.

**The gate threshold is provisional at 0.95.** It has only ever been run against two of our
own renders. Calibrate it once real references exist — a reimplementation against the
original will not score like two builds of the same renderer.

### 6.3 The Divergence Ledger

Same pattern as the def compiler's semantic verification. Every scene that differs from its
reference produces a ledger entry, and every entry is classified:

```
scene                     structure  colour   class     note
witchwood-sky-0600        0.981      ΔE 4.2   ACCEPTED  gradient banding removed (16-bit LUT lerp)
witchwood-sky-1200        0.994      ΔE 1.1   OK
witchwood-land-0600       0.612      ΔE 9.8   BUG       horizon 40px low — HEIGHT_SCALE unsourced (§9)
```

- `OK` — within both thresholds.
- `ACCEPTED` — differs, with a written reason. This is where "I improved it" gets recorded.
- `BUG` — differs, unexplained. **A new unexplained entry is the regression signal.**

The ledger is committed. A change that moves a scene from `OK` to `BUG`, or that adds a
`BUG` row, must be explained or reverted before the step closes. A change that turns a `BUG`
into `OK` is the point of the exercise.

### 6.4 Subsystem isolation makes this usable now

`EnableLandscape(0)`, `EnableWater(0)`, `EnableWeather(0)`, `EnableShadows(0)`,
`EnableScreenEffect*(0)` on the original (§3.9) restrict the reference to the subsystem under
test. This is what makes comparison viable *today*, with only the sky implemented — the
alternative is diffing our stub landscape against the real one and learning nothing.

Each manifest scene declares its `subsystems`, and the capture procedure derives the console
prologue from it. As subsystems land, scenes are added that enable more.

### 6.5 What this replaces

The earlier plan put `shot` and `probe` on the openalbion binary. That was wrong: it grows the
game's interface to serve the test harness, and it couples the two. Everything above lives in
`packages/mirror` instead. The game keeps a viewer CLI; the library is the shared surface.

### 6.6 Tracy — still deferred

Unchanged from the earlier assessment: our failures are accuracy, not frame time, and
profiling an architecturally wrong renderer measures the wrong thing. The trigger is step 5.2,
when landscape starts drawing N alpha-blended layer passes per patch across a streamed grid.
Then: `profiling` with `profile-with-tracy` (or `tracing-tracy` to bridge existing spans),
plus `wgpu-profiler` for GPU timestamp scopes — the half that actually matters for a renderer.

### 6.7 Shader transcription convention

Every WGSL file that transcribes an original shader carries the disassembly in its header and
cites it per line — see `renderer/sky/outer_sky.wgsl` for the established form. This makes a
shader reviewable by diffing against the asm rather than by reading WGSL and hoping.

### 6.8 Logging

`tracing` + `tracing-subscriber` with `env-filter` are already in `openalbion`.

- **Per-frame logging is banned at `info`.** Dump values from a test or a one-shot
  `debug` line instead.
- Target-scoped filters: `RUST_LOG=openalbion::renderer::sky=debug`.
- `info` = lifecycle. `debug` = one-shot resolution decisions with values. `trace` = per-frame,
  off by default.
- Log **decisions with provenance**, not observations:
  `"sky gradient top ← col 4 row 13 (theme ENVIRONMENT_THEME1 kf 4)"`.

### 6.9 Tests

| Layer | What it tests | Example |
|---|---|---|
| Parser | Data reads back correctly | `lev`, `tga` round-trips (mostly exist) |
| **Oracle-pinned** | A ported algorithm matches decomp literals | LUT column = `ColourLookupColumn + kf`; dome has 1 + 36*2 vertices |
| **Provenance** | Values reach the GPU from the right byte | `scene::*` output, snapshot-tested — no GPU needed (§11.1) |
| **Mirror** | Structure matches the original | §6.2, gated on structure only |

The middle two are new and are the ones that would have caught the LUT bug. An oracle-pinned
test reads like:

```rust
// engine_sky_renderer.cpp:596 — radius literal 6.5e3, 36 segments (uVar13 < 0x24)
assert_eq!(mesh.vertices.len(), 1 + 36 * 2);
assert!((mesh.vertices[1].position.xy().length() - 6500.0).abs() < 0.01);
```

## 7. Working together

The failure mode was a chain of small guesses with only the result visible. The counter is to
make the *derivation* the reviewable artefact, not the diff.

**Per step, in order:**

1. **Derivation note** — before any code. The question, the quoted asm / decomp / def lines,
   and the conclusion. Short, §3-style. Jamen confirms or corrects the reading.
   *This is the gate; nothing is implemented before it passes.*
2. **Implementation** — one mechanism, matching the approved derivation.
3. **Evidence** — the numbers, attached rather than described: a unit test pinning the
   ported value, or a `debug` dump of what `scene` produced. §11.1 is what makes this
   cheap — the conversion layer is testable without a GPU or a Fable install.
4. **Commit** — message names the oracle:
   `sky: fetch gradient at ColourLookupColumn+keyframe (environment_theme.cpp:1862)`

**Escalate immediately, don't guess, when:** two oracles disagree; a decomp body is too
mangled to read confidently; a step needs an architectural choice the plan does not cover
(step 5.1 is a known one); or a value cannot be sourced and would have to be invented.

**The tripwire:** if I am about to change a number to make the picture look better, stop.

### 7.1 Reference capture — findings from the real runs *(historical)*

> **Abandoned 2026-08-08.** None of the below is work to resume. It is kept because the
> findings are hard-won and still true about the *game*, not about our plans: how to get
> the retail and dev builds running under Proton on this machine, which binary is which,
> what the dev console can do, and how to resolve symbols from `Ego_r.pdb`. If the engine
> ever needs to be observed directly again, start here rather than rediscovering it.

> **RESUME HERE (2026-08-06, later).** The debug build **runs** — the binary to use is
> `ego_r.exe`, not `FableWin.exe` (the latter imports the non-redistributable VS2010 *debug*
> CRT and can never load here). It boots unattended to a rendering 1280×720 window with zero
> dialogs, and its console carries the free camera, HUD toggle, engine TGA screenshot and a
> much finer subsystem-isolation set than retail — so **FSE/Lua is no longer needed and the
> old "quest activation is the blocker" problem is moot.** One hop remains: getting past the
> front end. `SetSkipFrontend(TRUE)` exits silently; `ShowDevFrontEnd TRUE` reaches a
> keyboard menu that synthetic keys cannot reach, because the headless compositor's seat has
> no keyboard. See "Debug build — it runs" below for the full state and ranked leads.


**Status: retail launches, renders and reaches its main menu under automation. It does not
yet reach gameplay unattended.** Everything below was learned by running it, not by reading.

#### The launch environment (solved)

```bash
env -u WAYLAND_DISPLAY -u SDL_VIDEODRIVER DISPLAY="$NESTED_X" \
    STEAM_COMPAT_CLIENT_INSTALL_PATH="$STEAM_ROOT" \
    STEAM_COMPAT_DATA_PATH="$STEAM_ROOT/steamapps/compatdata/204030" \
  steam-run "$STEAM_ROOT/steamapps/common/SteamLinuxRuntime_4/_v2-entry-point" \
    --verb=run -- "$PROTON_EXPERIMENTAL/proton" run "$HOME/Fable/Fable.exe"
```

Five things that each broke it, in the order they bite:

1. **NixOS needs `steam-run`.** Both `proton` and `pressure-vessel-wrap` are dynamically
   linked for a generic distro and fail with *"NixOS cannot run dynamically linked
   executables"*.
2. **Use the Proton the prefix was built with.** `compatdata/204030` is at `11.0-100` =
   **Proton Experimental**. Launching with Proton 10.0 makes it *downgrade and rebuild the
   prefix*, which silently discards the EULA acceptance and graphics-detection registry
   values. Saves survive; those settings do not.
3. **Go through Steam Linux Runtime 4.0** (`SteamLinuxRuntime_4/_v2-entry-point`) — Proton
   Experimental's `toolmanifest.vdf` requires appid 4183110. Launching Proton bare gives a
   Wine with no FreeType, and every dialog renders as an empty 75×50 stub.
4. **Set `DISPLAY`, not `WAYLAND_DISPLAY`.** Wine uses X11, so `DISPLAY` decides which
   compositor the window lands in. With only `WAYLAND_DISPLAY` set, the game opened *on the
   user's desktop*. Unset `WAYLAND_DISPLAY` so it cannot escape the nested session.
5. **Nested headless sway works** and provides XWayland. Get its `DISPLAY` by having the sway
   config `exec` a command that writes `$DISPLAY` to a file.

#### Dialogs on the way in

| Dialog | Handling |
|---|---|
| EULA | **`Decline` holds the focus ring** — a blind `Return` declines and the game exits. Accept is `alt+a`. Persists once accepted. |
| "Fable - Warning" (`0MB, 0M ATI` — Wine reports no RAM/VRAM) | Tick *Don't show this warning again* (311,504), then *Continue Anyway* (371,535). Persists. |
| Title splash — *Press Left Mouse Button To Continue* | Not skipped by `SetSkipFrontend(TRUE)` as placed. |
| Main menu — shows `11 - Continue Game` | Reached, but see below. |

#### Front-end automation — **solved**

Retail runs unattended from launch to in-game: dialogs, splash, main menu, save selection.
The working script is `packages/mirror/capture-retail.sh`.

**The key discovery: the game reads DirectInput *relative* mouse motion and tracks its own
cursor.** Absolute `xdotool mousemove` is meaningless to it — which is why a move to
(640,360) once left the drawn cursor at (710,370), and why clicking the "right" coordinates
never worked. The fix is to slam the pointer into a corner so the game's cursor clamps to a
known origin, then walk a known delta:

```bash
xdotool mousemove_relative -- -3000 -3000    # clamp its cursor to (0,0)
xdotool mousemove_relative -- 640 340        # now at (640,340)
xdotool click 1
```

Clicking two targets in sequence each pass walks the whole front end, and each is harmless
empty space on the screen where it does not apply, so the pair is safe to repeat:

| Target | Screen | Hits |
|---|---|---|
| `640 340` | main menu | `NN - Continue Game` |
| `297 162` | Load Game | `AutoSave` |

Hit regions run from an item's text down to the next item (~45px), so aim below the text
baseline, not at it.

#### Retail vs the dev build — different command sets

`dbugst.ini`'s command list describes the **debug build**, not retail. Verified by string
search:

| Command | retail `Fable.exe` | dev `FableWin.exe` / `ego_r.exe` |
|---|---|---|
| `SetSkipFrontend`, `TakeScreenshot`, `SetInputLoad`/`Save`, `AutomatedMode`, `SetResolution`, `SetPlayerStartPos`, `ShowDevFrontEnd` | **absent** | present |
| `EnableSky`, `EnableLandscape`, `PauseTime`, `SetTimeOfDay`, `DebugCamera`, `SetStartingHolySite` | present | present |

So in retail there is **no** front-end skip, **no** engine screenshot and **no** input
playback — the front end must be clicked through, and `grim` is the capture path. The
commands that matter for the testbed (subsystem isolation, clock) *are* present.

`~/doc/Fable_Anniversary-2013-02-25/Fable/FableWin.exe` (53M) is a **dev build with the full
command set**, and `~/git/fable-reimpl` is a decomp of it (its source paths read
`fable tlc build repository` / `Fable1_5MainPC` — Anniversary was built from the TLC
codebase). Worth evaluating as a second reference engine: it would need no synthetic input
at all, and it is the binary every constant we are porting actually came from. Unevaluated:
whether it runs under Proton and what data it expects.

#### Remaining for a reference capture

The world loads and `MirrorCapture` registers, but the quest's `Main()` has not run — no
`mirror:` lines appear in `FableScriptExtender.log`. `ActivateQuest("MirrorCapture")` in
`user.ini` executes at boot, before the world exists. The same applies to the isolation
commands: `EnableSky(1)`/`EnableLandscape(0)`/`PauseTime()` run at boot and do not survive
into the loaded save — the proof screenshot still shows full landscape.

So the open question is **how to run console commands and activate a quest after the save
loads**. Leads, in order:
1. Does an FSE quest with `AddQuestRegion` self-activate when the hero enters that region?
   If so, no console activation is needed at all.
2. Is there an in-game console key in retail, drivable with the same relative-motion input?
3. Can FSE call the console, or expose the `Enable*` toggles directly?

#### Capability survey for clean screenshots (2026-08-06)

**1. FSE — sufficient for camera and clock, not for presentation.**
1,275 exported symbols. Has: the whole `Camera*` family
(`CameraMoveToPosAndLookAtPos`, `CameraMoveBetweenLookFromAndLookTo`,
`CameraCircleAroundPos/Thing`, `CameraUseCameraPoint`, `CameraDefault`,
`CameraUseScreenEffect`); clock (`SetTimeOfDay`, `SetTimeAsStopped`); world placement
(`EntityTeleportToPosition`, `GainControlAndMoveToPosition`,
`DontPopulateNextLoadedRegion`, `IsRegionLoaded`); and presentation
(`StartMovieSequence`/`EndMovieSequence`, `EndLetterBox`, `FadeScreenIn`, `IsInCutscene`).
Lacks: any HUD toggle, free camera, screenshot, render-state access, and file I/O
(Lua has `coroutine/debug/math/package/string/table` — no `io`, no `os`).
It is the *game script* API (`CGameScriptInterface`), not an engine API.

**2. Retail console `Enable*` set is richer than assumed** — from `Fable.exe` strings:
`EnableSky`, `EnableLandscape`, `EnableWater`, `EnableWeather`, `EnableShadows`,
`EnableAnimatedMeshes`, `EnableStaticMeshes`, `EnableRepeatedMeshes`, `EnableSprites`,
`EnableSpriteTrails`, `EnableDecals`, `EnableGroupDecals`, `EnableLines`,
`EnablePrimitives`, `EnableChangingPrimitives`, `EnableFlareSprites`,
`EnableWeaponTrails`, `DrawWeaponTrails`, `DrawProjectileWeaponTrails`,
`EnableScreenEffect{ColourFilter,GlowRenderer,OutlineGlow}`, `EnableSounds`.
These are `CEngineComponent::GetConsoleEnableFunctionName()` values.
**No GUI/HUD toggle among them** — `GlobalDrawGUI` is debug-build only.

**3. Free camera exists in the engine.** `CPlayer` (`fablelib/player.hpp:997-1231`):
`CCamera CurrentFreeCamera / FreeCamera / OldFreeCamera`, `bool UsingFreeCamera /
FreeCameraTrackingPlayer / ControllingFreeCamera`, `void SetUsingFreeCamera(bool)`,
`bool IsUsingFreeCamera() const`, `void UpdateFreeCamera()`, plus
`CInputProcessControlFreeCamera`. `SetUsingFreeCamera(bool)` is a clean hook target.
**`DebugCamera` is a retail console command** — test it before writing any hook.

**4. Resolution and quality are not console-settable in retail.** No `SetResolution`,
no `SetMaxTextureSize`, no quality commands. They live in the registry under
`HKCU\Software\Microsoft\Microsoft Games\Fable TLC` (vendor-keyed encoding) and are
written by the in-game Options menu or `FableLauncher.exe`. Since synthetic input now
works, driving Options once is viable and persists. No engine screenshot in retail, so
`grim` on the nested output remains the capture path — lossless PNG of the real
framebuffer, so quality is bounded only by the render resolution.

**5. The dev build has all of it natively.** `FableWin.exe` / `ego_r.exe` in the
Anniversary tree carry `SetResolution`, `TakeScreenshot`, `SetSkipFrontend`,
`GlobalDrawGUI`, `FreeCamOnWithPlayer`, `AutomatedMode`, `SetPlayerStartPos`. If it
runs, it sidesteps the HUD, free-cam, resolution and screenshot problems at once — and
`~/git/fable-reimpl` is a decomp of that binary.

#### Step 1 results — quest activation is the blocker

Tested `StartMovieSequence` (HUD hide) and quest-driven camera parking. **Neither ran.**

- **FSE quests register but never activate.** The log ends at *"Custom scripts registered
  with the game"* and no `Init`/`Main` is ever called — probes placed in **both**
  `MirrorCapture.lua` and `FSE_Master.lua` produced no output. `FSE_Master` is *not*
  auto-active despite its comment.
- **`ActivateQuest(...)` from `user.ini` is too early.** Console script runs at boot;
  registration happens when the world loads (FSE log jumps 12 → 29 lines at that point).
  Activating a quest that does not exist yet is a no-op.
- So **every FSE capability is currently unreachable**, including the camera park. This is
  the single blocker — not the camera API, not the HUD.

**Retail has NO working console.** `Fable.exe` contains `ConsoleAlpha` and
`ConsoleListContaining`, but these are remnants — the console was stripped and deactivated,
long-established in the modding community (confirmed by Jamen 2026-08-06). **Do not chase
it.** The debug build's console is real.

**Keyboard input works in-world** — it opened the Logbook. So once the console key is
known, driving it is already solved.

**Options menu is reachable from the pause menu** (Inventory / Skills / Hero Status / Map /
Photo Journal / **Options** / Return To Game). That is the route to max resolution and
quality settings, which are registry-backed and persist (§7.1 question 4).

#### Debug build evaluation (2026-08-06) — promising, not yet running

Retail has **no working console** — the `ConsoleAlpha`/`ConsoleListContaining` strings are
remnants of one that was stripped and deactivated (confirmed by Jamen; long-established in
the modding community). Do not chase it. The **debug build does** have a console.

**The Anniversary tree is already a hybrid, and it is the configuration we want.**
`~/doc/Fable_Anniversary-2013-02-25/Fable/Data` symlinks its *art* to the retail TLC
install — `graphics.big`, `pc/textures.big`, `pc/frontend.big`, `shaders/pc/shaders.big`,
`Misc/pc/effects.big`, and the English fonts/text/dialogue all point at
`/home/jamen/Fable/data/...`. Native to the tree: `FinalAlbion.wad`/`.stb`,
`meshdata.bbb`, `CompiledDefs`, `Defs`, `LightingTable`. 5.0 GB total.

That means **dev engine + retail TLC art**, which resolves the open worry: the art is *not*
remastered, so colour comparison against this build is valid, not just structural
comparison. Combined with `~/git/fable-reimpl` being a decomp of this binary, oracle and
reference would finally be the same thing.

**"Worse graphics" is configuration, confirmed.** `default_userst.ini` sets
`SetMaxTextureSize(512)` where retail's `userst.ini` uses `2048`, plus
`SetResolution(1024,768,16)` and `SetStaticMapQuality(2)`. All console-settable in this
build, unlike retail.

**Config files:** `FableWin.exe` (PE32) uses `user.ini`/`userst.ini`, created from the
`default_user.ini`/`default_userst.ini` templates in the tree (there are also
`deliverable_*` and `final_deliverable_*` variants). `default_userst.ini` documents
`SetInputSave("packet_save.sav")` / `SetInputLoad(...)` for record/playback, and there is a
`record_gameplay.ini` — the input-playback route that retail lacks.

**Gotcha:** the Proton prefix must live **inside Steam's `compatdata` tree** — pressure-vessel
does not map `/tmp`, so a prefix there fails with `FileNotFoundError: .../pfx.lock`. Pass
`STEAM_COMPAT_MOUNTS` for any game directory outside the Steam tree.

#### Debug build — **it runs. Use `ego_r.exe`, not `FableWin.exe`.** (2026-08-06)

`FableWin.exe` was the wrong binary. Its PE import table names **`MSVCR100D.dll` /
`MSVCP100D.dll`** — the *debug* CRT, which ships only with Visual Studio 2010, is not
redistributable, has no Wine builtin, and is not in the tree's `dlls/` (that holds the
VC7.1 pair, `msvcr71`/`msvcp71`, for a different binary). The loader therefore fails
before any code runs: no window, no `bbb.log`. That was the whole mystery.

`ego_r.exe` (16 MB, the dev **release** build) imports `MSVCR100.dll` / `MSVCP100.dll` /
`d3dx9_43.dll` — **all three are Wine builtins in Proton Experimental**. It launches,
renders, and carries the *same* full command set as `FableWin.exe` (verified by string
search on all of `SetResolution`, `TakeScreenshot`, `SetSkipFrontend`, `GlobalDrawGUI`,
`FreeCamOnWithPlayer`, `AutomatedMode`, `SetPlayerStartPos`, `EnableSky`, `SetTimeOfDay`).
`Ego_r.pdb` (134 MB) sits beside it. **`ego_r.exe` is the reference engine.**

Launcher: `packages/mirror/capture-dev.sh`. Config: `packages/mirror/dev-build-config/`
(`user.ini` + `userst.ini`, mirrored into the game tree; neither existed before).

**Solved, and each one bit:**

1. **Both ini files are read**, `default_userst.ini` first then `userst.ini` (proven with
   `WINEDEBUG=+file`). So `userst.ini` overrides the template. `boot.ini` is also looked
   for and its absence is harmless — that AGENTS lead is closed.
2. **`SkipConfigDetection(TRUE)` works** — `ConfigDetect.dll` is then never loaded, and
   the "0MB RAM / 0M ATI" warning never appears.
3. **The "did not exit correctly" dialog is a registry flag, not a real error.**
   `GFConfigDetection` writes `HKCU\Software\Microsoft\Microsoft Games\Fable TLC\GFX_RESET
   = 1` on **every** boot and clears it only on a clean exit (`main.cpp:281`). A harness
   that kills the game therefore re-arms it every run. `capture-dev.sh` clears it in
   `pfx/user.reg` while wineserver is down.
4. **`AllowBackgroundProcessing(TRUE)` is mandatory headless.** `main.cpp:1142`:
   `sys_init.WaitWhileInactive = !GAllowBackgroundProcessing`, and the comment says the
   game "must have focus before it does anything at all, even during development". Without
   it the game ignores all input and exits after ~7 minutes. This explained two separate
   mystery symptoms at once.
5. **Only one Fable at a time, per prefix.** `WinMain` takes a global mutex
   (`main.cpp:363`) and a second instance `return 0`s immediately — no window, no log,
   indistinguishable from a crash. `capture-dev.sh` refuses to start if one is up.

With 1–4 in place the build boots **fully unattended to a rendering 1280×720 window with
zero dialogs**.

**`TakeScreenshot(Int)` is gated on `AllowMovieRecording`.** `display_engine.cpp:1191`
only reaches `SaveAsTGA` when `GAllowMovieRecording` is set. Output is
`data\movies\shot%06d.tga` — lossless, straight off the back buffer, so once we are
in-world `grim` and the compositor drop out of the capture path entirely.

**The isolation vocabulary is far richer than retail's** (from `ego_r.exe` strings) — this
is the direct answer to "our renderer draws less than the original". Beyond retail's set:
`EnableClouds`, `EnableSea`, `EnableTextures`, `EnableCompositeTextures`,
`EnableLandscapeBumpMapping`, `EnableLandscapeFog`, `EnableLandscapeTesselation`,
`EnableLandscapeLODUpdate`, `EnableGlow`, `EnableParticles`, `EnableParticleRendering`,
`EnableLocalLights`, `EnableShadowedSpotLights`, `EnableDithering`, `EnableMouseCursor`,
`EnableForeground`/`EnableBackground`, `EnableEngineScreenshotMode`. Plus the free camera
as *console commands*, not an FSE binding: `SetFreeCam`, `SetFreeCamPos`,
`SetFreeCamLookVector`, `SetFreeCamFOV` / `SetFreeCameraFov`, `SetFreeCamHeightLock`,
`FreeCamOnWithPlayer`; and `GlobalDrawGUI` for the HUD. **FSE/Lua is no longer needed for
any of this** — the §7.1 "quest activation is the blocker" problem is moot.

**RESUME HERE — the one remaining hop: getting past the front end.**

- `SetSkipFrontend(TRUE)` makes `ego_r.exe` exit silently ~5 s after the window appears.
  Reproduced 4× — with and without `SkipProfileSelection`, `SetStartingHolySite`, and with
  a user profile copied into the debug prefix from retail's
  (`Documents/My Games/Fable/Saves/11/`). Cause not yet established; nothing is logged.
- `ShowDevFrontEnd TRUE` (what the tree's own dev config uses) *does* work and reaches a
  keyboard-driven text menu: `9 - Quit` / `A - [Debug Profile]` / `B - Create New Profile`.
  Far more automatable than retail's mouse-only front end — **except that no synthetic
  keystroke reaches it.**
- **Why keys don't land:** wlroots' headless backend creates no input devices, so the seat
  never advertises a keyboard capability and XWayland gets none. XTEST *pointer* events
  still work — which is exactly why the retail mouse script works — but key events go
  nowhere, and `wtype` reports success while doing nothing. The window genuinely has focus
  (`xdotool getwindowfocus` returns it); the keyboard simply does not exist.
- **Do not left-click the dev front end** — a click on the Select Profile screen quits the
  game.
- **Xvfb is not the fix.** It has full XTEST keyboard support, but no Vulkan, so DXVK
  cannot create a device and the game dies at [5]. Falling back to WineD3D/llvmpipe would
  also invalidate colour comparison, which is the whole point of this build.

Ranked leads for next session:
1. **Give the nested compositor a real keyboard.** Run sway with `WLR_BACKENDS=wayland`
   nested inside Jamen's session (inherits the parent seat, keys work, costs a visible
   window), or feed it a `uinput` device via `ydotool` with libinput enabled (drop
   `WLR_LIBINPUT_NO_DEVICES=1`). Needed only *once* if it creates a Debug Profile that
   persists.
2. **Input playback.** `SetInputSave("packet_save.sav")` / `SetInputLoad(...)` +
   `AutomatedMode(TRUE)` are documented in `default_userst.ini` and `record_gameplay.ini`.
   Record the front-end walk once on a real display, replay it headlessly forever. This is
   the deterministic answer and retail has no equivalent.
3. **Find out why `SetSkipFrontend` dies** — with the profile question eliminated, the next
   step is `GFMain`'s skip-frontend path in `main.cpp` / `main_game_component.cpp`.

#### `SetPlayerStartPos` did not fix skip-frontend (2026-08-07)

Tested `SetSkipFrontend(TRUE)` + `SetStartingHolySite("LookoutPointHSP")` +
`SetPlayerStartPos(102.78125, 74.156006, 37.494278)` (the holy site's own position, from
`Data/Levels/FinalAlbion/LookoutPoint.tng`). **Still exits silently at the same point.**
So the missing hero-placement hypothesis from `CGame::Play` is wrong, or at least not the
whole cause.

Diagnostics that produced nothing: `SetDisplayErrors(true)` raises no dialog, `bbb.log`
stays 0 bytes, and `WINEDEBUG=+debugstr` captures no `OutputDebugString` output at all —
so `LIB_ERROR`/`LIB_TRACE` do not route anywhere reachable from outside the process.
**The exit cannot be diagnosed from outside.** That is now the argument for doing the
in-process hook first and using it to answer this question.

#### Symbol resolution from `Ego_r.pdb` — solved, and it is the hook's foundation

`llvm-pdbutil` refuses the file ("Too many directory blocks"): block size is 1024 and the
stream directory spans 512 blocks, so the directory's block map needs 2 blocks, which LLVM
only supports as 1. The container is otherwise an ordinary MSF 7.00.

`packages/mirror/tools/pdbsyms.py` reads it directly and extracts **116,172 public
symbols** with RVAs. Anchors captured in `packages/mirror/tools/egor-anchors.txt`:

| RVA | Symbol | Use |
|---|---|---|
| `0x0073fc50` | `CConsole::RunTextCommand(const CCharString&)` | run any console command |
| `0x0073fdb0` | `CConsole::RunScript(const CWideString&)` | run a whole ini script |
| `0x006fa5b0` | `CCharString::CCharString(const char*, long)` | build the argument |
| `0x00016da7` | `CMainGameComponent::Update()` | per-frame hook point (virtual) |
| `0x000162ff` | `CMainGameComponent::Render()` | ″ |
| `0x00e6d184` | `GTakeScreenshot` (long) | write N to capture N frames |
| `0x00df8568` | `GAllowMovieRecording` (bool) | gate for the above |
| `0x00e6d15b` | `GSkipFrontend` (bool) | |
| `0x00e6d19d` | `GOverridePlayerStartPosFromConsole` (bool) | |
| `0x00e6d1a8` | `GOverridePlayerStartPos` (C3DVector) | |
| `0x00e6d1c4` | `GForceStartingHolySite` (CWideString) | |
| `0x00e6d1a5` | `GFreeCamOnWithPlayer` (bool) | |
| `~0x00df87e8` | 101 × `NGlobalConsole::*` bools | contiguous toggle block |

**Gotcha: the decomp's addresses are `FableWin.exe`'s, not `ego_r.exe`'s.** `CGame::Play`
is `LAB_00ac7d88`-adjacent in `~/git/fable-reimpl` but lives at RVA `0xf540` in `ego_r.exe`.
Always resolve against `Ego_r.pdb` for the binary we actually run.

Section base is `0x1000` (RVA = `.text` offset + `0x1000`); resolve in-process as
`GetModuleHandleW(NULL) + RVA`.

No `CConsole` instance appears in the public symbols, so the console object must be reached
another way (a static local, or a member of an owning object) — that is the one open
question for calling `RunTextCommand`. Writing the globals directly needs no instance and
no calling convention, so it is the safe first milestone.

#### Working practices

- **No fixed iteration cap on capture runs.** A timeout that kills the run mid-investigation
  destroys the state you were about to inspect. Loop until success or a stop file
  (`touch /tmp/cap/stop`), and make cleanup unconditional via a trap.
- **Never `pkill -f` / `pgrep -f` a pattern that also appears in your own command line** —
  the shell running the command matches and kills itself (exit 144). This cost several
  cycles. Write a PID file and kill that.
- Resolve `xdotool` once with `nix build --print-out-paths` and call the binary; `nix run`
  per call adds seconds to every loop iteration.
- Back up `dbug.ini`, `dbugst.ini` and `FSE/quests.lua` before touching them, and restore on
  exit via a trap.
- Register the capture quest by adding a `MirrorCapture` entry to `FSE/quests.lua`
  (`file = "MirrorCapture/MirrorCapture"`, unique `id`), and activate it from the console
  script with `ActivateQuest("MirrorCapture")`.
- The window title is `"Fable - The Lost Chapters "` — **with a trailing space**. An anchored
  `$` regex will not match it.

#### What is confirmed working

- The game boots, renders at the nested output's resolution, and reaches its main menu with
  the existing save profile visible.
- FSE attaches and initialises its API pointers.
- `grim` against the nested Wayland display captures the game's frames correctly.

**Cadence:** one step per working session, ending with a written summary of what landed, what
the evidence was, and what the next derivation note will cover. Steps 0 and 1.1–1.3 are
mechanical and run with a lighter touch; step 2 onward gets the full protocol.

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
| `openalbion/src/scene/terrain.rs` | `HEIGHT_SCALE = 2048.0` | unsourced — real scale is in `engine_landscape*.cpp` |
| `openalbion/src/scene/terrain.rs` | `CELL_SIZE = 1.0` | unsourced — ″ |
| `renderer/src/terrain.rs` | `texture_scale = 0.0625` | unsourced; moot once layer meshes supply `CliffU`/`CliffV` (step 5.2) |
| `renderer/src/terrain.wgsl` | placeholder light dir + `0.25/0.75` shade | placeholder; real form is `Ambient + saturate(n·l)²·Diffuse + max(−n·l,0)·Backlight` (step 5.4) |
| `renderer/src/model.wgsl` | placeholder light dir + `0.3/0.7` shade | placeholder; models not yet in scope |
| `renderer/src/model.rs` | `ALPHA_CUTOFF = 0.5` | unsourced |
| — | `LightArray` / `LightGlobals` offsets in the Lights layout | §3.8 open item; due when step 5.4 needs lighting |

Retired from this table: the sky dome's `36` segments (`engine_sky_renderer.cpp:616`,
`while (uVar13 < 0x24)`) and its `7000` / `−500` / `6500` extents are sourced and cited
in place.

---

## 10. History

- **2026-08-08** — **The renderer became its own crate, and the mirror was abandoned.**
  `packages/renderer` now depends on wgpu/glam/bytemuck/tracing alone: passes take
  `TerrainData`, `Model` and `TextureImage` instead of reaching into `Files`, `Lev`, `Mesh`
  and the `.big` archives, so a pass can no longer invent a data lookup (§11.1). Asset
  decoding and mesh building moved to a `scene` module. Verified as a visual no-op — three
  scenes rendered byte-identically before and after — using `mirror` immediately before
  deleting it. `openalbion` went back to being a plain binary. `packages/mirror` and
  `scenes.toml` are gone: driving the original engine for reference screenshots cost more
  than it returned, and §5 step 1 / §6 / §7.1 are now history rather than plan. A
  `world-edit` binary was tried and removed the same day: it bought duplicated
  `files`/`scene`/`camera` and nothing the engine could use, so world editing is deferred
  and will be a **mode inside `openalbion`** if it happens. Its terrain-pick ray march and
  tests are in `777bf94` if that mode ever wants them. The renderer crate was kept — it
  stands on its own (§11.2), independent of the editor question that prompted it.
- **2026-08-06** — **The dev build runs.** `FableWin.exe` was the wrong binary all along: it
  imports `MSVCR100D`/`MSVCP100D`, the non-redistributable VS2010 debug CRT with no Wine
  builtin, so the loader fails before any code executes — hence no window and no `bbb.log`.
  `ego_r.exe` imports only builtins and boots. Four further blockers solved
  (`SkipConfigDetection`, the `GFX_RESET` bad-exit registry flag, `AllowBackgroundProcessing`
  for headless focus, the single-instance mutex); the build now reaches a rendering 1280×720
  window with **zero dialogs, fully unattended**. Established that the dev console supersedes
  FSE entirely — free camera, `GlobalDrawGUI`, `TakeScreenshot` (gated on
  `AllowMovieRecording`, writes lossless TGA), and a much finer `Enable*` set that can turn
  the original *down* to what OpenAlbion actually draws. Remaining: the front end —
  `SetSkipFrontend` exits silently, and the dev front end's keyboard menu is unreachable
  because the headless wlroots seat has no keyboard (pointer works, which is why the retail
  mouse script does). Xvfb ruled out: no Vulkan for DXVK. Landed
  `packages/mirror/capture-dev.sh` and `packages/mirror/dev-build-config/`.
- **2026-08-05** — **Step 0 complete** on branch `renderer-refocus`: invented mechanisms
  stripped (`dc165b1`), raw non-sRGB colour space (`8e59ee5`), native Z-up (`99f0d8d`),
  unverified constants audited (§9), two self-inflicted bugs caught by smoke-running
  and fixed. Sky now renders its raw texture with zeroed gradients; landscape renders
  untextured flat-lit. Next: step 1 (tooling).
- **2026-08-06** — **Retail front end solved.** The game reads DirectInput *relative* mouse
  motion and tracks its own cursor, so absolute `mousemove` never worked; corner-slam plus a
  known delta does. Retail now runs unattended from launch to in-game (dialogs, splash, menu,
  save load) via `packages/mirror/capture-retail.sh`. Also established that retail lacks
  `SetSkipFrontend`/`TakeScreenshot`/`SetInputLoad`/`SetResolution` entirely — that command
  list is the dev build's. Remaining: activating the capture quest and the isolation commands
  *after* the save loads.
- **2026-08-06** — **Retail capture attempted for real.** Got the original launching,
  rendering and reaching its main menu under full automation in a nested headless
  compositor. Solved five environment blockers (steam-run, correct Proton, Steam Linux
  Runtime, DISPLAY vs WAYLAND_DISPLAY, EULA/warning dialogs) — all recorded in §7.1.
  Blocked on synthetic input being unreliable against DirectInput; the strong lead is that
  startup commands belong in `dbugst.ini`, not `dbug.ini`. Prefix was accidentally
  downgraded and rebuilt during this work (saves intact; EULA + graphics registry values
  lost and re-accepted). Install restored afterwards.
- **2026-08-06** — **Step 1.5: comparison landed.** `mirror compare` with SSIM-over-luma
  structure (gated, non-zero exit) and full-resolution colour (reported), plus an
  `a | b | heatmap` contact sheet. Exercised on the FOV correction, which it classified
  correctly as structural (0.896) with a modest colour shift, localising the change to the
  horizon band. Still to do in 1.5: the persistent Divergence Ledger file, and calibrating
  the gate against a real reference.
- **2026-08-06** — **Step 1.4 automated.** Found the game automates itself: `dbugst.ini`
  documents the startup console command set, including `SetSkipFrontend`,
  `SetPlayerStartPos` (which dissolves the region-residency problem that would otherwise
  need a hand-loaded save per region), `SetResolution`, `AutomatedMode` and `TakeScreenshot`.
  Verified nested headless sway brings up XWayland, and Proton can be launched directly
  without the Steam client. `mirror capture-prep` now emits a boot ini, an FSE quest and a
  one-command orchestration script. Four unknowns remain for a first real run (§7.1).
- **2026-08-06** — **Step 1.4 researched.** Established the full capture mechanism (§7.1):
  `dbug.ini` as the console-script entry point, `Enable*` for isolation, `PauseTime` +
  `SetTimeAsStopped` + `SetTimeOfDay` for the clock, FSE
  `CameraMoveToPosAndLookAtPos(from, to, duration)` for the camera, `grim -w` for the frame.
  `mirror capture-prep` now generates the prologue, the FSE quest and the checklist.
  Biggest find: **FOV is data and it is horizontal** (§3.10) — `CAMERA_MODE.FOV`, default 70,
  with `fovY = 2·atan(tan(HFOV/2)/aspect)` ≈ 43° at 16:9. We had been using 70° as a vertical
  FOV. Corrected, with the derivation as a unit test; it was never a constant to fit.
- **2026-08-06** — **Step 1.1–1.3 landed.** `openalbion` split into a library plus a thin
  viewer binary; shared scene assembly moved to `openalbion::scene`; `Renderer` gained an
  offscreen target and `render_to_image()`. New `packages/mirror` with a scene manifest
  (`scenes.toml`, 6 starter scenes), `mirror list` / `capture-prep` / `render`. Headless
  capture verified end to end against `~/Fable`. Remaining in step 1: reference capture
  (1.4), compare + ledger (1.5), probe (1.6), register map (1.7).
- **2026-08-06** — Tooling reassessed after Jamen rejected building `shot`/`probe` into the
  openalbion CLI. Established that the original engine is controllable here (§3.9): runs under
  Proton, per-subsystem `Enable*` console commands, FSE Lua camera + frozen clock. Replanned
  step 1 as `packages/mirror`, a comparison testbed with a scene manifest, structure-gated /
  colour-reported metric and a Divergence Ledger (§6). RenderDoc ruled out for capturing the
  original (32-bit PE vs x64 RenderDoc); numeric extraction routed through the existing ASI
  hook infrastructure as a spike.
- **2026-08-05** — Renderer review. Identified the LUT column model (§3.1), the landscape
  layer architecture (§3.4) and the sRGB mismatch (§3.5) as the three root causes behind the
  sky/terrain fix loop. Decoded the `.bbb` shader bank format (§3.7) and recovered the named
  shader constant register map (§3.8), cross-verified against the disassembly on six
  registers. Z-up settled. Document reconstituted; plan in §5 and tooling in §6 pending review.

---

## 11. Crate architecture

```
packages/
  lzo/          minilzo binding
  fable-data/   parsers: .lev .tng .wad .big .bbb, defs, textures
  renderer/     wgpu passes + the data they draw          ← no asset formats
  openalbion/   the engine recreation                     (bin)
  fool/         asset CLI                                 (bin)
```

**One game binary.** `openalbion` is the project. World editing, when it happens, is a
**mode inside it** rather than a second binary — decided 2026-08-08 after briefly trying
the split, which only bought duplicated `files`/`scene`/`camera` for no gain the engine
could use. The renderer stayed a crate on its own merits (below); nothing else needed to.

### 11.1 The renderer's input boundary — *why this shape*

`renderer` depends on **wgpu, glam, bytemuck, derive_more, tracing** and nothing else. It
cannot open a file, does not know what a `.big` is, and has never seen a `Lev`. Every input
arrives as a plain struct:

| Type | Is |
|---|---|
| `TextureImage` + `ImageFormat` | decoded pixels or BCN blocks, ready to upload |
| `TerrainData` | vertices, indices, one image per theme layer, palette→layer map |
| `Model` / `ModelPrimitive` / `ModelMaterial` / `AlphaMode` | geometry + materials |
| uniform setters (`update_sky_uniforms`, …) | the shader constants, by name |

**This is the structural answer to §0.** The fix loop happened because passes were doing
their own data lookups and getting them wrong — §4's pattern, *"the geometry is roughly
right and the data lookup is invented."* A pass handed a finished `TerrainData` has nothing
left to invent, so ground rule 3 ("verify before rendering") is now enforced by the
compiler rather than by a reviewer noticing.

It also buys the test layers §6.9 asks for: mesh builders and value lookups are ordinary
unit tests needing **no GPU and no Fable install**, and the renderer can be exercised
headlessly (`Renderer::new_headless` + `render_to_image`) against hand-written constants.

**Rule: nothing in `packages/renderer` may depend on `fable-data` or touch the filesystem.**
If a pass needs a number, it arrives as a parameter. That is the whole invariant.

### 11.2 Why a crate and not a module

`renderer` is the one split worth having, and it earns it three times over:

- **It is its own compilation unit.** Renderer edits stop rebuilding through `fable-data`,
  which pulls `fable-defs` from git.
- **It is enforceable.** A module can quietly `use crate::files`; a crate cannot, because
  the dependency is not in its manifest. §11.1's invariant is checked, not remembered.
- **It is self-contained.** The wgpu surface, the passes and their inputs are one thing
  with one boundary, and can be read without the engine around them.

`openalbion::scene` stays a module of the binary. It is the conversion layer — Fable's
assets in, renderer inputs out — and per §6.8 it is where provenance logging belongs:
*"which byte became which constant"* is a conversion question, not a shader one. It has
exactly one consumer, so it has no reason to be a crate.
