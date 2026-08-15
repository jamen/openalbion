# OpenAlbion — agent guide

A recreation of Fable: The Lost Chapters' engine, ported from oracles rather than guessed at.
This document is the working agreement: what is settled, what is next, and how to tell the
difference between the two.

**It is current, not cumulative.** A fact belongs here stated as a fact, in the present tense,
with its citation — not as a story about when it was discovered. §10 keeps the session record,
compressed to what each one *corrected*, and is the only place narrative lives.

**A note on section numbers.** Source comments cite this document heavily (~90 references).
Numbering is therefore stable and is not renumbered on edit. One inherited ambiguity: in code
comments, `§N.M` for `N ≥ 5` usually means *step N, item M* of §5's subsystem list — `§6.12` is
step 6's item 12 (`.wld` world placement), not §6 item 12. §6.3/§6.6/§6.7/§6.8/§6.9 are the
exception and do mean §6's own subsections.

---

## 0. Where we are

Four subsystems draw a level today. The renderer is faithful where it has been derived and
inert where it has not — deliberately, per §2 rule 5.

| Subsystem | State | §  |
|---|---|---|
| Landscape (foreground) | **Built.** Triplanar layers over normal-indexed blend tables, additive over a blackout pass | §3.4, step 5 |
| Things (static meshes) | **Built.** `.tng` → def `Graphic` → `graphics.big` asset, instanced | §3.11, step 6 |
| Texture sampling | **Built.** Shipped mip chains, anisotropy 4, MSAA 4× | §3.12, step 7 |
| Local detail (foliage) | **Built.** Generated, deterministic, exactly reproducible | §3.13, step 8 |
| Sky | **Partial.** Textures, keyframe blend and the gradient lerp are right; the gradients themselves are zero pending the environment layer, and the base band draws through the wrong pipeline | §3.2, §3.3, step 3 |
| Environment layer | **Half.** Themes and keyframes are read; **the colour LUT is not** — `lighting_colours.tga` is loaded into `Files` and consumed by nothing | §3.1, step 2 |

**Two things gate everything else, and they are independent.**

1. **The environment layer (§5 step 2)** is the highest-value *faithfulness* work left. Sky
   gradients, landscape lighting and mesh lighting are the same four LUT rows, all three
   currently running the same neutral placeholder (§9). It is now **one constant** to change —
   `LightingUniforms::NEUTRAL` — since §12.7 landed.
2. **Bindless rendering (§12)** is the highest-value *engineering* work left, and is what
   Jamen has asked for next. It is a prerequisite for font rendering and the dev console
   (§13), and it pays for itself independently by giving the renderer a texture cache it does
   not have today.

**Bindless comes first**, then fonts and the console, then the environment layer — Jamen's
call, 2026-08-15.

**A trap that cost a session, now fixed and worth not reintroducing:** `cargo check` does not
link, so a broken host link can hide indefinitely. It did — `mingw.stdenv.cc` in the devshell's
`packages` exported a bare `CC`/`AR` for the Windows target, and `packages/lzo` built a COFF
object the Linux host link could not resolve. **Run `cargo build`, not just `cargo check`.**

---

## 1. The oracles

Ranked by authority. Higher entries override lower ones. Never guess when an oracle covers
the question.

| # | Resource | What it settles | Notes |
|---|---|---|---|
| 1 | `~/git/fable-reimpl/src/**/*.hpp` | Class layouts, field names, offsets, method signatures, **shader constant register maps** | Generated from CodeView debug info. **Authoritative** — if a decompiled body contradicts a header, the header wins. |
| 2 | `~/dl/Fable Shader Disassembly.md` | Exact per-vertex/per-pixel maths | 465 shaders, complete. Grouped by `## SHADERS_*`, each in a `<details><summary><strong>NAME</strong>` block. Provenance unverified; §3.7 says how it could be regenerated. |
| 3 | `~/doc/.../Fable/Data/shaders/*.bbb` | The shaders as the game ships them: name, bytecode, source path | Compiled banks. Format decoded in §3.7. |
| 4 | `~/doc/.../Fable/Data/Defs/*.def` | Real authored data (rows, columns, textures, tuning) | Debug build ships text `.def`; retail ships only `CompiledDefs/*.bin`. |
| 5 | `~/git/fable-reimpl/src/**/*.cpp` | Algorithms, control flow, constants | Ghidra output — noisy. Trust literals and structure; be sceptical of types and register artifacts. |
| 6 | `~/git/fable-decomp` | Raw decomp inputs / sidecars | For when #5 is unreadable. |
| 7 | `~/Fable`, `~/doc/Fable_Anniversary-2013-02-25` | The engine, running | §3.9. Expensive; nothing currently depends on it. |

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
~/Fable/data/                               # retail TLC art; the Anniversary tree symlinks to it
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

These exist because of a specific failure. Two subsystems were once tuned by eye through a
long `fix:`/`diagnostic:`/`debug:` loop — constants adjusted inside an architecture that had
been *guessed at*, with no way to tell whether the mesh, the lookup, the shader or the colour
space was at fault. Four oracles already answered every one of those questions. The rules are
what stop that recurring.

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

Rules 1 and 3 do not apply to §12 and §13: bindless rendering and font rendering have no
oracle, because the original engine did neither. Rules 2, 4, 5 and 6 carry over unchanged, and
rule 4 there means *pixel-identical*, not *looks right*.

---

## 3. Verified ground truth

Read out of the oracles, with exact citations. This is the knowledge base — everything
below is a present-tense fact about *the game*, not about our progress.

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

> **The reading to avoid**, because it was implemented once and cost weeks: treating X as
> time-of-day and bilinearly interpolating along it. That walks across *other themes'* columns
> as the clock advances and filters between unrelated themes, and no row-index tweak or alpha
> clamp rescues it. It was deleted rather than tuned (§2 rule 5); the gradients read zero
> today.

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

Against `renderer/sky/outer_sky.wgsl`: the vertex-side gradient lerp is a correct
transcription, and `c0.w` is fed the real blend factor from
`CEnvironmentThemeDef::SkyTexture1Blend` and the keyframe blend. The gradients behind
`c92`/`c93` are zero until step 2.

`VSHADER_SKY_BASE_BAND` is **`mov oD0, c92`** — flat top-gradient colour, no texture, no
vertex colour. The base band is still drawn through the *outer sky* pipeline, which samples
sky textures. Wrong shader; step 4.1.

`VSHADER_SKY_SPRITE` is `mov oD0, v1; mov oT0, v2` + the standard transform — matches our
`sky_sprite.wgsl`.

### 3.4 Landscape — triplanar layers over a normal-indexed blend table

> **Two readings to avoid, both of which cost weeks once.** `CliffU`/`CliffV` are *not*
> texture coordinates, and the second texture stage is *not* a composited surface. The
> composited-surface story belongs to the **background** LOD path, which we do not implement.

`CLandscapeLayerMesh::CVertex` (`engine_landscape_layer_mesh.hpp:71`):

```cpp
unsigned char X;      // grid position within patch
unsigned char Y;
unsigned char Blend;  // this layer's theme weight at this vertex
unsigned char CliffU; // NOT texture coords — the vertex normal, packed
unsigned char CliffV;
```

`BuildMapDirMask` (`engine_landscape_mesh_builder.cpp:623`) fills them with
`round((normal.x * 0.5 + 0.5) * 255)` and the same for Y, and
`BuildBlendingTables` (`engine_landscape.cpp:826`) builds **five 128×128 alpha tables**, one
per `LANDSCAPE_TEXTURE_MAPPING_DIRECTION`, holding `GetMappingDirectionBlend(dir, normal)`
for the normal each texel decodes to. `RenderForeground` (`engine_landscape_patch.cpp:1076`)
binds `ForegroundBlendTables[layer.MappingDirection]` to **stage 0** and sets `c40`/`c41`
from `PositionToTextureUVTransformU/V[layer.MappingDirection]`.

So the foreground pass is:

```
oT0 = (CliffU, CliffV)          → blend table: how much this DIRECTION applies here
oT1 = (pos·c40, pos·c41)        → planar projection along that direction
oD0.w = fade(distance) × Blend  ; c42 = ForegroundFadeTransform
oD0.rgb = Ambient + saturate(n·l)²·Diffuse + max(−n·l,0)·Backlight

colour = t1 (the layer's own ground texture) × oD0.rgb × 2
alpha  = t0.w (blend table) × oD0.w
```

**This is triplanar mapping**, not a composited surface. `CEngineSurfaceCompositionManager`
is a character/mesh texture cache and has nothing to do with it; the thing it was confused
with is the *background* patch's procedural texture (`RenderProceduralTexture` +
`PSHADER_LANDSCAPE_PROC_TEXTURE`), which is the distant-LOD representation — the same layers
rendered top-down into a per-patch target so far terrain draws in one pass.

**The layers composite additively over a blackout pass, not source-alpha-over.**
`VSHADER_LANDSCAPE_FOREGROUND_BLACKOUT_PASS` is the same transform ending in
`mov oD0.xyzw, c0.x` — solid black — drawn over the same layer meshes first, laying down
depth and the origin the layers accumulate onto. Because the alphas sum to exactly 1 at every
point, `Σ αᵢ·colourᵢ` is a true weighted average that covers completely; source-alpha-over
would give `1 − Π(1 − αᵢ)` and leak the background wherever no single layer is at full
strength. That is *why* the theme weights are renormalised to 255 and why the five direction
blends partition unity — both are preconditions for additive compositing, and getting it
wrong showed up immediately as sky bleeding through the seams between themes.

Per vertex, `ReadThemesAndCreateLayers` (`:798`) turns each of the three themes into a `TOP`
layer from the theme's **base** texture and four cliff layers (`FRONT`/`BACK`/`LEFT`/`RIGHT`)
from its **cliff** texture, dropping any whose weight is ≤ 16 and merging layers that share a
`(direction, texture set)`. The five directions' blends **sum to 1 for any normal** — that is
what keeps the `mul_x2` from double-darkening a slope, and it pins the reading of the
`asin`/`acos` arguments the decomp cannot show.

Patches are 16×16 cells / 17×17 vertices (`PolyMaskGrid[16][16][2]`, `VertexBlend[17][17]`,
`PatchGridWidth = MapWidth >> 4`). Layers are a linked list per patch, each with its own
vertex/index buffer.

**The `.rdata` tables** (`MappingDirNormals`, `PositionToTextureUVTransformU/V`,
`MappingDirectionToUVLocalisationTableIndexForU/V`) are initialised data and so absent from
the decomp. They were read out of `ego_r.exe` via `Ego_r.pdb`. **The extraction script and its
write-up (`tools/pdbsyms.py`, `tools/landscape-statics.md`) were never committed and are gone**
— `fable-data/src/landscape/mod.rs` still cites them and the RVAs it quotes (`.data`
`0x00e188d0`) are the surviving provenance. `MappingDirNormals` is
`TOP=+Z FRONT=−Y BACK=+Y LEFT=−X RIGHT=+X` (a third confirmation of Z-up), and the UV
transforms put **one texture tile across 8 world cells**.

**The `.lev` side** carries everything needed, with three traps, all now handled in
`fable-data/src/landscape/`:

- **world Y is the file's row index, with no flip** — and the obvious reading of the decomp
  says otherwise, which is the trap. The accessors do read `(SizeY − y) * (SizeX + 1) + x`
  (`map.cpp:2135`, `map_render.cpp:119`), but the **load loop applies the same flip on write**
  (`fablelib/map.cpp:2600`): it walks the file sequentially and stores file row `f` at array
  row `SizeY − f`. Reading world `Y` from array row `SizeY − Y` therefore returns file row
  `Y`. `(SizeY − y)` describes the engine's in-memory layout, not the file's. We keep cells in
  file order, so applying the read-side flip alone mirrors the landscape in Y — and a mirrored
  terrain is self-consistent and invisible until something else is drawn in world space to
  disagree with it. `.tng` placements are that witness (§3.11), and `placement_test.rs` keeps
  it a gate;
- height is `<file f32> * 2048.0` on load (`fablelib/map.cpp:2594`) then quantised to 1/128
  by `PeekLandscapeHeight`;
- the theme palette's stored def index is **stale** in retail data (off by a constant 702 for
  every LookoutPoint entry). `CMap::LoadFromFile` re-resolves it from the entry's *name*
  (`GetDefGlobalIndexFromName`, `:2561`), and so must we. Resolving by index finds nothing,
  which is why the landscape was untextured for so long.

**Retail does not build this at runtime.** `CEngineLandscapeMap::OpenStaticMap` streams
precomputed patches out of `FinalAlbion_RT.stb`, which is a `BBBB` bank container (same magic
as the shader banks, §3.7); `CEngineLandscapeMeshBuilder` is the editor/dynamic path.
**We port the builder**, deliberately: we already parse `.lev`, and the `.stb` is only a cache
of what the builder computes. A partial `.stb` directory parser exists in this repo's history
(`git show 1079634:fable_data/src/stb/mod.rs`) if that ever changes.

**Filler levels** are ordinary `.lev` files, classified per region in `FinalAlbion.wld` as
`ContainsMap` (loaded, walkable) or `SeesMap` (visible only). The mechanism is
`CEngineStaticMapEdgeHeights`: `PeekThemeId`/`PeekThemeBlend`/`PeekLandscapeHeight` first try
the loaded map, then walk into the **neighbouring** map through `CEngineWorldMap`'s 32×32-cell
tile grid, and only fall back to a **4-cell border** of cached edge data when a map has no
`GameMap` — i.e. is not loaded. An unloaded filler's interior is never sampled; its geometry
comes from the static map file. Sampling clamps to the *cell* grid, one smaller than the
vertex grid, so the far seam row and column belong to the neighbour and are unreachable while
we load one map at a time.

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

**Verdict:** worth a small extractor — half a day, and it would de-risk oracle #2 for every
subsequent subsystem. Not worth more than that, and nothing currently depends on it.

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

Read off the constructor body (`engine_vs_layout_lights.cpp:56-90`):

| Range | Register | Value |
|---|---|---|
| `LightArray` | `c19`–`c30` | `.Offset = 0x13`, `.Count = 0xc`, `LightSize = 2` → 6 lights × 2 |
| `LightAttenuations` | `c31`–`c34` | `.Offset = 0x1f`, `.Count = 4` |
| `LightGlobals` | `c35` | `.Offset = 0x23`, `.Count = 1` — **the backlight** |
| `ShadowedSpotlightAttenuation` | `c36` | `.Offset = 0x24` |
| `ShadowedSpotlightColour` | `c37` | `.Offset = 0x25` |
| `User` | `c38`–`c95` | `.Offset = 0x26`, `.Count = 0x3a` |

So `c19` = light[0] direction, `c20` = light[0] colour, and lights 1–5 follow at
`c21`/`c22`, `c23`/`c24`, …

Cross-checked three ways: `VSHADER_STATIC_DIRLIGHT_2POINTLIGHTS` reads its two point lights
at `c21` and `c23`, exactly light[1] and light[2] ✓; its `mov r8, c32` lands inside
`LightAttenuations` ✓; and `CShaderRenderManager::InitialiseLight`
(`lib_shader_render_manager.cpp:1686`) writes an attenuation only when `index != 0`, i.e.
**light 0 is the directional light and has none** ✓.

The landscape and static mesh passes both name `c19`/`c20`/`c35` on this basis, and they are
no longer a hypothesis.

### 3.9 The original engine as a reference

Not a plan, and nothing currently depends on it. The comparison harness that used this
(`packages/mirror`) was abandoned 2026-08-08 and deleted, along with `capture-retail.sh`,
`capture-dev.sh` and `pdbsyms.py`. What survives is the knowledge, because rediscovering it
cost days: which binary, how to launch it, what its console can do, and where the symbols are.

**`ego_r.exe` is the reference engine** — the dev *release* build in
`~/doc/Fable_Anniversary-2013-02-25/Fable/`, 16 MB, with `Ego_r.pdb` (134 MB) beside it.
**Not `FableWin.exe`**: its PE imports name `MSVCR100D.dll`/`MSVCP100D.dll`, the VS2010 *debug*
CRT, which is non-redistributable and has no Wine builtin — the loader fails before any code
runs, with no window and no log. `ego_r.exe` imports the release CRT and `d3dx9_43.dll`, all
Wine builtins under Proton Experimental, and carries the same full command set.

**The Anniversary tree is dev engine + retail TLC art.** Its `Data/` symlinks `graphics.big`,
`pc/textures.big`, `pc/frontend.big`, `shaders/pc/shaders.big` and the English text at
`~/Fable/data/...`; native to the tree are `FinalAlbion.wad`/`.stb`, `meshdata.bbb`,
`CompiledDefs`, `Defs`, `LightingTable`. The art is **not** remastered, so colour comparison
against this build would be valid, not merely structural — and `~/git/fable-reimpl` is a decomp
of this same binary, so oracle and reference would be the same thing.

**Launching it (Linux/NixOS), five things that each broke it:**

1. **NixOS needs `steam-run`** — `proton` and `pressure-vessel-wrap` are dynamically linked
   for a generic distro.
2. **Use the Proton the prefix was built with.** `compatdata/204030` is at `11.0-100` =
   Experimental. An older Proton rebuilds the prefix and silently discards the EULA acceptance
   and graphics-detection registry values.
3. **Go through `SteamLinuxRuntime_4/_v2-entry-point`** — Experimental's `toolmanifest.vdf`
   requires appid 4183110. Bare Proton gives a Wine with no FreeType and every dialog renders
   as an empty stub.
4. **Set `DISPLAY`, not `WAYLAND_DISPLAY`** (Wine is X11), and *unset* `WAYLAND_DISPLAY` so the
   window cannot escape onto the real desktop.
5. **Nested headless sway works** and provides XWayland. The prefix must live inside Steam's
   `compatdata` tree — pressure-vessel does not map `/tmp`.

**Five more for the dev build specifically**, each of which also bit:

1. Both `default_userst.ini` and `userst.ini` are read, in that order, so the latter overrides.
2. `SkipConfigDetection(TRUE)` stops `ConfigDetect.dll` loading and the "0MB RAM" warning.
3. The "did not exit correctly" dialog is a **registry flag**, not an error: `GFConfigDetection`
   writes `HKCU\…\Fable TLC\GFX_RESET = 1` every boot and clears it only on a clean exit
   (`main.cpp:281`), so a harness that kills the game re-arms it every run.
4. **`AllowBackgroundProcessing(TRUE)` is mandatory headless.** `main.cpp:1142` sets
   `WaitWhileInactive = !GAllowBackgroundProcessing`; without it the game ignores all input and
   exits after ~7 minutes.
5. **One instance per prefix.** `WinMain` takes a global mutex (`main.cpp:363`) and a second
   instance `return 0`s immediately — no window, no log, indistinguishable from a crash.

With those in place it boots **fully unattended to a rendering 1280×720 window, zero dialogs**.

**The console vocabulary is the durable prize, and §13.5 should borrow from it.** Every engine
component has a `CEngineComponent::GetConsoleEnableFunctionName()`. Retail `Fable.exe` has
`Enable{Sky,Landscape,Water,Weather,Shadows,AnimatedMeshes,StaticMeshes,RepeatedMeshes,Sprites,
SpriteTrails,Decals,GroupDecals,Lines,Primitives,ChangingPrimitives,FlareSprites,WeaponTrails,
Sounds}` and `EnableScreenEffect{ColourFilter,GlowRenderer,OutlineGlow}`. `ego_r.exe` adds
`Enable{Clouds,Sea,Textures,CompositeTextures,LandscapeBumpMapping,LandscapeFog,
LandscapeTesselation,LandscapeLODUpdate,Glow,Particles,ParticleRendering,LocalLights,
ShadowedSpotLights,Dithering,MouseCursor,Foreground,Background,EngineScreenshotMode}`, the free
camera as console commands (`SetFreeCam`, `SetFreeCamPos`, `SetFreeCamLookVector`,
`SetFreeCamFOV`, `SetFreeCamHeightLock`, `FreeCamOnWithPlayer`), `GlobalDrawGUI` for the HUD,
`TakeScreenshot` (gated on `AllowMovieRecording`; writes `data\movies\shot%06d.tga` off the back
buffer), `SetResolution`, `SetMaxTextureSize`, `PauseTime`, `SetTimeOfDay`, and input
record/playback (`SetInputSave`/`SetInputLoad` + `AutomatedMode`).

> **`EnableLandscapeTesselation` and `EnableLandscapeLODUpdate` being separate toggles is why
> §5 step 5.6 is a supported configuration rather than a shortcut.**

**Retail has no working console.** `ConsoleAlpha`/`ConsoleListContaining` are remnants of one
that was stripped — long-established in the modding community, confirmed by Jamen 2026-08-06.
Do not chase it.

**Symbol resolution.** `llvm-pdbutil` refuses `Ego_r.pdb` ("Too many directory blocks": block
size 1024, stream directory spans 512 blocks, so its block map needs 2 and LLVM supports 1); the
container is otherwise an ordinary MSF 7.00, and a direct reader extracted 116,172 public
symbols. **That reader was never committed and is gone** — this table is what survives of it:

| RVA | Symbol |
|---|---|
| `0x0073fc50` | `CConsole::RunTextCommand(const CCharString&)` |
| `0x0073fdb0` | `CConsole::RunScript(const CWideString&)` |
| `0x006fa5b0` | `CCharString::CCharString(const char*, long)` |
| `0x00016da7` / `0x000162ff` | `CMainGameComponent::Update()` / `::Render()` — per-frame hook points |
| `0x00e6d184` / `0x00df8568` | `GTakeScreenshot` (long) / `GAllowMovieRecording` (bool) |
| `0x00e6d15b` | `GSkipFrontend` (bool) |
| `0x00e6d19d` / `0x00e6d1a8` | `GOverridePlayerStartPosFromConsole` / `GOverridePlayerStartPos` |
| `0x00e6d1c4` / `0x00e6d1a5` | `GForceStartingHolySite` / `GFreeCamOnWithPlayer` |
| `~0x00df87e8` | 101 × `NGlobalConsole::*` bools, contiguous |

RVA = `.text` offset + `0x1000`; resolve in-process as `GetModuleHandleW(NULL) + RVA`.
**The decomp's addresses are `FableWin.exe`'s, not `ego_r.exe`'s** — always resolve against
`Ego_r.pdb` for the binary actually run. No `CConsole` instance is in the public symbols, so
writing the globals directly is the safe first milestone; calling `RunTextCommand` needs the
object found another way.

**Where it stopped, and what would unblock it.** `SetSkipFrontend(TRUE)` makes `ego_r.exe` exit
silently ~5 s after the window appears — reproduced 4×, with and without a profile, nothing
logged, and not diagnosable from outside (`bbb.log` stays empty and `WINEDEBUG=+debugstr`
captures nothing, so `LIB_ERROR`/`LIB_TRACE` route nowhere). `ShowDevFrontEnd TRUE` *does* reach
a keyboard-driven menu, but **no synthetic keystroke lands**: wlroots' headless backend creates
no input devices, so the seat advertises no keyboard and XWayland gets none. XTEST *pointer*
events still work. Xvfb has keyboard support but no Vulkan, so DXVK cannot create a device.
The two real leads are (a) nest sway inside the real session with `WLR_BACKENDS=wayland` so it
inherits a seat — needed once, if it creates a persisting Debug Profile — or (b) record the
front-end walk once on a real display and replay it with `SetInputLoad` + `AutomatedMode`.

**Capture.** `grim` (Wayland) and OBS are installed. **RenderDoc cannot help**: 1.45 is x64-only
and Vulkan/GL/GLES-only, and Fable is a 32-bit PE — so shader constants would have to come from
an in-process hook (the route `DebugThingListHook.asi` already proves in this setup), not a
graphics debugger.

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
this was never a constant to fit against a reference, it was a
constant to read. `Camera::fov_y` (`openalbion/src/camera.rs`) derives it, with the
derivation as a unit test; `Camera::fov_h` holds the horizontal value.

Two residuals, both flagged on `Camera::fov_y`:
- The argument to `_CItan()` is register-passed and invisible in the decomp, so half-angle vs
  full-angle is inferred rather than read. Half-angle is near-certain: full-angle would make
  `ScaleX = 1/tan(70°) = 0.36`, an implausibly wide view, where half-angle gives 1.43.
  A reference capture would settle it definitively, but there is no capture route any more
  (§5 step 1), so this stands on the arithmetic alone.
- *Which* camera mode is active in a given shot is not yet established, so `fov_h` defaults
  to `CAMERA_MODE_TEMPLATE`'s 70 and should be read from `camera_mode.def` per mode once the
  camera system lands.

`Use2DFOV` selects the second path, where vertical FOV is independent — that is the 2D/UI
camera (`ENGINE.FOV_2D`) and does not apply to the world camera.

### 3.11 Things — placing a level's static meshes

**`c5..c8` carries a rotation-only view, and the camera translation is applied per-geometry.**
Three shaders pin this between them and only one reading satisfies all three:
`VSHADER_OUTER_SKY` transforms a dome built around the origin with no translation term, so it
can only end up around the viewer if the view matrix has none; `VSHADER_LANDSCAPE_FOREGROUND`
supplies the translation itself with `add r1, r0, -c4` (`c4` = `CameraPos`);
`VSHADER_STATIC_DIRLIGHT` has neither, because `CalcObjectMatrix` has already put the mesh in
the world and `CombinedProjectionMatrix = Projection × View × World`
(`CShaderRenderManager::UpdateWorldTransform`, `lib_shader_render_manager.cpp:2960-3090`).

> **So there are two matrices, and mixing them up is silent.** Feeding a camera-relative pass a
> view-projection that already contains the camera translation, while its shader subtracts `c4`
> as well, displaces that geometry by `-camera_pos` every frame — and nothing complains unless
> something else is drawn in world space to disagree. `Camera` carries both, and a unit test
> asserts a world point lands on the same pixel through either.

**The object matrix.** `CEngineInternalPrimitiveMeshBase::CalcObjectMatrix`
(`engine_primitive_manager_mesh_base.cpp:557`) writes a row-vector `CMatrix3x4`:

```
E11..E13 = -scale * (Forward × Up)      E21..E23 = -scale * Forward
E31..E33 =  scale * Up                  E41..E43 =  position
```

`GetWorldPosition` (`engine_primitive_manager_static_meshes.cpp:815`) reads `E41..E43` back as
the thing's world position, which pins the translation to the fourth row and so the whole
convention. As columns of a column-major matrix: object **+X → Up×Forward, +Y → −Forward,
+Z → +Up**. That triple is right-handed — `(U×F)×(−F) = U` — which is the check that the sign
reading is not inverted. `scale` multiplies the rotation and not the translation.

**Mesh coordinates are 100× world coordinates.** `CTCGraphicAppearance` builds the scale it
hands the primitive as (`fablelib/tc_graphic_appearance.cpp:4658`, and seven other sites in
that file):

```c
fVar4 = (this->MainGraphic).RenderSizeX * this->Scale * (float)9.999999776482582e-3;
```

`9.999999776482582e-3` is `0.01f`; `RenderSizeX` is the def's `Graphic.RenderSizeX` (default
1.0) and `Scale` is the appearance scale, 1.0 in its constructor (`:526`), which a thing's
`ObjectScale` multiplies. So **`scale = RenderSizeX × ObjectScale × 0.01`**. It is not a fudge:
`MESH_SMALL_WALL_CURVED_POST_01` is 176 units tall in the file and 1.76 world units on the
ground, where one landscape cell is 1.0.

**Resolution is `DefinitionType` → def `Graphic` → `graphics.big` asset id.** Four def types
carry an `EngineGraphic` and between them cover every placed thing that draws: `OBJECT` (2,849
defs), `CREATURE` (517), `BUILDING` (321), `MARKER` (57). `Graphic.BankIndex` is an **asset id**,
not an index and not a symbol name — verified across three levels, where every non-zero index
resolves to a mesh-typed asset (Witchwood 38/38, LookoutPoint 192/192, Arena 57/57). This is why
the text `objects.def` bridge is gone: retail `game.bin` has always had the answer.

**`.tng` placements are an independent witness to the landscape's shape**, and the most
valuable one we have: things were authored standing on the ground, so "do the placements sit
on the terrain" tests the *terrain*, not just the placements. It is what caught the row-order
flip above. With the corrected indexing **90 %** of LookoutPoint's placements sit within a
metre of the ground and the mean deviation is **0.29**, where the flipped reading managed
28 % and 2.85. `packages/openalbion/tests/placement_test.rs` keeps this as a gate.

One cell is one world unit, confirmed: `CMap::GetThemeSizeZAt(C3DVector)`
(`fablelib/map.cpp:2143`) truncates a world position straight to a `C2DCoordI` cell index
with no division. `HEIGHT_SCALE = 2048` is confirmed a second time by the load loop, which
multiplies each cell by `2.048e3` as it reads it (`map.cpp:2600`).

**Meshes are wound clockwise-front.** Measured from the meshes' own normals, which need no
oracle: for each triangle, compare the right-hand-rule normal of its winding against the
average of its three vertex normals. Over 400 meshes from `graphics.big` they **disagree on
385,787 triangles and agree on 1,082** — all 400 are CW-front. That matches D3D9's default
`D3DCULL_CCW` (cull the counter-clockwise side). wgpu defaults to `FrontFace::Ccw`, so the
model pass was culling front faces and drawing the interiors. The landscape never caught it
because it draws with `cull_mode: None`.

> This is the check to reach for whenever winding is in question — it is a property of the
> data, needs no reference image, and gives a number rather than an impression.

**Half of every static mesh's triangles are degenerate.** 474,048 of 998,466 emitted
triangles have two equal indices — strip-stitching artefacts. `PrimitiveBlock`'s
`degenerate_triangles` flag is **clear on all 917 blocks** measured, so `expand_block` never
drops them; they rasterise nothing, so this is waste rather than a defect, but it doubles
every index buffer. Dropping index-degenerate triangles unconditionally is safe and is worth
doing. (`primitive_count` *is* the triangle count: for static meshes, `Σ(count + 2)` over
strip blocks equals the declared `index_count` on every primitive measured — the only
mismatches are animated meshes, which carry no static blocks.)

**Two measurements that changed the code:**

- **UVs are not clamped.** 501 of 1500 meshes sampled out of `graphics.big` carry UVs outside
  `0..1`. D3D9's default addressing is WRAP; the model sampler was `ClampToEdge`, smearing a
  third of the library.
- **`Material::base_texture_id` is a global asset id**, not an index into the mesh asset's own
  `texture_ids`: 1412/1412 non-zero ids resolve directly in `textures.big`, the indexed reading
  resolves 0. Also, 441 of 1853 materials have `base_texture_id == 0`, so "no texture" is the
  normal case and must draw white rather than drop the mesh.

`Mesh::transform_matrix` is identity on all 1500 meshes sampled, so ignoring it is safe.

### 3.12 Texture sampling — mip chains, anisotropy, and where MSAA stands

> **The lesson worth keeping.** Distant terrain aliasing had three separate causes, and the
> parser had the data for two of them all along — the accessor's name, `get_top_mip_*`, made
> the loss look intentional. **A field that is parsed and read by nothing**
> (`TextureMetadata::mip_maps`) **is the tell.**

**The engine samples the mip chain that ships in the archive — it does not build one.**
`CTextureManager::ReduceMipmapLevel(CGraphicFrame, CManagedTexture, ulong skip_levels)`
(`bbblibrary/lib_texture_manager_2.cpp:1568`) implements `SetMaxTextureSize` by *dropping N
top levels*: it halves width/height `skip_levels` times, allocates a texture with
`GetNoLevels() - skip_levels` levels, and copies surface levels down. No resampling — the
levels already exist. (`CTexture::GenerateMipmaps` / `GenerateMipmapsWithGPU`,
`lib_texture.hpp:1146-1147`, are the render-target path, not the asset path.)

`CTextureManager::CalculateTextureSize` (`lib_texture_manager_2.cpp:1976`) is the
chain-layout oracle. Ghidra mis-names the parameters — they are `(width, height, with_mips,
format)`:

```c
if (fourcc == 'DXT1')                { blockmin = 4; bpp = 4; }
else if (fourcc == 'DXT3' || 'DXT5') { blockmin = 4; bpp = 8; }
else                                   blockmin = 1, bpp = GetColourDepth(fmt);
for (; w != 0 || h != 0; h >>= 1) {
    total += max(w, blockmin) * max(h, blockmin);
    if (!with_mips) break;
    w >>= 1;
}
return total * bpp >> 3;
```

> **Every level's dimensions clamp to a 4-pixel minimum for DXT, and the chain runs to 1×1.**

**Measured: the chains are present, complete, and only level 0 is compressed.** Over 4,000
of `textures.big`'s 6,324 texture assets (`graphics.big` carries **zero** — every texture
lives in one archive), `raw_image_data.len()` equals the exact chain sum above for the
declared `mip_maps` on **3,978**; 7 are genuinely top-only; 12 are the special cases below.

```
mip_maps histogram: {1:11, 2:39, 3:59, 4:406, 5:361, 6:421, 7:855, 8:1841, 9:4, 10:3}
```

`Texture::parse` LZO-decompresses the top mip and appends the remaining input **raw**, so
the totals matching exactly proves **levels 1..n are stored uncompressed and are already in
`raw_image_data` today**. `get_top_mip_bcn_image` slices `..top_mip_length` and throws them
away; `TextureMetadata::mip_maps` is parsed and read by nothing. There is no decode work to
add — this is a slicing-and-upload change.

**Three data caveats, all measured, all handled in step 7:**

- **`dxt_compression` is not uniformly a BCn tag.** Distribution over all 6,324 assets, with
  bits-per-pixel implied by `top_mip_map_size / (padded w·h)`:

  ```
  dxt=31  bpp=4    3683  DXT1 (BC1)     dxt=1   bpp=32        6  uncompressed 32bpp
  dxt=32  bpp=8    2558  DXT3 (BC2)     dxt=1   bpp=256/2048  2  degenerate headers
  dxt=35  bpp=8       4  DXT5 (BC3)     dxt=24  bpp=16        1  D3DFMT_X1R5G5B5
  dxt=31/32 fractional bpp  ~55        frame_count > 1 (animated; size is per-frame)
  ```

  `bcn_encoding_from_dxt` mapped `1 => Bc1`. **Tag 1 is not DXT1** — it is uncompressed
  32bpp: `ITEMS_EXPRESSIONS_CONTAINMENT_RIGHT_ON` is 64×64 with `top_mip_map_size == 16384
  == w·h·4` and a chain of 21,824 = 16384+4096+1024+256+64. Those 9 assets decoded as
  garbage. Tags `3`, `5`, `33`, `34` never occur in the data at all — they were invented.
- **A few assets pack sub-4×4 levels unclamped.** The 512×512 DXT5 sky textures store
  349,525 bytes where `CalculateTextureSize` computes 349,552 — a 27-byte deficit, exactly
  the last three levels stored as raw `w·h` (16+4+1) rather than block-clamped (16+16+16).
  So **walk the chain and stop when the bytes run out; never seek by a computed offset.**
- **`frame_count > 1`** (19 of 4,000) packs every frame; per-asset chain arithmetic does not
  apply. Nothing draws them yet.
- **`depth > 1` is a volume texture**, and `top_mip_map_size` then covers every slice. Found
  by the 7.1 test, which flagged `WEATHER_RAIN` (32×32, 8192 bytes where 32×32 DXT3 is 1024)
  and `MIST_ALPHA` (64×64, 262144) as disagreeing with their encoding — until depth was
  folded in, at which point `8 × 1024` and `64 × 4096` land exactly. Four assets, counting
  each one's `_PC` twin in a second bank. The 2D path declines them rather than mistaking
  slice 0's dimensions for the whole asset.

**Anisotropy is 4, and it is read, not chosen.** `CEngine::AnisotropicFilteringLevel`
defaults to **2** (`fableengine/engine.cpp:2686`) and is pushed into the per-stage
`D3DSAMP_MAXANISOTROPY` slot each frame (`engine.cpp:5724`). `NGlobalConsole::
ConsoleSetMaxAnisotropy` (`fablelib/global_console.cpp:1371`) writes its `ValSLONG` argument
into **the same global slot** — which pins the field as the D3D max-anisotropy *degree*, not
a filter-mode enum. Shipped retail `~/Fable/user.ini` opens with:

```
SetMaxAnisotropy(4);
```

`TEXTURE_FILTER_MODE { POINT=0, LINEAR=1, ANISOTROPIC=2 }` and `TEXTURE_MIPMAP_MODE {
POINT=0, LINEAR=1, DISABLE=2 }` (`_misc/e.hpp:763,770`) confirm anisotropic min/mag and
trilinear mip are both first-class engine states.

**MSAA is a divergence, and is labelled one.** `MULTI_SAMPLE_MODE { 1X, 2X, 4X }`
(`_misc/m.hpp:20`), `ESurfaceMultisampleType` = D3D9's `D3DMULTISAMPLE_TYPE`
(`_core/L4.hpp:1494`), and `CDisplayManager::{EnumerateAntiAliasingModes,
IsAntiAliasingModeValid, SetDisplayMode(dims, depth, multisample, …)}`
(`lib_display_manager.hpp:1156-1159`) make it a device-level setting enumerated against
hardware; `ConsoleSetAntialiasing` (`global_console.cpp:1414`) re-creates the device. In the
shipped configuration it is **off** — `~/Fable/dbugst.ini:90-91`:

```
//SetAntialiasing(TRUE);
//SetAntialiasing9x(TRUE);
```

So enabling it is an improvement, not a transcription: §6.3 `ACCEPTED`, with the reason
written down — not a `// UNVERIFIED:` constant, and not a default smuggled in silently.

**What none of this fixes.** The landscape runs the foreground triplanar pass at every
distance (step 5.6, deferred). The original does not sample these textures far away at all — it
draws per-patch `RenderProceduralTexture` composites. Mipmaps take distant terrain from
*aliased* to *correct but over-blurred and over-drawn*; the background LOD is its own step.

### 3.13 Local detail — a level's foliage, generated

**A ground theme names a generator, and every level already has everything needed.**
`CEngineThemeDef::LocalDetailGeneratorDef` (`fablelib/defs/engine_theme_def.hpp:900`) points
at a `LOCAL_DETAIL_GENERATOR` def of layers, each of objects. Measured against retail
`game.bin`: **65 generators, 101 layers, 227 objects, 138 distinct meshes**; 79 of 463
`ENGINE_THEME` defs name a generator and **all 79 resolve by index** — the `.lev` palette's
stale index (§3.4) is not a trap here; and **all 227 `Mesh` fields resolve to mesh assets** in
`graphics.big`, exactly like `Graphic.BankIndex` (§3.11). No new parser was needed for any of
it, and `fable-defs` already modelled every field.

**Three primitive types, decided in `CLocalDetailObjectCollectionType`'s constructor**
(`engine_local_detail_theme.cpp:2650`), in this order — a `ZSpriteFadeEnd` wins over
`IsRepeatedMesh`:

| Type | Of 227 | The engine does | We do |
|---|---|---|---|
| `MESH` | 92 | `AddStaticMesh` — the *same call* a `.tng` thing makes (`engine_local_detail_primitives.cpp:484`) | the existing model pass, unchanged |
| `REPEATED_MESH` | 88 | `SHADERS_REPEATED_MESH`, 16 instances per draw | `LocalDetailPass`, transcribed |
| `HYBRID_MESH_ZSPRITE` | 47 | mesh near, generated billboard impostor far | the mesh half; impostor deferred (step 8.7) |

So half of local detail needs **no new pipeline at all**, and that is faithfulness rather than
a shortcut.

**Placement is deterministic, and that is the whole reason we generate rather than read the
shipped cache.** Everything random comes from one PRNG:

```
GFROR13(x) = rotate_right(x, 13)          bbblibrary/lib_global_tools.cpp:1133
seed       = GFROR13(seed * 0x24a1 + 0x24df)
```

Ghidra spells the additive constant three ways (`0x24da+5`, `0x24dc+3`, `&DAT_000024df`) —
one number. Two consumers, both starting from a zeroed seed:

- **`CDisplacementTable`** (`engine_local_detail_generator.cpp:84`) — 32³ floats,
  `fmod((float)(u32)seed, 65536) / 65536`, filled with the *middle* axis outermost (the loop
  nest's strides are 0x80 outer, 0x1000 middle, 4 inner). `GetRandomDisplacement(x, y, z)`
  (`:2190`) is a bare `Table[x&31][y&31][z&31]`, so **a cell has only 32 distinct random
  values** however much it grows — which is why Fable's grass clumps.
- **`BuildElementGrid`** (`engine_local_detail_theme.cpp:1357`) — dart throwing on a
  **toroidal 32×32 cell tile**, stopping after 256 consecutive rejections, with each layer
  keeping `SpacingFromLayer[j]` from layer *j*'s points. Densities are what set a level's
  size: spacing 0.12 → 37,241 points per tile (36/cell), 6.5 → 15.

**The pattern therefore repeats exactly every 32 world cells**, by construction. Faithful, and
not a bug to fix.

**Per placement point** (`AddObjectsFromBlendedThemes` :3956, `AddObjectsFromLayerElement`
:3256): one draw picks which of the cell's three blended ground themes owns the point, then
four more pick the object, its scale, whether it survives the slope fade, and its rotation.
Notable readings:

- `Probability` is a **relative weight within the layer**, never a chance of nothing —
  `BuildObjectSelectionTable` normalises by the sum, which is what the 11 shipped layers
  summing to 0.30…1.10 depend on. Its advance fires on `cumulative * 32 < slot`, giving every
  layer a half-slot bias toward its first object.
- `ThemeBlendThreshold` is **0.00 on all 227 shipped objects**. The comparison is transcribed;
  it can never reject anything but a zero-weight theme.
- The slope fade reads the ground normal's **Z**, settled by the `C3DVector` written over
  `CMatrix3x4`'s tail. Grass at 0.80…0.90 thins out and then stops as the ground steepens.
- Scale is `(Scale + (2·rand − 1)·ScaleRandomElement) × 0.01` — **the same 100× mesh-unit
  constant as §3.11**, sourced independently, and with no `RenderSizeX`.
- Height and normal are bilinear over `PeekLandscapeHeight` and `PeekMapNormal`
  (`PeekInterpolatedMapNormal`, `engine_world_map.cpp:1688`), so objects land on exactly the
  surface the landscape pass draws.

**The repeated-mesh pass has its own register layout** (`engine_vs_layout_repeated_mesh.cpp:60-91`),
which cross-checks against `VSHADER_REPEATED_MESH` on every register it touches:

| Range | Name | |
|---|---|---|
| `c19`–`c34` | `ObjectMatricies` | `.Offset = 0x13, .Count = 0x10` |
| `c35`–`c50` | `ObjectOffsets` | `.Offset = 0x23, .Count = 0x10` |
| `c51`–`c66` | `LightingResults` | `.Offset = 0x33, .Count = 0x10` |
| `c67`–`c82` | `MainLightLightingResults` | `.Offset = 0x43, .Count = 0x10` |
| `c83`–`c95` | `User` | `.Offset = 0x53, .Count = 0xd` |

`BuildFromSourceMeshes` (`engine_local_detail_primitives.cpp:2799`) fills them as
`ObjectMatricies[i] = (cos·Scale, sin·Scale, 0, 0)` and
`ObjectOffsets[i] = (E41, E42, E43, Scale)`. Two things follow directly:

> **`TiltToSlope` has no effect on a repeated mesh.** The tilt basis
> `AddObjectsFromLayerElement` composes into the placement matrix cannot survive into four
> floats, however many defs set the flag — and grass, bracken and dandelions all set it.

> **The two zeroed components are the wind skew**, which `SetupWindAnimation` writes later.
> Both `v0.z` terms read `r0.z`, so wind displaces x and y together rather than along a
> per-object direction.

**Repeated meshes are lit once per object from the ground normal**, which is why the
constructor forces `LandscapeNormalLighting` on for them. `CalcSWLightingNoClip`
(`fableengine/engine_lighting.cpp:2600`) computes
`Ambient + saturate(−L·n)²·Diffuse + max(L·n, 0)·Backlight` — **`VSHADER_STATIC_DIRLIGHT`'s
expression over the same four constants**, not merely a similar one. So the landscape, the
static meshes and the foliage all still light from one set of environment LUT rows, and step 2
remains one change.

**The fade is a screen-space stipple under an alpha test**, not blending —
`VSHADER_REPEATED_MESH_STIPPLE_ALPHA` computes `distance · c84.x + c84.w` into `oD0.w` and
`PSHADER_REPEATED_MESH_STIPPLE_ALPHA` adds it to a screen-space dither sample and `cnd`s the
texel's alpha away. **The dither pattern is procedural**: `CEngineResourceManager::
BuildAlphaStippleTexture` (`engine_resource_manager.cpp:2900`) builds a **32×32** ordered
dither at startup — shuffling 1024 indices with the same `GFROR13` chain — and
`SetupAlphaStippleTexture` (`:3463`) splats it into all four channels. There is no asset to
find. This is also the original's own answer to the foliage silhouettes §5 step 7.5 deferred
`alpha_to_coverage` for.

**Scale, measured.** Local detail is ~100× the placement count of `.tng` things, not
~10,000×: Witchwood 691, Darkwood 1,147, LookoutPoint 15,845, Arena 0. A whole map's objects
fit in one instance buffer per mesh and generate in well under a second, which is why
`CLocalDetailCacheMap`'s ~9,000 lines of quadtree, cache groups and file blocks are not
ported — they page a thirty-map world through an Xbox's memory, and we load one map.

**The one trap, and it bit.** The engine positions objects in **world** cells because its
terrain carries the same `MapX`/`MapY` offset; ours does not — the landscape pass draws a map
at the world origin. So the origin must reach the random draws (it decides *which* foliage
grows) and nothing else, or a level's foliage stands thousands of cells off its own hillside.
Every shipped origin is a multiple of 32 and the draws mask to five bits, so the two readings
are indistinguishable by eye and only a test tells them apart.
---

## 4. *(retired)*

The 2026-08-05 audit that drove step 0 (§10). Every row of it has been actioned. The pattern it
named is worth keeping in mind, because it recurred twice more: **the geometry is usually roughly right, and the data lookup is invented.**
That is where the effort goes, and §11.1 is the structural answer to it.

---

## 5. Subsystems — what is built, and what is deliberately not

Each step: derivation posted for review → implementation → verification artefact → commit
citing the oracle (§7).

**"Deliberately not done" lists are load-bearing.** They are the difference between a decision
and a drift, and they are the first place to look before starting anything — the answer to
"why doesn't X work" is usually a numbered item here with a reason attached.

- **Step 0 — strip back.** Done. Deleted every invented mechanism, dropped `add_srgb_suffix()`
  (§3.5), converted to native Z-up (§3.6).
- **Step 1 — the mirror.** **Abandoned 2026-08-08.** A comparison testbed driving the original
  engine headlessly needed far more setup than it returned; `packages/mirror` and `scenes.toml`
  are deleted. What was learned about the game itself is §3.9. Verification went back to §2's own
  rules, which never depended on it. What survived: the lib/bin split became §11's crate split,
  and `Renderer::new_headless` + `render_to_image` became `--screenshot`.

### Step 2 — The environment layer — **HALF BUILT**, and it is the next faithfulness work

A faithful port of `CEnvironmentThemeSetDay` / `CBlendedEnvironmentTheme` into
`fable-data/src/environment.rs`. Derivation is §3.1 and is complete; nothing blocks this but
time.

**Half of it already exists.** `fable_data::environment` reads `ENVIRONMENT_THEME_DAY`
keyframes and brackets them by time (`keyframes_at_time`, `sky_textures_at_time`), and
`Files` loads `lighting_colours.tga` into memory. **What is missing is everything that touches
the LUT** — `ColourLookupColumn` is read by nothing, and the loaded TGA has no consumer. Items
2.1–2.4 below are the gap; 2.5 largely exists as `keyframes_at_time`.

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

*Why it gates three subsystems:* the sky's gradient colours, the landscape's lighting and the
static meshes' lighting are the same four LUT rows (1/0/3 plus the light direction), and all
three run the same neutral placeholder today (§9). It lands as one change **only if
`FrameUniforms` is one type first** — see §12.7.

### Step 3 — Outer sky, faithfully — **PARTIAL**

**Nothing here is faked.** The invented time→column LUT walk and its `* 0.4` alpha fudge were
deleted in step 0 rather than tuned (§2 rule 5), so the gradients are passed as **zero** and
the sky shows its raw texture — the unimplemented half is visible rather than plausible.
`scene::sky_textures_at_time` resolves the real texture pair and the real keyframe blend from
`ENVIRONMENT_THEME1`. What is left is the LUT fetch behind `c92`/`c93`, which is step 2's job.

*One thing to fix while here:* the theme name `"ENVIRONMENT_THEME1"` is hardcoded in
`main.rs:427`. Which theme a level uses is data.

- 3.1 Rebuild the mesh to match `BuildOuterSkyMesh` exactly (interleaved pairs, U step 1/35,
  Z-up). Assert vertex/index counts against the decomp's loop bounds in a test. Our apex UV is
  `(0.5, 0.0)` where the original is `(-DELTA_NOTIONAL_ZERO, -DELTA_NOTIONAL_ZERO)` —
  effectively `(0, 0)` under clamp addressing (`engine_sky_renderer.cpp:581`). The cap renders
  as a flat disc either way (V is 0 across it in both), but the sampled column differs.
- 3.2 Rewrite `outer_sky.wgsl` as a literal transcription of `VSHADER_OUTER_SKY` +
  `PSHADER_OUTER_SKY`, with each WGSL line commented with the asm it came from (§6.7).
- 3.3 Feed `c92`/`c93` from step 2's `BlendedTheme`.
- 3.4 ~~Feed the pixel-shader `c0.w` from the real texture blend factor~~ — **done**, from
  `EnvironmentTheme::sky_textures_at_time`.
- 3.5 ~~Resolve and upload `SkyTexture0` / `SkyTexture1` per keyframe~~ — **done**, re-uploaded
  only when the active pair changes.
- 3.6 Land the first golden images: sky at 00:00, 06:00, 12:00, 18:00.

### Step 4 — Base band, sun, moon, stars, clouds — **NOT STARTED**

- 4.1 Base band: own pipeline, `mov oD0, c92`. It draws through the *outer sky* pipeline
  today, which samples sky textures — the wrong shader (§3.3).
- 4.2 Sun/moon: port `RenderSun` (`engine_sky_renderer.cpp:1617`) and `RenderMoon` (`:1812`);
  read `CSkyDef` for the real orbit/size/texture fields.
- 4.3 Star field: `BuildStarFieldVB` (`:3559`) + `RenderStarField` (`:3824`).
- 4.4 Clouds: `BuildCloudMesh` (`:434`) + `RenderClouds` (`:2636`) — largest remaining sky
  piece; defer until 4.1–4.3 land.

### Step 5 — Landscape — **BUILT** (foreground only)

Derivation §3.4. `fable-data::landscape` ports `CEngineLandscapeMeshBuilder`; `TerrainData` is
one draw per `(texture, mapping direction)`; `terrain.wgsl` transcribes `VSHADER`/
`PSHADER_LANDSCAPE_FOREGROUND` line by line.

*Evidence:* LookoutPoint resolves 38 of 38 palette slots (previously 0) into 26 layer passes
over 48,955 vertices with no placeholder textures.

**Deliberately not done:**

- 5.6 Background LOD — the quadtree, tesselation, edge strips and per-patch procedural
  textures. We run foreground everywhere; the original has `EnableLandscapeTesselation` and
  `EnableLandscapeLODUpdate` as their own toggles, so this is a supported configuration.
- 5.7 Bump mapping (`PSHADER_LANDSCAPE_FOREGROUND_BUMP`) and every shadowed/spot variant.
- 5.8 Water — `CWaterPatchDescriptors`, and the theme's `WaterHeight`/`WaterType`.
- 5.9 Local detail — became its own subsystem, step 8.
- 5.10 ~~Loading neighbouring maps~~ — **done**, with step 6.12. A region's `ContainsMap` and
  `SeesMap` entries load together and stitch across their shared boundary.

**Lighting is neutral** pending step 2, running the real
`Ambient + saturate(n·l)²·Diffuse + max(−n·l,0)·Backlight` with placeholder constants.

### Step 6 — Things — **BUILT** (static meshes only)

Derivation §3.11. `.tng` placements → def `Graphic` → `graphics.big` asset id, placed with a
ported `CalcObjectMatrix`, drawn instanced through a transcribed `VSHADER_STATIC_DIRLIGHT` +
`PSHADER_TEXTURE_DIFFUSE` over the same lighting constants the landscape reads.

*Evidence:* Witchwood 38/38, LookoutPoint 192/192 over 44 meshes, Arena 57/57 over 4 — no mesh
failures, and `placed + skipped == every thing in the file` as a test invariant.

**Deliberately not done:**

- 6.7 Animated/skinned meshes (`ENGINE_GRAPHIC_ANIMATING_MESH`, `SHADERS_PALSKIN`). 9 of 288
  things in LookoutPoint but **91 of 355 in Arena** — counted and logged per graphic type
  rather than approximated in bind pose.
- 6.8 Sprites, 3D sprites and generated effects — their own primitive managers.
- 6.9 Dropping index-degenerate triangles at decode. Half of every static mesh's triangles are
  strip stitches with two equal indices (474,048 of 998,466 measured) and the
  `degenerate_triangles` block flag is clear everywhere, so `expand_block` keeps them all. They
  rasterise nothing — pure waste, safe to drop, not yet done. **Cheap and worth doing; see
  §12.10.**
- 6.10 Frustum culling and `RenderFadeDistance` (`CEngineFadeDistance`,
  `UpdateStaticMeshAlpha`). The per-instance `colour` that carries the fade is already in the
  vertex layout and is opaque white; expect the fade constants to be `UNVERIFIED`-shaped, as
  `SetupRenderModeShadersAndConstants` is the same render-state-cache mangle that defeated the
  landscape blend modes.
- 6.11 Local lights: the 2/4/5-point-light shader variants and the 113 `CTCPhysicsLight`
  things. The register layout is known exactly (§3.8); `c21`–`c34` are simply zero.
- 6.12 ~~`.wld`-driven world placement~~ — **done**, with 5.10. `MapX`/`MapY` position each map
  in world cells, and a boundary vertex reads the neighbour's height, normal and theme rather
  than clamping to its own edge.
- 6.13 Decals, shadows, outline/glow and every `_ENV_`/`_BUMP_` variant.

### Step 7 — Texture sampling — **BUILT**

Derivation §3.12. The format tag mapping, the shipped mip chain, trilinear + anisotropy 4, and
MSAA 4× as an `ACCEPTED` divergence (§6.3).

*Evidence:* 6,242 of the 6,245 2D block-compressed assets in `textures.big` yield a complete
chain — 42,265 levels, 6.8 per asset — with level 0 byte-identical to the old top-mip accessor.
On LookoutPoint, MSAA drops hard luminance steps (`|ΔL| > 60` between horizontal neighbours)
from **537 to 108** while the mean gradient is unchanged (4.353 → 4.251): the jaggies go, the
image is not blurred.

**Blend tables stay single-mip, deliberately.** They are CPU-built 128×128 R8 lookups indexed
by the packed vertex normal (`oT0 = (CliffU, CliffV)`, §3.4), not by a surface parameterisation.
§3.4's additive compositing is correct *only* because the five direction blends partition unity
at every texel; mip-filtering that table would break the partition and reintroduce exactly the
seam leakage that pinned the blend mode in the first place. **This is why §12 keeps two named
samplers rather than one** (§12.4).

**Deliberately not done:**

- 7.5 `alpha_to_coverage_enabled`. Cutout materials `discard` at `ALPHA_CUTOFF = 0.5`, which
  MSAA does not smooth, so foliage silhouettes still crawl. A *second* divergence; the
  original's own answer to this is 8.5's stipple fade, which is a transcription rather than a
  divergence — prefer that.
- 7.6 The `frame_count > 1` animated textures and the 9 non-BCn assets. Nothing draws them.
- 7.7 `SetMaxTextureSize` / `ReduceMipmapLevel` as a quality knob. Understood (§3.12), a
  two-line skip; no reason to want it yet.

*The one structural choice worth knowing:* the MSAA resolve is its own pass, not a
`resolve_target` on the last drawing pass. Resolving in every pass would resolve three times
for nothing, and resolving in *one* of them would make that pass silently load-bearing —
reorder the passes and the frame goes blank.

*Still the first thing to suspect if terrain seams ever appear:* the landscape draws coplanar
layer passes over a blackout pass with `cull_mode: None`, so per-sample depth testing could
change edge behaviour where those layers meet. It did not at 4×, but the mechanism is there.

### Step 8 — Local detail — **BUILT**

Derivation §3.13. `fable-data::local_detail` ports `CEngineLocalDetailGenerator` and the
placement half of `CLocalDetailCacheMap`; static-mesh objects go through the existing model
pass because that is literally the call the engine makes for them; repeated meshes get a
transcribed `SHADERS_REPEATED_MESH` pass.

*Evidence:* Witchwood 691 objects (381 mesh / 90 hybrid / 220 repeated), Darkwood 1,147
(486/51/610), LookoutPoint 15,845 (114/68/15,663), Arena 0 — pinned exactly as tests, since
placement is a pure function of the defs, the heightmap and the PRNG. Every object stands on
the terrain to within a millimetre, two runs agree byte for byte, and the world origin is
proved to move the draws and not the objects.

**Next, and it is the visible gap:** 8.5 **the distance fade**. Grass draws to the horizon
today. `VSHADER_REPEATED_MESH_STIPPLE_ALPHA` + `PSHADER_REPEATED_MESH_STIPPLE_ALPHA` dither it
out in screen space under the alpha test, from `FadeStart`/`FadeEnd` (20–22 m for grass,
100–140 m for trees) through `ModifyFadeDistanceForVideoOptions`. The stipple pattern is
**procedural** (§3.13), so the only real work is porting `BuildAlphaStippleTexture`'s ordered
dither — and the render state it needs is not behind the render-state cache that defeated the
landscape blend modes, because it is all in the shader.

**Deliberately not done:**

- 8.6 The `.stb` local detail cache. Retail ships the finished placements inside
  `FinalAlbion_RT.stb` (a `BBBB` bank, §3.7). **Decided 2026-08-10 (Jamen): generate instead**,
  on §3.4's precedent and because the `.stb` has cost more than it returned before. It stays
  available as a tier-2 oracle — the engine's own object matrices to diff ours against — if
  exactness is ever in question.
- 8.7 ZSprite impostors (`SHADERS_ZSPRITE`, `CEngineBillboardGenerator`,
  `CEnginePrimitiveManagerRepeatedZSprites`). Consequence, stated: trees keep full geometry to
  their fade distance instead of collapsing to a billboard at ~55 m. Costs triangles, looks
  better, diverges from the original's distant silhouette.
- 8.8 Wind animation. 68 of 227 objects set `HasWindSkew`; the instance layout already carries
  the two components `SetupWindAnimation` would write.
- 8.9 Shadow meshes and `CastShadows` — 40 objects carry a distinct one, and nothing in the
  renderer casts a shadow yet.
- 8.10 The cache/quadtree/streaming machinery, ~9,000 lines. Justified by the measured scale
  (§3.13): a whole map's objects fit in one instance buffer per mesh and generate in well under
  a second. Revisit only if the resident set stops fitting.
- 8.11 Dynamic areas (`AreaChanged`, `UpdateDynamicArea`, `ConsoleAddLocalDetail*`) — the
  editor path.
- 8.12 Video-options fade scaling. `ModifyFadeDistanceForVideoOptions` is understood; factor
  1.0 and clamp 0.0 are the full-quality values, so it is a knob with nothing to turn.
- 8.13 Local lights on foliage — the twin of 6.11. `CalcSWLightingNoClip` goes on to add the
  113 `CTCPhysicsLight` things' contributions, and we stop before that loop.

---

## 6. Working practice

### 6.3 Classifying a difference

Every deliberate departure from the original gets a class and a written reason, so
"I improved it" and "it is broken" never look the same:

- `OK` — matches, within the tolerance for the thing being compared.
- `ACCEPTED` — differs, **with a written reason**, recorded in §9. MSAA is the standing
  example: the original ships antialiasing off (`~/Fable/dbugst.ini:90-91`, both
  `SetAntialiasing` lines commented out) and we turn it on because it looks better.
- `BUG` — differs, unexplained. **A new unexplained difference is the regression signal**, and
  must be explained or reverted before a step closes.

The original gate for this was a structural-similarity metric against captured reference
frames; that harness is gone (§5 step 1). The classification survives it and is the part that
mattered — the discipline is writing the reason down, not computing a number.

### 6.6 Profiling — still deferred

Our failures are accuracy, not frame time, and profiling an architecturally wrong renderer
measures the wrong thing. The trigger to revisit: when a frame's draw-call count or GPU time
becomes the thing blocking a subsystem. Then: `profiling` with `profile-with-tracy` (or
`tracing-tracy` to bridge existing spans), plus `wgpu-profiler` for GPU timestamp scopes — the
half that actually matters for a renderer.

§12 will move the draw-call count without being asked to; **do not claim it made anything
faster without measuring** (§12.9).

### 6.7 Shader transcription convention

Every WGSL file that transcribes an original shader carries the disassembly in its header and
cites it per line — see `renderer/sky/outer_sky.wgsl` for the established form. This makes a
shader reviewable by diffing against the asm rather than by reading WGSL and hoping.

A WGSL file that is *not* a transcription (there are none today; §13's text shader will be the
first) says so in its header, so the absence of an asm block is a statement rather than an
omission.

### 6.8 Logging

`tracing` + `tracing-subscriber` with `env-filter` are already in `openalbion`.

- **Per-frame logging is banned at `info`.** Dump values from a test or a one-shot
  `debug` line instead.
- Target-scoped filters: `RUST_LOG=openalbion::scene::terrain=debug`.
- `info` = lifecycle. `debug` = one-shot resolution decisions with values. `trace` = per-frame,
  off by default.
- Log **decisions with provenance**, not observations:
  `"sky gradient top ← col 4 row 13 (theme ENVIRONMENT_THEME1 kf 4)"`.

### 6.9 Tests

| Layer | What it tests | Example |
|---|---|---|
| Parser | Data reads back correctly | `lev`, `tga`, `def` round-trips |
| **Oracle-pinned** | A ported algorithm matches decomp literals | LUT column = `ColourLookupColumn + kf`; dome has 1 + 36*2 vertices |
| **Provenance** | Values reach the GPU from the right byte | `scene::*` output, snapshot-tested — no GPU needed (§11.1) |

The middle two are the ones that catch data-lookup bugs, which is where this project's bugs
live. An oracle-pinned test reads like:

```rust
// engine_sky_renderer.cpp:596 — radius literal 6.5e3, 36 segments (uVar13 < 0x24)
assert_eq!(mesh.vertices.len(), 1 + 36 * 2);
assert!((mesh.vertices[1].position.xy().length() - 6500.0).abs() < 0.01);
```

Tests that need a Fable install **skip** rather than fail when there is not one
(`fixtures() -> Option<_>`), so the suite runs anywhere.

---

## 7. Working together

The failure mode was a chain of small guesses with only the result visible. The counter is to
make the *derivation* the reviewable artefact, not the diff.

**Per step, in order:**

1. **Derivation note** — before any code. The question, the quoted asm / decomp / def lines,
   and the conclusion. Short, §3-style. Jamen confirms or corrects the reading.
   *This is the gate; nothing is implemented before it passes.*
2. **Implementation** — one mechanism, matching the approved derivation.
3. **Evidence** — the numbers, attached rather than described: a unit test pinning the ported
   value, or a `debug` dump of what `scene` produced. §11.1 is what makes this cheap.
4. **Commit** — message names the oracle:
   `sky: fetch gradient at ColourLookupColumn+keyframe (environment_theme.cpp:1862)`

For §12/§13, which have no oracle, step 1 is a *design* note and step 3 is a
pixel-identical capture or a measured count — same shape, different evidence.

**Escalate immediately, don't guess, when:** two oracles disagree; a decomp body is too mangled
to read confidently; a step needs an architectural choice the plan does not cover; or a value
cannot be sourced and would have to be invented.

**The tripwire:** if I am about to change a number to make the picture look better, stop.

**Cadence:** one step per working session, ending with a written summary of what landed, what
the evidence was, and what the next derivation note will cover. That summary goes in
§10, and anything it *establishes* is folded into §3, §5 or §9 in the present tense.

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

# wgpu/naga facts — read the vendored source, never assume the API
ls -d ~/.cargo/registry/src/*/wgpu-types-28.0.0 ~/.cargo/registry/src/*/naga-28.0.0

# what this adapter actually supports
nix shell "nixpkgs#vulkan-tools" --command vulkaninfo --summary

# one offscreen frame, for a pixel-identical before/after
cargo run -p openalbion -- --level LookoutPoint --screenshot out.ppm
```

---

## 9. Unverified constants

Track anything that could not be sourced. Empty is the goal.

| Location | Value | Status |
|---|---|---|
| `renderer/src/lighting.rs` | `LightingUniforms::NEUTRAL` — ambient/light dir/diffuse/backlight | placeholder — neutral by construction (`Ambient` 0.5 cancels the shaders' `mul_x2`), replaced wholesale by step 2's LUT rows 1/0/3. **One definition, shared by all three world passes (§12.7)**, so step 2 is one edit |
| `renderer/src/terrain.rs` | `fade_transform = (0,0,0,1)` | fade disabled through the real mechanism; `ForegroundFadeStart`/`End` arrive from a console command whose defaults the decomp does not show (`CLandscapeSettings`' ctor is inlined away) |
| `renderer/src/terrain.rs` | additive layer blend + blackout pass | **derived, not transcribed.** `SetupForegroundStates` goes through a render-state cache Ghidra reduces to offset arithmetic, so the `D3DRS_*` values are unreadable — but the blend mode is forced by the alphas summing to 1, and by the existence of `VSHADER_LANDSCAPE_FOREGROUND_BLACKOUT_PASS`. Confirmed on screen: alpha-over leaked sky between themes, additive-over-black does not |
| `fable-data/src/landscape/mesh.rs` | `DirectionMask::build` normal | **DIVERGENCE**, marked in place: `BuildMapDirMask` weights up to eight face normals; that arithmetic is too mangled to transcribe, so `PeekMapNormal` is used instead. Same surface, different smoothing |
| `renderer/src/model.rs` | `ALPHA_CUTOFF = 0.5` | unsourced. Note the local-detail pass reads a *real* per-object-type `AlphaRef` — this is the static-mesh path only |
| `renderer/src/model.wgsl` | world-space normal instead of object-space light | **DIVERGENCE**, marked in place: the original pre-transforms `c19` into each object's frame; we rotate the normal instead. Identical for the orthonormal matrices `CalcObjectMatrix` produces, and it keeps one light constant shared with the landscape |
| `renderer/src/texture.rs`, `terrain.rs` | `mipmap_filter: Linear` | **inferred, not read.** `TEXTURE_MIPMAP_LINEAR` exists (`_misc/e.hpp:770`) but which value the engine sets is behind the same render-state cache this table already records as unreadable. Linear is *forced* anyway — wgpu requires it when `anisotropy_clamp > 1`, and §3.12 sources the anisotropy at 4 |
| `renderer/src/local_detail.rs` | per-instance vertex buffer instead of 16 instances in vertex constants | **DIVERGENCE**, marked in the shader: `VSHADER_REPEATED_MESH` indexes `c19`/`c35`/`c51` with the address register because vs_1_1 has no instancing. The arithmetic is transcribed unchanged; the batch size of 16 has no observable effect to preserve |
| `renderer/src/local_detail.wgsl` | lighting evaluated in the vertex shader, not on the CPU | **DIVERGENCE**, marked in place: the original writes `CalcSWLightingNoClip`'s result into `LightingResults` per object. Both inputs are per instance either way, so the result is identical, and this keeps the lighting constants in one place for step 2 |
| `fable-data/src/local_detail/place.rs` | the draw counter's third index | **UNVERIFIED**: `GetRandomDisplacement`'s third argument is register-passed and invisible in both call sites. That a counter is incremented immediately before each draw is visible; that it starts at zero once per cell, and that the theme draw increments it too, is the reading. Plausibility-neutral — it changes *which* object stands where, not whether the result looks right |
| `fable-data/src/local_detail/grid.rs` | `cell = floor(x + 0.5)` | **derived, not read.** The `__ftol2_sse` arguments are FPU values, but the storage forces it: the encode is `floor((offset + 0.5) · 255)` and the decode is `byte / 255 − 0.5`, so the offset must land in `[−0.5, 0.5]`. Under `floor(x)` half of every grid would clamp to the cell's far edge and the ±16 wrap would never fire |
| `renderer/src/lib.rs` | MSAA 4× | **DIVERGENCE**, `ACCEPTED` (§6.3): the original ships AA **off**. Enabled deliberately as an improvement, per Jamen 2026-08-10; to be made configurable later |

Retired from this table, all now sourced and cited in place: the `LightArray`/`LightGlobals`/
`LightAttenuations` register offsets (§3.8, cross-checked three ways); `model.wgsl`'s invented
`0.3/0.7` shade; the sky dome's `36` segments and its `7000`/`−500`/`6500` extents; the
landscape's `HEIGHT_SCALE = 2048.0` and `CELL_SIZE = 1.0`; and `texture_scale`, which was
`0.0625` invented and is `0.125` read out of `ego_r.exe`.

---

## 10. History

Newest first, one entry per working session: what landed, and the correction worth remembering.
Anything a session *established* is a present-tense fact in §3, §5 or §9 instead — this is the
record of how it got there, not a second copy of it.

- **2026-08-15** — **Guide restructured; bindless planned, reviewed and measured.** No renderer
  code. This document was cumulative (2,442 lines, mostly session narrative and abandoned
  tooling) and is now current-only, with §3.9 and this section carrying what remains of the
  history. **Landed: the flythrough camera is gone** — `scene::camera_path`, the `--flythrough*`
  flags, and the `App` state driving them; it served one sample video and had no other consumer.
  §12's first draft was reviewed against the vendored wgpu 28 source and corrected on five
  points: `max_binding_array_elements_per_shader_stage` **defaults to 0** and must be requested
  (the layout would have failed validation on day one); the sampler binding array was
  unnecessary and is dropped; the per-material bind group has a third entry (`MaterialUniforms`)
  the plan did not account for; `clear_models` would have leaked the registry, so lifetime has
  to be decided up front; and WGSL allows at most one `var<immediate>` per module.
  `MAX_BINDLESS_TEXTURES` was **measured rather than guessed** (§12.3) — a level peaks at ~100
  textures, the largest region at 359 meshes, the whole 402-level world at 1,331 — and the same
  measurement quantified the missing texture cache at 2.5–3.9× redundant work per level.
  **Found on the way: the workspace does not link** — the devshell exports a bare mingw `CC`, so
  `packages/lzo` builds a Windows COFF object for a Linux host link (§0).
- **2026-08-10** — **Levels have foliage** (§3.13, step 8). Ported `CEngineLocalDetailGenerator`
  and the placement half of `CLocalDetailCacheMap`; repeated meshes got a transcribed
  `SHADERS_REPEATED_MESH` pass. No new parser was needed — retail `game.bin` already carries 65
  generators over 227 objects and 138 meshes. The whole subsystem is deterministic off one PRNG,
  so Jamen's call was to **generate rather than read the shipped `.stb` cache**, with counts
  pinned as tests. **Two corrections from implementing it:** objects come out in map-local cells
  while the random draws stay in *world* cells (the foliage stood 3,232 cells off LookoutPoint's
  hillside until that was split), and `TiltToSlope` cannot affect a repeated mesh because only
  `(cos·Scale, sin·Scale, 0, 0)` survives into its object matrix. Landed on the way: `game.bin`
  parsed once, and `FinalAlbion.wld` loaded for the world origin.
- **2026-08-10** — **Texture sampling** (§3.12, step 7). The renderer had never uploaded a mip
  level; the chains were in the archive all along, and the accessor's name — `get_top_mip_*` —
  made the loss look intentional. Anisotropy sourced at 4 from the shipped `user.ini`; MSAA
  recorded as an `ACCEPTED` divergence. Also caught: `dxt_compression == 1` is uncompressed
  32bpp, not DXT1, so 9 assets were decoding as noise, and four aliases in
  `bcn_encoding_from_dxt` were invented — no asset uses `3`, `5`, `33` or `34`.
- **2026-08-10** — **Levels are populated** (§3.11, step 6). `.tng` things resolve through their
  def's `Graphic` to a `graphics.big` asset id — 38/38, 192/192 and 57/57 across three levels —
  so the text `objects.def` bridge was deleted. **Four latent bugs found on the way, three of
  them in code that predated the work:** the landscape was mirrored in Y; Fable's meshes are
  clockwise-front, so `FrontFace::Ccw` was drawing their interiors (385,787 triangles vs 1,082,
  measured); the landscape pass was double-subtracting the camera position; and the model
  sampler clamped where D3D9 wraps. All three of the old ones were invisible while the landscape
  was the only thing drawn in world space — **placements are the independent witness that
  exposed them.**
- **2026-08-09** — **The landscape is textured** (§3.4, step 5). §3.4's earlier reading was wrong
  in one decisive way — `CliffU`/`CliffV` are the packed vertex *normal*, not texture
  coordinates — and correcting it made the subsystem simpler. Recovered the five `.rdata` tables
  from `ego_r.exe` via `Ego_r.pdb`, which pinned the texture scale at one tile per 8 cells. The
  bug that had kept the ground white: the `.lev` theme palette's stored def index is stale in
  retail data and the engine resolves it by *name*.
- **2026-08-08** — **The renderer became its own crate, and the mirror was abandoned** (§11).
  Passes take `TerrainData`, `Model` and `TextureImage` instead of reaching into `Files`, `Lev`
  and the archives, so a pass can no longer invent a data lookup. Verified as a visual no-op —
  three scenes rendered byte-identically before and after — using `mirror` immediately before
  deleting it. That check is still the pattern §12.8 uses.
- **2026-08-06** — Step 1.1–1.4. The lib/bin split, offscreen rendering, and the comparison
  harness that was later abandoned. Biggest find, and it outlived the harness: **FOV is data and
  it is horizontal** (§3.10) — `CAMERA_MODE.FOV`, default 70, giving ~43° vertical at 16:9. We
  had been feeding 70 in as a *vertical* FOV. It was never a constant to fit; it was one to read.
- **2026-08-05** — Renderer review. Identified the LUT column model (§3.1), the landscape layer
  architecture (§3.4) and the sRGB mismatch (§3.5) as the three root causes behind a long
  `fix:`/`diagnostic:`/`debug:` loop. Decoded the `.bbb` bank format (§3.7) and recovered the
  named shader constant register map (§3.8), cross-verified against the disassembly on six
  registers. Z-up settled. **§2's ground rules date from here** and are the response to that
  loop.

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

**One game binary.** `openalbion` is the project. World editing, when it happens, is a **mode
inside it** rather than a second binary — decided 2026-08-08 after briefly trying the split,
which only bought duplicated `files`/`scene`/`camera` for no gain the engine could use. The
renderer stayed a crate on its own merits (below); nothing else needed to.

### 11.1 The renderer's input boundary — *why this shape*

`renderer` depends on **wgpu, glam, bytemuck, derive_more, tracing** and nothing else. It
cannot open a file, does not know what a `.big` is, and has never seen a `Lev`. Every input
arrives as a plain struct:

| Type | Is |
|---|---|
| `TextureImage` + `ImageFormat` | decoded pixels or BCN blocks, ready to upload |
| `TerrainData` | vertices, indices, one image per theme layer, palette→layer map |
| `Model` / `ModelPrimitive` / `ModelMaterial` / `AlphaMode` | geometry + materials |
| `ModelInstance` / `LocalDetailInstance` | placements |
| uniform setters (`update_sky_uniforms`, …) | the shader constants, by name |

**This is the structural answer to §2.** The fix loop happened because passes were doing their
own data lookups and getting them wrong. A pass handed a finished `TerrainData` has nothing
left to invent, so ground rule 3 is enforced by the compiler rather than by a reviewer noticing.

It also buys §6.9's test layers: mesh builders and value lookups are ordinary unit tests
needing **no GPU and no Fable install**, and the renderer can be exercised headlessly
(`Renderer::new_headless` + `render_to_image`) against hand-written constants.

**Rule: nothing in `packages/renderer` may depend on `fable-data` or touch the filesystem.**
If a pass needs a number, it arrives as a parameter. That is the whole invariant.

**§13 will test this rule.** `fontdue` rasterizes a glyph from a font file — that is asset
conversion, the same category as `scene`'s job, so `fontdue` does not belong in `renderer`.
The text *pass* does. See §13.5.

### 11.2 Why a crate and not a module

- **It is its own compilation unit.** Renderer edits stop rebuilding through `fable-data`,
  which pulls `fable-defs` from git.
- **It is enforceable.** A module can quietly `use crate::files`; a crate cannot, because the
  dependency is not in its manifest. §11.1's invariant is checked, not remembered.
- **It is self-contained.** The wgpu surface, the passes and their inputs are one thing with
  one boundary, and can be read without the engine around them.

`openalbion::scene` stays a module of the binary. It is the conversion layer — Fable's assets
in, renderer inputs out — and per §6.8 it is where provenance logging belongs: *"which byte
became which constant"* is a conversion question, not a shader one. It has exactly one
consumer, so it has no reason to be a crate.

---

## 12. Bindless rendering — the architecture change

Planned 2026-08-15, not implemented. §2 rules 1 and 3 do not apply (there is no oracle);
rules 2, 4, 5 and 6 do, and rule 4 here means **pixel-identical**, not *looks right*.

Everything about wgpu below was read out of the vendored crate source —
`~/.cargo/registry/src/index.crates.io-*/{wgpu-28.0.0,wgpu-types-28.0.0,wgpu-hal-28.0.1,naga-28.0.0}`
— in the same spirit as rule 1, applied to library capabilities instead of decomp facts. Cited
by file and line so the next reader can check rather than trust.

### 12.1 Why now, not later

Every pass owns its texture binding independently, and all four do the same thing: build a
`BindGroupLayout` for "one texture + one sampler (+ a uniform)", then create a fresh
`BindGroup` per texture drawn. `ModelPass::build_materials` makes one per material
(`model.rs:524`); `TerrainPass` one per `(ground texture, blend table)` layer pass
(`terrain.rs:484`); `LocalDetailPass` one per repeated-mesh batch (`local_detail.rs:326`);
`OuterSkyPass` one for its two blended textures (`sky.rs:418`). None of this is wrong — it is
the ordinary wgpu pattern — but it does not scale to what is coming.

Font rendering (§13) wants one bindless-registered texture per rasterized glyph, with no atlas.
A fifth copy of the per-pass-bind-group pattern would multiply it by however many glyphs are on
screen, rebuilding bind groups as new characters appear. Build the bindless path once,
generally, and have all four existing passes *and* the future text pass draw through it.

**There is a second, independent motivation, and it is measurable: there is no texture cache.**
`Material::base_texture_id` is a global asset id (§3.11), but `scene::build_model` calls
`decode_texture` on every material of every mesh with no memory of having seen an id before,
and `ModelPass::build_materials` uploads whatever it is handed. Measured across the shipped
data (method in §12.3):

| Level | Meshes | Material texture refs | Distinct textures | Redundancy |
|---|---|---|---|---|
| Witchwood | 35 | 93 | 24 | **3.9×** |
| Darkwood | 43 | 100 | 29 | **3.4×** |
| LookoutPoint | 68 | 178 | 70 | **2.5×** |
| Arena | 4 | 16 | 13 | 1.2× |

So between a half and three quarters of every level's texture decode-and-upload work is
repeated. A registry keyed by asset id makes the second and later reference a cache hit for
free (§12.6). **`read_mesh_by_id` has the same problem one layer up** and no cache either:
`load_things` and `load_repeated_meshes` each call it, so a mesh used by both is read from
`graphics.big` and decoded twice.

### 12.2 What wgpu 28 actually offers

| Feature | Source | What it does | Platforms |
|---|---|---|---|
| `TEXTURE_BINDING_ARRAY` | `features.rs:715` (`1<<8`) | `binding_array<texture_2d<f32>, N>` in WGSL, indexed by a **dynamically uniform** value | DX12, Metal (MSL 2.0+, macOS 10.13+), Vulkan |
| `SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING` | `features.rs:774` (`1<<11`) | indexing by a value that varies **per fragment** within one draw | DX12, Metal 2.0+, Vulkan 1.2+ (or `VK_EXT_descriptor_indexing`) |
| `IMMEDIATES` | `features.rs:1537` | small per-draw data, `RenderPass::set_immediates(offset, data)` (`render_pass.rs:518`), declared `var<immediate>` in WGSL — wgpu's current name for push constants | DX12, Vulkan, Metal native; OpenGL emulated |
| `PARTIALLY_BOUND_BINDING_ARRAY` | `features.rs:793` (`1<<13`) | a bind group may supply fewer entries than the layout's `count` | Vulkan, DX12 — **not Metal**. *Not needed*: §12.4 always fills every slot |
| `BUFFER_BINDING_ARRAY`, `STORAGE_RESOURCE_BINDING_ARRAY` | `features.rs:736`, `:749` | arrays of buffers / storage resources | Not needed — the renderer only samples |

Six structural facts, each of which shapes the design:

1. **Binding arrays are *sized*, not unbounded.** `BindGroupLayoutEntry::count:
   Option<NonZeroU32>` is fixed at layout-creation time. The design needs a chosen capacity
   (§12.3), and outgrowing it means rebuilding the layout, which cascades into every pipeline.
2. **A `BindGroup` is immutable.** Nothing lets an existing slot point at a different texture.
   Registering a new texture rebuilds the *whole* bindless `BindGroup` — cheap (`N` view
   handles via `BindingResource::TextureViewArray`, `bind_group.rs:71`, not `N` copies of data)
   but not free, and not something to do per-texture in a loop (§13.3).
3. **⚠ The binding-array limits default to zero and must be raised.**
   `max_binding_array_elements_per_shader_stage` and
   `max_binding_array_sampler_elements_per_shader_stage` are **0** in `Limits::default()`,
   `Limits::downlevel_defaults()` and `downlevel_webgl2_defaults()` alike
   (`limits.rs:378-379`, and the doc-comment tables at `:320`, `:440`, `:517`). Their doc says:
   *"This 'defaults' to 0. However if binding arrays are supported, all devices can support
   500,000"* (1,000 for samplers). `DeviceDescriptor` derives `Default` (`device.rs:10`), and
   `required_limits`' own doc is explicit: *"Exactly the specified limits, and no better or
   worse, will be allowed in validation of API calls on the resulting device"* — validation uses
   what you **asked for**, never what the adapter can do. The renderer passes
   `..Default::default()` today (`lib.rs:186`, `:259`), so **the bind group layout would fail
   validation even though the feature was granted.** This is the single mistake most likely to
   stall day one. The right shape is to start from `adapter.limits()` and lower it to what is
   wanted, not to start from `Limits::default()` and raise it blind.
4. **`max_immediate_size` also defaults to 0** (`limits.rs:397`) and needs both the `IMMEDIATES`
   feature and a raised limit, or `set_immediates` panics. Expect 128–256 bytes on Vulkan, 256
   on DX12, 4096 on Metal (`limits.rs:222-230`) — this machine's RADV reports
   `maxPushConstantsSize = 256`. §12.5's struct is 16 bytes.
5. **A WGSL module may contain at most one `var<immediate>` global** (naga `ir/mod.rs:359-365`;
   the keyword is `immediate`, `front/wgsl/parse/conv.rs:21` — *not* `push_constant`). So each
   shader gets exactly one per-draw struct, not several composed.
6. **Every entry in one binding array must match the layout's declared sample type and view
   dimension.** Today every renderer texture is `Float { filterable: true }`, D2,
   non-multisampled — Bc1/Bc2/Bc3, Rgba8Unorm and R8Unorm all qualify (`image.rs:30-40`), and
   the glyph bitmaps of §13 are R8Unorm too. **This is an invariant to write down**: the first
   texture that is not (a depth buffer, a storage texture, a cube map) needs its own binding,
   not the shared array. Differing mip counts and differing dimensions between entries are fine.

**This machine supports all of it.** AMD Radeon RX 580 (Polaris 10), RADV, Mesa 26.2, Vulkan
1.4: `shaderSampledImageArrayNonUniformIndexing = true`, `runtimeDescriptorArray = true`,
`descriptorBindingPartiallyBound = true`, `maxPerStageDescriptorUpdateAfterBindSampledImages =
1,015,808`, `maxPushConstantsSize = 256`. So capacity is not hardware-bound here; it is bounded
by memory and by the least capable target we care about.

**Decision — request non-uniform indexing from day one.** Nothing in §12.8's migration needs
it: every existing pass draws one material/layer/batch per draw, so its texture index is
constant for the whole draw — *dynamically uniform*, covered by `TEXTURE_BINDING_ARRAY` alone.
Only §13's glyph batching needs it. But it is supported on exactly the same platforms, so
asking now costs nothing and avoids a second feature/limits migration later.

**Decision — bindless is a hard requirement, not a fallback path.** If the adapter lacks
`TEXTURE_BINDING_ARRAY`, `request_device` fails with a clear message and the game does not
start. Maintaining two texture-binding paths would reintroduce exactly the duplication §12.1
exists to remove, and the renderer *already* hard-requires `TEXTURE_COMPRESSION_BC`
(`lib.rs:185`), so this is the same kind of requirement, not a new kind. What it costs: GL and
WebGPU, neither of which this project targets. **State this in the error message**, so a user
on unsupported hardware gets a reason rather than a panic.

### 12.3 How big — `MAX_BINDLESS_TEXTURES`, measured

Measured against the shipped data on 2026-08-15, not chosen. Method: for each level, take every
mesh asset id its `.tng` things resolve to (§3.11) plus every mesh its local-detail generators
place (§3.13), decode each distinct mesh, and count distinct non-zero `Material::base_texture_id`
plus the palette themes' `BaseTexture`/`CliffBaseTexture`.

| Scope | Distinct meshes | Distinct mesh textures | Ground textures |
|---|---|---|---|
| Arena | 4 | 13 | 11 |
| Witchwood | 35 | 24 | 14 |
| Darkwood | 43 | 29 | 23 |
| LookoutPoint | 68 | 70 | 20 |
| Largest `.wld` region (`ExecutionTree`) | 359 | — | 50 |
| **Whole world**, all 402 `.lev` files | **1,331** | — | **132** |

So a single level's resident set is **≈ 100 textures** (70 mesh + 20 ground + 5 blend tables +
2 sky, at the LookoutPoint peak). A whole region is a few hundred. The **entire game world** is
1,331 distinct meshes, whose textures — at LookoutPoint's roughly 1:1 mesh-to-texture ratio —
will not exceed a few thousand.

> **Recommendation: `MAX_BINDLESS_TEXTURES = 4096`.**
> It covers the whole world with headroom, so the regrowth problem in fact 1 never has to be
> solved. It costs one `TextureView` handle per slot in a `Vec` and one descriptor per slot in
> the bind group — on the order of 128 KB of descriptors, against a hardware ceiling of
> 1,015,808 here and 1,000,000 on Metal argument-buffers Tier 2 (`wgpu-hal` `metal/adapter.rs:737`).
> The only real risk of a large capacity is Metal **Tier 1**, where the same file reports 128 or
> even 31 elements — so if macOS ever matters, probe `adapter.limits()` and clamp, rather than
> hardcoding blind.

**Pin these counts as a test with the implementation.** They are a pure function of the shipped
data, exactly like §5 step 8's placement counts, so they belong in the suite rather than in this
paragraph.

### 12.4 Architecture — `BindlessTextures`

A new `renderer/src/bindless.rs`, owning:

- `slots: Vec<TextureView>`, length `MAX_BINDLESS_TEXTURES`, **pre-filled at construction with
  the fallback 1×1 white view** already built by `model.rs`'s `create_white_view`. Every slot is
  always valid, so the array is always fully bound — which is precisely why
  `PARTIALLY_BOUND_BINDING_ARRAY` (unsupported on Metal) is never needed.
- `index_of: HashMap<TextureKey, BindlessIndex>` plus a next-free counter. `TextureKey` is the
  global asset id for game textures (§3.11), a `(direction)` for the five blend tables, or a
  glyph key `(FontId, char, px_size)` for §13.
- `register(key, view) -> BindlessIndex` — returns the existing index on a repeat key (this
  *is* the dedup); otherwise takes the next free slot and marks the registry dirty. **Touches no
  GPU state.** Returns an error rather than panicking when full, so exhaustion is a diagnosable
  condition and not a crash.
- `rebuild_if_dirty(&Device) -> &BindGroup` — recreates the one bind group from `slots` via
  `TextureViewArray` only when dirty. Called at controlled sync points, **never** per
  `register`.
- **Two named samplers, plain bindings — deliberately *not* a sampler array.** A
  `binding_array<sampler, N>` with a per-draw `sampler_index` is the symmetric-looking answer
  and is unnecessary: sampler choice is *per pass and statically known in each shader*. Model and local detail use
  `repeat_sampler` alone; terrain uses `ground_sampler` (repeat/aniso) and `blend_sampler`
  (clamp, no mip — for the reason in §5 step 7); sky uses `linear_clamp_sampler`. Two ordinary
  sampler bindings alongside the texture array covers every case. This removes the
  `max_binding_array_sampler_elements_per_shader_stage` limit from the requirement list, removes
  `sampler_index` from the per-draw data, and sidesteps Metal's much lower sampler-array limits.

**Bind group index convention, fixed before §12.8 step 1**: **group 0 = the pass's own frame
uniforms; group 1 = the shared bindless group.** That is exactly where each pass's per-draw
group already sits (`model.rs:653`, `terrain.rs:620`, `local_detail.rs:456`, `sky.rs:484`), so a
migrated pass swaps a per-draw bind group for the shared one *at the same index* and no other
pass's pipeline layout changes. Set group 1 once per render pass, not per draw.

**Registry lifetime — decide this before §12.8 step 1, because it decides `BindlessIndex`'s
type.**
`Renderer::clear_models()` exists and runs on every scene load (`main.rs:462`); today it drops
the meshes and their textures with them. Once the registry owns the views, `clear_models` would
leak every slot and every texture unless it also clears the registry — at which point every
`BindlessIndex` previously handed out is stale. Two workable answers:

- **Level-scoped registry (recommended).** `clear_models` clears the registry too. Indices are
  valid only between one clear and the next, which is already true of every `GpuModel` that
  holds them. Document the invariant; the type stays a bare `u32`.
- **Generation-tagged indices.** `BindlessIndex { generation: u32, slot: u32 }`, validated on
  use. Safer, more machinery. Only worth it if indices ever outlive a scene load.

Either way, **say which**, and make `clear_models` do it — silently leaking the whole world's
textures across scene loads is the failure mode this section exists to prevent.

### 12.5 Per-draw data — and what the per-material bind group *actually* carries

The per-material bind group is **not** just a texture and a sampler. `ModelMaterialBindGroupLayout`
has three entries (`model.rs:215-242`), the third being a `MaterialUniforms { alpha_test: u32,
alpha_cutoff: f32 }` in its own 16-byte uniform buffer, **created per material**
(`model.rs:518`). `LocalDetailPass`'s per-batch group carries `alpha_cutoff` the same way
(`local_detail.rs:320`). Any plan that replaces only the texture leaves the layout standing and
"retire the scaffolding" cannot happen.

So the per-draw immediate carries all of it — one struct per shader (fact 5):

```wgsl
struct DrawConstants {
    texture_index: u32,   // slot in the bindless array
    alpha_test:    u32,   // non-zero enables the cutout discard
    alpha_cutoff:  f32,
    _pad:          u32,
}
var<immediate> draw: DrawConstants;
```

16 bytes, comfortably inside every backend's limit (fact 4). Terrain's variant carries
`ground_index` and `blend_index` instead of one index (§3.4 binds a *pair* per layer pass).
`RenderPass::set_immediates(0, bytemuck::bytes_of(&draw))` replaces each
`set_bind_group(1, &material.bind_group, &[])`, and every `PipelineLayoutDescriptor`'s
`immediate_size` goes from `0` (`model.rs:266`, `local_detail.rs:209`) to
`size_of::<DrawConstants>()`.

**Consequences worth stating:** one 16-byte uniform buffer *per material* disappears entirely,
and `MaterialUniforms` with it — for LookoutPoint that is 178 buffer allocations (§12.1) down
to zero.

**Documented fallback if `IMMEDIATES` is ever unavailable:** a uniform buffer with
`has_dynamic_offset: true`, one entry per draw, set via `set_bind_group`'s offset argument. A
value read from a uniform buffer is still dynamically uniform, so the binding array is happy,
and no feature is needed. Slower and more bookkeeping; recorded here so it is a known option
rather than a mid-migration discovery.

### 12.6 The dedup — where it goes, and what it is worth

Two independent caches, and the *upper* one is the bigger win:

1. **GPU-side, free:** `BindlessTextures::register` keyed by asset id turns a repeated
   `base_texture_id` into a cache hit. Saves the upload.
2. **Scene-side, the real win:** a `scene::TextureCache` from asset id → `BindlessIndex` (or to
   an already-decoded `TextureImage`) lets `build_model` skip the **archive read and the BC
   slice** entirely on a repeat, not just the re-upload. §12.1's table is the measurement: 2.5–3.9×
   redundant work today.
3. **Meshes too, and it needs no bindless at all:** `Files::read_mesh_by_id` re-reads and
   re-decodes on every call, and both `load_things` and `load_repeated_meshes` call it. A
   `HashMap<u32, Rc<Mesh>>` beside it is a small, independent change that can land before §12
   or after.

Note that the two caches are *not* redundant: (1) dedups within one renderer, (2) dedups the
CPU work before the renderer is ever called. Do (2).

### 12.7 Unify the lighting registers — **DONE**

`c3`/`c19`/`c20`/`c35` (§3.8) were declared in `model.rs`, `terrain.rs` and `local_detail.rs`,
each with its own copy of the same neutral placeholder — and the same four fields restated in
three `.wgsl` files. §5 step 2 says the environment layer "lights meshes and terrain in one
change", which was only true if there was one place to make it.

Now `renderer::lighting::LightingUniforms` + `lighting.wgsl`, the latter prepended to all three
shaders at module creation so the two language's layouts cannot drift. The four registers are
`vec4`s, so embedding the struct left every byte offset where it was.

*Evidence:* LookoutPoint's `--screenshot` frame is **byte-identical** before and after
(`md5 4137e8a4…`), and `cargo test --workspace` is green.

**This is the shape step 2 now edits: one constant, `LightingUniforms::NEUTRAL`.**

### 12.8 Migration order — one pass per commit

1. ~~**Land `BindlessTextures`, the features and the limits — unused.**~~ **DONE.**
   `renderer/src/bindless.rs`, plus one shared `request_device` replacing the block `new` and
   `new_headless` each had a copy of. Capacity is `MAX_BINDLESS_TEXTURES` (4096) clamped to
   `adapter.limits()`, refusing below `MIN_BINDLESS_TEXTURES` (256) rather than downgrading
   into a failure at some later draw.
   *All three verifications passed:*
   (a) LookoutPoint's `--screenshot` frame is **byte-identical** to before (`md5 4137e8a4…`);
   (b) the device **granted** what was asked — `Device: bindless 4096 textures, 16B immediates,
   non-uniform indexing available`, behind a hard `assert!` rather than a hope;
   (c) the array built and validated at its declared capacity — `Bindless textures: 4096 slots`
   — which only happens because `max_binding_array_elements_per_shader_stage` was raised.
   `renderer/tests/bindless_test.rs` keeps (b) and (c) as gates.
2. ~~**Model pass.**~~ **DONE.** `ModelMaterialBindGroupLayout`, its per-material `BindGroup`
   and `MaterialUniforms` are all gone; a 16-byte `DrawConstants` immediate carries the
   texture slot and the alpha-test pair per sub-mesh draw, and the shared bindless group binds
   **once per pass** instead of once per material. `scene::build_model` takes an `is_resident`
   predicate, so a repeat asset id skips the archive read and the BC slice as well as the
   upload (§12.6).
   *Verify:* LookoutPoint **and** Witchwood are byte-identical to before, and the registry
   lands where §12.3 predicted — LookoutPoint's region registers **97** slots against a
   predicted ~100. The dedup is measured, not assumed:
   **LookoutPoint 88 of 189 material references already resident (47%), Witchwood 93 of 144
   (65%)** — that much archive reading and BC slicing no longer happens at all.
3. ~~**Local detail.**~~ **DONE.** The same change, with `alpha_cutoff` in the immediate
   instead of a per-batch uniform buffer, and no alpha-test flag — this pass always
   alpha-tests, which is what makes tens of thousands of grass instances cheap. It now shares
   the *array* with the model pass as well as the types, so a mesh drawn by both — which is
   the normal case, since local detail's static-mesh half goes through the model pass by
   design (§3.13) — shares one upload.
   *Verify:* LookoutPoint and Witchwood byte-identical. Full-level dedup, both upload paths
   counted together: **LookoutPoint 88 of 204 references resident (112 slots), Witchwood 93 of
   146 (51 slots), Darkwood 32 of 62 (29 slots)**.
4. ~~**Terrain.**~~ **DONE.** Two indices per draw plus the mapping direction's planar
   projection, so its immediate is 48 bytes where the others are 16 — still a third of
   Vulkan's guaranteed 128. Its own `ground_sampler` and `blend_sampler` are gone: they were
   byte-for-byte the shared `repeat_sampler` and `clamp_sampler` (§12.4).
   **The ordering hazard was real, and the guard caught it.** Terrain registers inside
   `set_terrain`, which runs *before* the scene's models load, so the old `clear_models` —
   called after — wiped its indices. The landscape would have sampled whatever took its slots
   next, which looks almost right. `BindlessTextures` now carries a generation, `TerrainPass`
   records the one it registered under, and a `debug_assert` at draw time compares them; it
   fired on the first run. `clear_models` is now `clear_scene` and is called *before*
   `set_terrain`, because the registry is scene-scoped, not model-scoped.
   *Verify:* LookoutPoint and Witchwood byte-identical. LookoutPoint's region now holds 227
   slots, Witchwood 100.
5. **Sky — next.** Two textures a frame, the smallest surface; last because it has the least
   to gain. Note it re-uploads only when the active keyframe pair changes, so it registers on
   a different cadence from everything else — the one pass whose registrations happen mid-run
   rather than at scene load, and so the first real exercise of `rebuild_if_dirty` outside a
   load.
6. **Retire what is now dead:** `ModelMaterialBindGroupLayout`, `MaterialUniforms`,
   `TerrainBindGroupLayouts.draw`, local detail's inline material layout,
   `SkyTextureBindGroupLayout`. Each pass keeps only its frame uniform group plus the shared
   bindless group. **If any of these are still referenced, step 2–5 left something behind** —
   that is the check, not a cleanup chore.

### 12.9 Deferred

- **Merging draws.** Bindless makes it *possible* to draw many materials in one call; §12.8 does
  not do it, and none of the verification above depends on it. Draw-call count will fall anyway
  (one `set_immediates` beats one `set_bind_group`), but **do not claim a speedup without
  measuring** (§6.6).
- **Exercising non-uniform indexing.** Requested from day one, unexercised until §13. Being
  *granted* the feature does not prove the shader path works: give it its own small test — two
  quads in one instanced draw, each indexing a different slot — before font rendering leans on it.
- **Growing past the chosen capacity at runtime.** §12.3 picks a number that makes this
  unnecessary. If it ever is necessary, it means rebuilding the layout and every pipeline.
- **Bindless buffers or storage textures.** Nothing needs either.
- **Cross-frame slot eviction.** §12.3's measurement says a level, a region, and plausibly the
  whole world fit. Revisit only if the resident set becomes genuinely unbounded.

### 12.10 Worth doing alongside, cheaply

Small, independent, and each makes the migration or the next subsystem smaller:

- ~~Fix the devshell link failure~~ (§0) — **done**, `mingw.stdenv.cc` off `PATH`.
- ~~§12.7's lighting unification~~ — **done**.
- **The mesh cache** (§12.6 item 3). Independent of everything.
- **Step 6.9 — drop index-degenerate triangles at decode.** 474,048 of 998,466 measured
  triangles are strip stitches that rasterise nothing. Halving every index buffer is free, safe,
  and touches `fable-data::mesh` only.
- **Two broken provenance citations, both one line to fix.** `fable-data/tests/lexer_corpus.rs:1`
  cites a `§11.4` that does not exist. `fable-data/src/landscape/mod.rs:81,97,102` cites
  `tools/landscape-statics.md`, which was never committed and is gone (§3.4) — the RVAs in
  those same comments are what actually carries the provenance now, and the comments should say
  so rather than point at a missing file.
- **`Renderer::new` and `new_headless` duplicate their whole device-creation block**
  (`lib.rs:168-287`). §12 edits both identically; fold the shared part into one function while
  editing it, or edit it twice forever.

### 12.11 Open questions — resolve at the §7 derivation review, before code

Questions 1 and 2 were answered by implementing step 1, on the recommendations below; both are
cheap to revisit and neither has a caller yet.

1. ~~**Registry lifetime**~~ — **level-scoped**. `BindlessTextures::clear` drops every
   registration and returns the slots, and `BindlessIndex` stays a bare `u32` whose validity
   runs from one clear to the next — the same lifetime the `GpuModel`s holding them already
   have. `Renderer::clear_models` clears it, and
   `renderer/tests/bindless_test.rs` keeps that a gate — without it a scene load would leak
   every previous level's textures *and* burn their slots.
2. ~~**Capacity**~~ — **4096, clamped to `adapter.limits()`**, floor 256.
3. **Does the sky pass migrate at all?** It has two textures and one sampler and gains almost
   nothing. Migrating it buys uniformity — one texture path in the renderer, no exceptions —
   which is worth something on its own. *Recommend yes, last, for that reason alone.* Still
   open.

---

## 13. Font rendering & the dev console — decisions recorded, work deferred

Not started, and deliberately behind §12. This section exists so the decisions already made are
not re-litigated when the work starts.

### 13.1 Parsing and rasterization: `fontdue`

Decided 2026-08-15 (Jamen). Pulls a TTF/OTF and rasterizes individual glyphs to 8-bit alpha
coverage bitmaps at a requested pixel size, on demand. No shaping engine — no ligatures, no
complex-script layout — which is fine for a dev console and Latin UI text; revisit only if a
real HUD or dialogue system needs it.

The bitmaps are R8, which is already an `ImageFormat` the renderer accepts (`image.rs:23`) and
already satisfies §12.2 fact 6's sample-type invariant. No new upload path is needed.

### 13.2 No texture atlas — bindless per-glyph textures

Decided 2026-08-15 (Jamen), explicitly rejecting the conventional answer. An atlas — pack every
glyph into one shared texture, address by UV rect — is the standard approach and deliberately not
this project's: it reintroduces exactly the packing/repacking/overflow bookkeeping that bindless
removes, and by the time this starts §12's registry already exists to use instead. Each glyph
bitmap (well under 64×64) becomes its own texture in the same `BindlessTextures` array game
meshes and terrain use, keyed by `(FontId, char, px_size)`.

**The cost, quantified so it is a choice and not an oversight:** one `wgpu::Texture` and one
`TextureView` per glyph. A dev console's printable ASCII at one size is ~95 textures — against
§12.3's measured level peak of ~100 and a capacity of 4096, that is noise. It stops being noise
if a UI ever wants many sizes or a CJK range; at that point the atlas question is worth
reopening, and reopening it is not a reversal of this decision but a change in what the decision
was about.

### 13.3 Rasterize on demand, cache, rebuild in batches

Follows directly from §12.2 fact 2: every new registration dirties the whole array, so
rasterizing and registering one glyph at a time as text streams in would rebuild the bind group
once per character. Instead: collect every newly-needed glyph for the frame, rasterize and
register all of them, then call `rebuild_if_dirty` **once**. A `HashMap<(FontId, char, px_size),
BindlessIndex>` means steady-state typing after the first frame touches nothing.

### 13.4 Where non-uniform indexing actually gets used

A run of text wants **one instanced draw of many glyph quads, each instance answering a
different bindless index** — so adjacent fragments from neighbouring quads hit different array
slots within one draw. A per-instance vertex attribute is not dynamically uniform, so this is the
concrete, load-bearing use for
`SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING` (§12.2).

**But note what it buys: batching, not correctness.** One draw per glyph with a `DrawConstants`
immediate (§12.5) needs no non-uniform indexing at all and would work on day one. For a console
of a few hundred glyphs that is very likely fast enough. **Start there**, and move to the batched
instanced draw when there is a reason — the feature is already requested either way, so nothing
is blocked by choosing the simple path first.

### 13.5 The dev console: rudimentary and basic, on purpose

Scope, as stated by Jamen: a basic development console, not a reproduction of Fable's own
(`NGlobalConsole`) and not a general-purpose UI framework. **In scope:** a toggleable overlay, a
text input line, scrollback of recent output, and a way to wire in commands — the natural first
ones being `Enable*`-style subsystem toggles, in the spirit of (not copying) the original's own
console vocabulary and §6.8's logging targets. **Out of scope unless it becomes needed:** history
search, autocomplete, rich text/colour spans, resizing, theming.

A `Enable{Sky,Landscape,StaticMeshes,RepeatedMeshes}`-shaped toggle set is more than a
convenience: it is the subsystem isolation that made comparing against the original possible at
all (§3.9), and it is genuinely useful for this renderer's own debugging.

### 13.6 Open — resolve when the work starts

- **Where the module lives.** The text *pass* belongs in `renderer`, which by then owns bindless
  textures and pipelines. `fontdue` must **not** become a `renderer` dependency — rasterization
  is asset conversion, the same category as `scene`'s job, and §11.1's rule is the whole reason
  that boundary holds. So font and console state belong beside `scene`, or in a new
  `openalbion::console` module playing the same "real input in, plain renderer input out" role.
  The renderer receives `TextureImage`s and glyph quads, exactly as it receives `Model`s today.
- **Text geometry.** One quad per glyph, instanced, each instance carrying a bindless index —
  either `ModelInstance`'s pattern extended or a parallel `GlyphInstance`. Decide once §12.8's
  model-pass migration has a working answer to copy.
- **Screen-space projection and DPI handling.** Not investigated. The console draws after the
  resolve pass, into the presentable texture, so it is the one pass that is *not* multisampled —
  which is what you want for text anyway.
- **Shader transcription convention (§6.7).** `text.wgsl` will be the first WGSL in the project
  with no original to transcribe. Its header should say so explicitly.
