# Local detail — derivation note and plan

> §7 step 1 artefact for AGENTS.md §5 step 8 (was 5.9, "Local detail (grass, flowers)").
> Written 2026-08-10. Shaped to drop into AGENTS.md as **§3.13** (the derivation) and
> **§5 step 8** (the plan) once reviewed. Nothing here is implemented yet.

---

## 0. The three questions, answered up front

**1. Do we parse enough game data?** **Yes — all of it, today, with zero new parsers.**
Measured against retail `game.bin` + `graphics.big`:

| | |
|---|---|
| `LOCAL_DETAIL_GENERATOR` defs in retail `game.bin` | **65** (101 layers, 227 objects) |
| `ENGINE_THEME` defs | 463, of which **79 name a generator** |
| Those references resolving **by index** | **79 / 79** — no stale-index trap (unlike §3.4's palette) |
| Object `Mesh` ids resolving to a mesh asset in `graphics.big` | **227 / 227** |
| Distinct meshes across all generators | 138, all parse with `fable_data::mesh` |

`EngineLocalDetailGeneratorDef` / `…LayerDef` / `…ObjectDef` and `EngineThemeDef::
local_detail_generator_def` are already modelled in `fable-defs`. `LandscapeMap` already
exposes `theme_slot`, `theme_blends`, `height_at` and `vertex_normal` — which is the entire
input the generator reads. **The missing piece is an algorithm, not a format.**

**2. Separate instanced pipeline, or reuse the model pass?** **Both, and the split is the
engine's own.** `CLocalDetailObjectCollectionType`'s constructor
(`engine_local_detail_theme.cpp:2650`) sorts every object into one of three primitive types,
and each type goes to a *different* renderer:

| Type | Count | The engine does | We should |
|---|---|---|---|
| `MESH` | 92 / 227 | `CEnginePrimitiveRenderer::AddStaticMesh` — **byte-for-byte the same call a `.tng` thing makes** (`engine_local_detail_primitives.cpp:484`) | reuse `ModelPass` unchanged |
| `REPEATED_MESH` | 88 / 227 | `SHADERS_REPEATED_MESH` — 16-instances-per-draw constant instancing, flat per-instance colour, wind skew, stipple-alpha fade | its own pass, transcribed |
| `HYBRID_MESH_ZSPRITE` | 47 / 227 | mesh near + a generated billboard impostor far | draw the mesh half; defer the impostor |

So `LOCAL_DETAIL_PRIMITIVE_TYPE_MESH` needs **no new pipeline at all** — that is not a
shortcut, it is what the original does. Only `REPEATED_MESH` earns a second pipeline.

**3. How do we do this accurately?** The generator is fully deterministic and every
constant is sourced (§A). Its randomness is one 5-instruction PRNG — `GFROR13(x) = ror32(x,
13)` (`bbblibrary/lib_global_tools.cpp:1133`) driving `s ← ror32(s·0x24a1 + 0x24df, 13)` —
so exact reproduction is *possible*, not merely plausible. Three arguments are
register-passed and invisible in the decomp (§A.9); those are the accuracy boundary, and
§B.6 proposes the `.stb` cache as the numeric oracle that settles them.

---

## A. Verified ground truth

### A.1 The data path — theme to generator to object

`CEngineThemeDef::LocalDetailGeneratorDef` (`fablelib/defs/engine_theme_def.hpp:900`) is a
def index onto a `CEngineLocalDetailGeneratorDef`
(`fablelib/defs/engine_local_detail_def.hpp`), which is `Layers[]`, each
`{ SpacingFromLayer[], Objects[] }`, each object:

```
Mesh ShadowMesh ZSpriteMesh          long  — graphics.big ASSET IDS (measured 227/227)
Probability ThemeBlendThreshold      float
Scale ScaleRandomElement             float
FadeStart FadeEnd                    float
ZSpriteFadeStart ZSpriteFadeEnd      float
SlopeFadeStart SlopeFadeEnd          float
AlphaRef AlphaMipBias                long/float
WindSkew{Constant,Random,Speed}Factor float
CastShadows ReceiveShadows IsRepeatedMesh IsZSprite
AlphaIsBoolean HasLandscapeNormalLighting TiltToSlope HasWindSkew   : 1 each, one byte
```

The bitfield order is pinned by the constructor's own reads
(`engine_local_detail_theme.cpp:2650`): `AlphaIsBoolean = (b>>4)&1`,
`LandscapeNormalLighting = (b>>5)&1`, `HasWindSkew = b>>7`, `ReceivesShadows = (b>>1)&1`,
`b & 4` selects the repeated-mesh path, `b & 1` the shadow mesh — i.e. bit 0 `CastShadows`
… bit 7 `HasWindSkew`, exactly the declaration order. `fable-defs` already has it right.

**Primitive type, decided in that constructor and in this order:**

```
if (ZSpriteFadeEnd > 0)   -> HYBRID_MESH_ZSPRITE
else if (IsRepeatedMesh)  -> REPEATED_MESH   (and LandscapeNormalLighting is FORCED true)
else                      -> MESH
```

`AlphaRef < 0` falls back to `ENGINE.LocalDetailBooleanAlphaDefaultAlphaRef` when
`AlphaIsBoolean`, otherwise to the neighbouring `ENGINE` field; ≥ 255 clamps to 255.
`ZSpriteMesh == 0` means "use `Mesh`"; likewise `ShadowMesh`.

The text oracle for reading the authored data is
`~/doc/Fable_Anniversary-2013-02-25/Fable/Data/Defs/engine_local_detail.def` (plus
`Local_Detail_Backup.txt`), which is where the named fade-distance enums live
(`GRASS_FADE_START = 20`, `MEDIUM_TREE_FADE_START = 100`, `TREE_ALPHA_REF = 128`, …).

### A.2 The placement grid — dart throwing on a 32 × 32 torus

`CLocalDetailPlacementGrid` (`engine_local_detail_theme.hpp`) is **one grid per
(generator, layer)**: `ElementIndexGrid[32][32]` of `u16` prefix offsets into
`Elements: Vec<u16>`, where each element packs two bytes.

- `PeekElementCount(x, y)` (`:1683`) is `ElementIndexGrid[(x&31)+1][y&31] − ElementIndexGrid[x&31][y&31]`
  — a prefix-sum layout, so the whole grid is one flat array.
- `PeekElement(x, y, i)` (`:1650`) decodes `(lo/255 − 0.5, hi/255 − 0.5)` — an offset in
  **[−0.5, +0.5] around the cell**, so a point may spill into its neighbours.
- `MapPosToElementSpace` (`:786`) wraps differences at ±16 with a period of 32: **the grid
  is toroidal**, so the pattern tiles seamlessly and *the whole local-detail layout repeats
  exactly every 32 world cells*. That is the original's behaviour, not an artefact to fix.

`BuildElementGrid` (`:1357`) is classic dart throwing:

```
seed = 0
loop {
    x = fmod((float)(i32)(seed = ror32(seed*0x24a1 + 0x24df, 13)), 2097152.0) / 65536.0   // [0,32)
    y = same, next draw
    for tree in [own, base, base-of-base, …]:                       // one per layer, chained
        if !tree.ClipObject(x, y, LayerSpacing[layer_of_that_tree]) { reject }
    accept: quadtree.AddObject(x, y, LayerSpacing[LayerID]);
            element_grid[cell(x)][cell(y)].push(x − cell(x), y − cell(y))
}
// 256 CONSECUTIVE rejections ends the loop (the counter resets to 0x100 on every accept)
```

`CLocalDetailPlacementGrid`'s constructor (`:1102`) copies
`Layers[layer].SpacingFromLayer[0..=layer]` into `LayerSpacing` and chains `BaseLayer` to
layer − 1's grid, whose quad tree becomes this one's `BaseQuadTree`. So a layer's points keep
`SpacingFromLayer[j]` away from layer *j*'s points and `SpacingFromLayer[layer]` from their
own. `SetupPlacementGrids` (`engine_local_detail_generator.cpp:1360`) + `GetPlacementGrid`
(`:2065`) share one grid between generators whose spacing vectors match.

**Measured density** (a faithful re-run of the loop above, own-layer spacing only):

| spacing | 0.12 | 0.25 | 0.5 | 0.75 | 1.0 | 1.5 | 2.5 | 4.5 | 6.5 |
|---|---|---|---|---|---|---|---|---|---|
| elements / cell | 35.2 | 8.8 | 2.2 | 0.99 | 0.57 | 0.24 | 0.086 | 0.031 | 0.015 |

The densest spacing in the shipped data is 0.12 → 36,063 elements in a 32×32 tile, which
still fits the `u16` element index. Nothing overflows.

### A.3 The generation loop — per cell, per layer, per element

`CQuadTreeElement::GetPrimitivesFromMap` (`engine_local_detail_cache.cpp:4353`) walks the
node's cells in bit-interleaved (Morton) order, reads the three theme ids and their three
blend bytes at that cell (`PeekThemeId` / `PeekThemeBlend`, with §3.4's neighbour-map and
`CEngineStaticMapEdgeHeights` fallbacks), maps each theme id to its
`CLocalDetailGeneratorTheme`, and calls:

`CQuadTreeElement::AddObjectsFromBlendedThemes(map, x, y, themes[3], blends[3])` (`:3956`):

```
for layer in 0.. while any theme has this layer:
    for element in 0.. while any of the three grids has more elements here:
        pick = round(GetRandomDisplacement(worldX, worldY, …) * 255 − 0.5)
        theme = pick <= blends[0]              ? 0
              : pick <= blends[0]+blends[1]    ? 1 : 2      // weighted by the ground blend
        if themes[theme] has this layer and its grid has element `element` at this cell:
            AddObjectsFromLayerElement(…, PeekElement(grid, x, y, element), …)
```

`AddObjectsFromLayerElement` (`:3256`), read literally:

1. `sel = round(rand * 32 − 0.5)`; `obj = layer.ObjectSelectionTable[sel]`; **return if < 0**.
2. **return unless `blend/255 > obj.ThemeBlendThreshold`.**
3. `scale = (Scale + (2·rand − 1)·ScaleRandomElement) · 0.01` — the **same `× 0.01`
   mesh-units-to-world constant as §3.11**, here without `RenderSizeX`.
4. `pos.xy = cell + element_offset`; the map containing that point is re-resolved through
   `CEngineWorldMap`'s 32×32-cell tile grid.
5. `normal = PeekInterpolatedMapNormal(pos.x, pos.y)` (`engine_world_map.cpp:1688`) —
   bilinear over the four `PeekMapNormal` cell normals, then normalised.
6. **Slope rejection:** `t = clamp((n − SlopeFadeStart)/(SlopeFadeEnd − SlopeFadeStart))`,
   `t = 1` when `Start >= End`; **return if `t <= rand`**.
7. `angle = rand`; the 2×2 rotation comes from `GFastCosTable`, a 1024-entry
   `cos(2πi/1024)` table (`bbblibrary/lib_maths.cpp:45`) linearly interpolated, with sine as
   the same table offset by 256 entries.
8. If `TiltToSlope`, the rotation is composed with an orthonormal basis built from the
   landscape normal.
9. `pos.z` = **bilinear interpolation of the four corner heights**, each `PeekLandscapeHeight`
   quantised to 1/128 — the same accessor and quantisation `fable_data::landscape` already
   ports.
10. `AddObject(type, matrix, scale, angle_or_wind_delay, normal)` into a cache group.

`BuildObjectSelectionTable` (`engine_local_detail_theme.cpp:2395`) fills 32 slots by walking
the cumulative `Probability / Σ Probability` — **normalised**, so `Probability` is a relative
weight within the layer, never a chance of nothing. `−1` appears only when a layer has no
objects. This matters: **11 of the 101 shipped layers have `Σ Probability ≠ 1`** (0.30 …
1.10), and under this reading they are simply renormalised.

`GetRandomDisplacement(x, y, z)` (`engine_local_detail_generator.cpp:2190`) is
`DisplacementTable[x&31][y&31][z&31]` — a 32³ float table filled once in the generator's
constructor (`:84`) from the same `ror32(s·0x24a1 + 0x24df, 13)` chain. **It exists only when
the generator is constructed with dynamic update enabled** — retail streams the finished
cache instead, exactly as it streams landscape patches (§3.4). We port the builder, for the
same reason we ported `CEngineLandscapeMeshBuilder`.

### A.4 Measured: what a level actually costs

| level | cells | palette slots with a generator | est. objects | repeated / mesh / hybrid |
|---|---|---|---|---|
| Witchwood | 64×64 | 13 | ~1,100 | 243 / 794 / 90 |
| Darkwood | 96×64 | 10 | ~1,600 | 758 / 791 / 50 |
| LookoutPoint | 128×128 | 19 of 38 | **~19,600** | 19,017 / 484 / 139 |
| Arena | 64×64 | 0 | 0 | — |

LookoutPoint's local-detail mesh library is **35 distinct meshes, 61,837 vertices, 73,216
triangles** — uploaded once, and all 35 parse today.

**This is the single most important measurement in this note.** Local detail is ~100× the
placement count of `.tng` things, not ~10,000×. A whole level's objects fit in one instance
buffer per mesh (≈ 1.6 MB at the current 80-byte `ModelInstance`), generate in well under a
second, and need **no streaming, no quadtree and no cache-group machinery** to be correct.
The engine's `CLocalDetailCacheMap` exists to page a 30-map world in and out of an Xbox's
memory; we load one map at a time and can build it whole. That deletes about 9,000 lines of
decomp from the port surface.

### A.5 The repeated-mesh pipeline — exact

`engine_vs_layout_repeated_mesh.cpp:60-91` gives this pass its own register layout, and it
cross-checks against the disassembly on every register the shader touches:

| Range | Name | |
|---|---|---|
| `c19`–`c34` | `ObjectMatricies` | `.Offset = 0x13, .Count = 0x10` |
| `c35`–`c50` | `ObjectOffsets` | `.Offset = 0x23, .Count = 0x10` |
| `c51`–`c66` | `LightingResults` | `.Offset = 0x33, .Count = 0x10` |
| `c67`–`c82` | `MainLightLightingResults` | `.Offset = 0x43, .Count = 0x10` |
| `c83`–`c95` | `User` | `.Offset = 0x53, .Count = 0xd` |

**16 instances per draw**, addressed by `mov a.x, v3` — a vertex attribute carrying the
instance slot. `VSHADER_REPEATED_MESH`:

```asm
mov a.x, v3                          ; v3 = instance slot 0..15
mov r0, c[a + 19]                    ; per-instance (cos·s, sin·s, skew, skew)
mov r1, c[a + 35]                    ; per-instance (posX, posY, posZ, zScale)
mul r2, r0, c85                      ; c85 = User[2] = (1, -1, 1, …) -> row 0 of the rotation
mul r3, r0.yxzz, c0.yyyx             ; -> row 1
dp4 r5.x, v0, r2                     ; x' = cos·x − sin·y (+ skew terms)
dp4 r5.y, v0, r3                     ; y' = sin·x + cos·y
mul r5.z, v0.z, r1.w                 ; z' = z · zScale
add r5.xyz, r5.xyz, r1.xyz           ; + world position
dp4 oPos, r5, c5..c8                 ; c5..c8, WORLD-SPACE geometry (§3.11)
mov oD0.xyz, c[a + 51]               ; per-instance colour — NO per-vertex lighting
mov oD0.w, c0.y                      ; alpha 1 (the STIPPLE variant computes it, below)
mov oT0, v2
```

`PSHADER_REPEATED_MESH` is `mul_x2 r0.rgb, v0, t0; mov r0.w, t0.w` — identical in form to
`PSHADER_TEXTURE_DIFFUSE` with the per-object colour at 1.

Two consequences worth stating plainly:

- **Repeated meshes are lit per instance, on the CPU, from the landscape normal.**
  `ProcessLightingSW` (`engine_primitive_manager_repeated_meshes.cpp:1063`) calls
  `CalcSWLightingNoClip(pos, normal, lights)` per object into `LightingResults`. The vertex
  shader never sees a normal. So the per-instance colour is the *same*
  `Ambient + saturate(n·l)²·Diffuse + max(−n·l,0)·Backlight` our `model.wgsl` already
  computes — evaluated once per object with the ground normal instead of once per vertex.
  It arrives from **step 2's LUT rows 1/0/3**, like everything else.
- **Only a Z rotation and a Z scale survive** to a repeated mesh. `TiltToSlope` builds a full
  `CMatrix3x4` in `AddObjectsFromLayerElement`, but `CLocalDetailPrimitiveRepeatedMesh` stores
  `C4DVector ObjectMatricies` / `ObjectOffsets` — four floats each. Whether the tilt is
  dropped or folded into the two skew components is **open** (§A.9).

### A.6 Fade — screen-space stipple, not blending

`VSHADER_REPEATED_MESH_STIPPLE_ALPHA` adds:

```asm
add r1, r5.xyz, -c4                  ; c4 = CameraPos
dp3/rsq/rcp r1                       ; r1 = distance to camera, replicated
dp3 r2, r1, c84.xyz
add oD0.w, r2, c84.w                 ; alpha = distance·AlphaTransformMult + Offset
rcp r3.x, r0.w ; mul r2, r0, r3.x
mad oT1.xy, r2.xy, c86.xy, c86.zw    ; oT1 = SCREEN-space uv for the stipple texture
```

and `PSHADER_REPEATED_MESH_STIPPLE_ALPHA`:

```asm
tex t0                               ; diffuse
tex t1                               ; stipple / dither pattern, sampled in screen space
mul_x2 r0.xyz, v0, t0
+add r0.w, t1.w, v0_bias.w           ; v0_bias.w = fade − 0.5
cnd r0.w, r0.w, t0.w, c0             ; > 0.5 ? keep the texel's alpha : kill it
```

So foliage fades by **dithering out in screen space under an alpha test** — no blending, no
sorting, no depth-order problem. `c84 = User[1]`, written by `UploadShaderConstants`
(`:1208`) as `AlphaTransformMult / GPrimitiveRenderer[0x88]`; `c85 = User[2] = (1, −1, 1, …)`
and `c86 = User[3]` is the clip→screen transform. `FadeStart`/`FadeEnd` pass through
`ModifyFadeDistanceForVideoOptions` (`engine_local_detail_generator.cpp:718`), a
`RuntimeFadeDistanceModiferTable[1 << PrimitiveType]` of `{ClampMin, Factor}` filled from
`CEngineVideoOptionsDef` — 1.0 / 0.0 at full quality.

This is directly relevant to AGENTS §5 step 7.5 (`alpha_to_coverage`, deferred): **the
original's own answer to foliage silhouettes is a stipple, and we can transcribe it rather
than diverge.**

### A.7 What the ZSprite half is

`HYBRID_MESH_ZSPRITE` draws the real mesh inside `ZSpriteFadeStart`, and a camera-facing
impostor from `ZSpriteMesh` out to `FadeEnd` (`VSHADER_ZSPRITE` / `PSHADER_ZSPRITE`,
`CEnginePrimitiveManagerRepeatedZSprites`). In the shipped data `ZSpriteMesh` is usually
*the same mesh* — the impostor texture is generated at runtime by
`CEngineBillboardGenerator` (`fableengine/engine_billboard_generator.cpp`, 963 lines).
Typical numbers: `MEDIUM_TREE_FADE 100..118` with `ZFADE 52..57`, so the impostor covers
roughly half the visible range of trees.

That is a render-to-texture atlas subsystem of its own. §C defers it, and the honest
consequence is written down there.

### A.8 Scale, coordinates and lighting all match what we already have

- **`× 0.01`** — the same mesh-units constant as §3.11, sourced twice now.
- **Z-up** throughout; heights are `PeekLandscapeHeight` (× 2048, quantised to 1/128), the
  accessor `fable_data::landscape` already ports.
- **World-space geometry** through `c5..c8`, exactly like `.tng` things (§3.11) — no
  camera-relative subtraction.
- **Same four lighting constants** (`c3`/`c19`/`c20`/`c35`) as the landscape and the model
  pass, so local detail joins the queue behind **step 2** rather than adding a new blocker.

### A.9 Not settled — the accuracy boundary

Everything below is invisible in the decomp because it is register-passed, and each one
changes *exact* placement while changing nothing about plausibility. They are the items to
mark `// UNVERIFIED:` and to settle with §B.6's oracle.

1. **`GetRandomDisplacement`'s third argument.** The signature is `(long x, long y, long z)`
   and the body indexes `[x&31][y&31][z&31]`. The call sites increment a `long*` counter
   immediately before each call, so `z` is near-certainly "the n-th random draw at this
   cell". Whether that counter is per cell, per layer or per element, and whether it starts
   at 0 or 1, is not visible.
2. **The DisplacementTable's fill order and initial seed** (`:84`) — the loop nest is
   readable, the float mapping from the PRNG word is not.
3. **`BuildElementGrid`'s cell/offset split.** The `__ftol2_sse` arguments are FPU values.
   The encode/decode round trip (`(v + 0.5)·255` stored, `b/255 − 0.5` read) forces the
   offset into [−0.5, 0.5], so the split must be `cell = round(x)`, not `floor(x)` — a
   derivation from the storage format rather than a reading of the code. Worth stating as
   such.
4. **Which normal component the slope test uses.** `SlopeFade 0.80..0.90` on grass and
   `0..0` elsewhere makes "the normal's Z, i.e. flatness" overwhelmingly likely, but the
   decomp shows a struct member alias.
5. **Whether `TiltToSlope` survives into a repeated mesh** (§A.5).
6. **`c85`'s `w` component** and the meaning of `ObjectMatricies`' `z`/`w` — the wind skew.
   `SetupWindAnimation` / `UpdateBoneMatrixForWindSheer` are the places to read.

And one measured fact that **retires** a mechanism before we implement it:
**`ThemeBlendThreshold` is 0.00 on all 227 shipped objects.** The comparison must still be
transcribed (it rejects at zero blend), but it will never do anything else, and no visual
difference can be attributed to it.

---

## B. The plan — AGENTS.md §5 step 8

Same protocol as steps 5–7: derive, verify numerically, implement one mechanism per commit,
cite the oracle in the message. Each sub-step names its evidence.

### 8.1 `fable-data::local_detail` — the generator, data only, no rendering

New module beside `landscape/`, depending on nothing new:

```
local_detail/
  rng.rs         GFROR13 + the ror32(s·0x24a1 + 0x24df, 13) chain; DisplacementTable
  grid.rs        PlacementGrid: BuildElementGrid, PeekElement, PeekElementCount
  generator.rs   Generator/Layer/Object from the defs; ObjectSelectionTable
  place.rs       AddObjectsFromBlendedThemes + AddObjectsFromLayerElement over LandscapeMap
```

Output is a plain `Vec<LocalDetailObject> { mesh_id, primitive_type, transform: [[f32;4];4],
scale, normal, object_index }` — no renderer types, testable with no GPU and no Fable
install for everything except the def/level fixtures.

*Evidence, all numeric:*
- `rng.rs`: `GFROR13(x) == x.rotate_right(13)` as a unit test against
  `lib_global_tools.cpp:1133`; the chain constant `0x24df` asserted (Ghidra spells the same
  address three ways — `0x24da+5`, `0x24dc+3`, `&DAT_000024df` — and they are one constant).
- `grid.rs`: element counts per spacing match the table in §A.2 (a pinned regression, and the
  same numbers a second implementation must reproduce); every decoded offset lies in
  [−0.5, 0.5]; `PeekElementCount` summed over the 1024 cells equals `Elements.len()`;
  the toroidal wrap is exercised at the seam.
- `generator.rs`: `ObjectSelectionTable` is 32 entries, never negative when the layer has
  objects, and its histogram matches `Probability / Σ Probability` within 1/32 — run over
  all 101 shipped layers, including the 11 whose sums are not 1.
- `place.rs`: `openalbion probe --local-detail LookoutPoint` prints per-generator and
  per-mesh counts with provenance, and a test pins the totals in §A.4 (±, since they replace
  my estimate with the real number) and asserts **every object sits on the terrain** — the
  same gate `placement_test.rs` applies to `.tng` things, and for the same reason: the
  heights are bilinear over `PeekLandscapeHeight`, so a mismatch means the port is wrong.

### 8.2 `MESH`-type objects through the existing model pass

`scene::local_detail` resolves the objects to `(mesh asset id, ModelInstance)` and merges
them into the map `scene::things` already produces. **No renderer change at all** — this is
literally what `CLocalDetailPrimitiveMesh::AddObjectsToPrimitiveRenderer` does.

*Evidence:* Witchwood gains ~800 mesh-type objects over 20-odd meshes; the instance count per
mesh is logged; `--screenshot` shows mushrooms, stumps and saplings standing on the ground.
This is the first step where the screen is allowed to be evidence, and only as confirmation.

### 8.3 `HYBRID` objects as their mesh half

One line in the type match, plus a counter for "impostor not drawn". Puts the oaks, birches
and pines in the world at their `FadeEnd` (100–140 m) with no impostor. Logged as a divergence
with its number, so "trees do not degrade at distance" is a recorded decision.

### 8.4 `LocalDetailPass` — the repeated-mesh transcription

A new pass in `packages/renderer`, `local_detail.rs` + `local_detail.wgsl`, transcribing
`VSHADER_REPEATED_MESH` + `PSHADER_REPEATED_MESH` line by line per §6.7, with the asm in the
header.

**One structural divergence, declared:** the original packs 16 instances into vertex
constants `c19`/`c35`/`c51` and indexes them with an address register because vs_1_1 has no
instancing. We use a per-instance vertex buffer — `{ rotation: [f32;4], offset: [f32;4],
colour: [f32;4] }`, 48 bytes — and one draw per mesh. The *arithmetic* is transcribed
unchanged; only the delivery of the constants differs, and the 16-instance batching has no
observable effect to preserve. Same class of divergence as `model.wgsl`'s world-space normal
(§9), and it should be listed there.

Per-instance colour is computed in `scene::local_detail` from the interpolated landscape
normal with the same expression `model.wgsl` uses, so **step 2 lights local detail, terrain,
meshes and sky in one change** — no new lighting stub is introduced.

*Evidence:* a golden-image no-op is not available, so: instance counts per mesh, a test that
every generated colour equals the `model.wgsl` expression evaluated on that object's normal,
and LookoutPoint's ~19,000 grass instances rendering in one draw per mesh.

### 8.5 The stipple fade

`VSHADER_REPEATED_MESH_STIPPLE_ALPHA` + `PSHADER_REPEATED_MESH_STIPPLE_ALPHA`: the
distance→alpha transform from `FadeStart`/`FadeEnd`, the screen-space stipple sample, and the
`cnd` alpha kill. This is what makes 20-metre grass correct rather than merely present, and it
is also the honest answer to §5 step 7.5 — the original *does* have a foliage-silhouette
mechanism and it is not `alpha_to_coverage`.

Needs one thing we do not have: the stipple texture. Find what the engine binds to stage 1
(`SetupRenderModeShadersAndConstants` is the same render-state cache that defeated the
landscape blend modes, so expect this to be the hard part of the step) before inventing a
4×4 Bayer matrix.

### 8.6 The `.stb` oracle — a spike with its own go/no-go

`~/Fable/data/Levels/FinalAlbion_RT.stb` (598 MB) opens with `BBBB` — **the same bank
container §3.7 already decoded for the shader banks** — and `CLocalDetailCacheMap::
OpenStaticMap` (`engine_local_detail_cache.cpp:2373`) reads the shipped local-detail cache
out of it. `CObjectTypeCollection::Load` / `Save`, `CObjectTypeCollectionPalette::Load`,
`CQuadTreeElement::LoadHeader` / `LoadStaticContents` and
`CLocalDetailPrimitive{Mesh,RepeatedMesh,MeshZSpriteBatch}::Load` are all in the decomp and
readable, and `git show 1079634:fable_data/src/stb/mod.rs` is a partial directory parser.

If it lands, it is **tier-2 ground truth** in §6.1's sense — the engine's own object matrices
for a map, to diff ours against. It would settle every one of §A.9's six open items at once,
and it is the only route that can. If it does not land in a day, stop: §A.9's items are
plausibility-neutral, and the plan does not depend on it.

*Order:* attempt it **after 8.1** and before 8.4. After 8.1 we have something to diff; before
8.4 so that a placement bug is not chased through a shader.

---

## C. Deliberately not done — so each is a decision, not a drift

- **8.7 ZSprite impostors** (`SHADERS_ZSPRITE`, `CEngineBillboardGenerator`,
  `CEnginePrimitiveManagerRepeatedZSprites`). Consequence, stated: trees keep full geometry
  to their fade distance instead of collapsing to a billboard at ~55 m. Costs triangles;
  looks *better*, not worse; diverges from the original's silhouette at distance.
- **8.8 Wind animation.** `HasWindSkew` is set on 68 of 227 objects.
  `CLocalDetailPrimitiveRepeatedMesh::SetupWindAnimation` and `UpdateBoneMatrixForWindSheer`
  drive `ObjectMatricies`' skew components per frame from `WindSkew{Constant,Random,Speed}Factor`
  and a per-object `WindDelayArray` byte. Land the pass first with the skew at zero; the
  vertex layout already carries it.
- **8.9 Shadow meshes and `CastShadows`.** 40 objects carry a distinct `ShadowMesh`; nothing
  in the renderer casts a shadow yet.
- **8.10 The cache/quadtree/streaming machinery** — `CLocalDetailCacheMap`'s ~9,000 lines.
  Justified by §A.4: a whole map's objects fit in memory. Revisit only when neighbouring maps
  load (the twin of 5.10 / 6.12).
- **8.11 Dynamic areas** (`AreaChanged`, `UpdateDynamicArea`, `ConsoleAddLocalDetail*`) — the
  editor path.
- **8.12 Video-options fade scaling.** `ModifyFadeDistanceForVideoOptions` is understood;
  factor 1.0 and clamp 0.0 are the full-quality values, so it is a knob with nothing to turn.
- **8.13 Local lights and shadowed variants** of the repeated-mesh shaders — the twin of 6.10.

---

## D. Decisions I need from you before 8.1

1. **Generate, or read the shipped cache?** I recommend **generate** (port the builder), on
   §3.4's precedent and because the `.stb` is a cache of exactly this computation — with the
   `.stb` demoted to §B.6's *verification* role rather than the data path. The counter-case
   is real though: reading it would be exact by construction, and 47 of 227 objects are
   HYBRID whose exact placement we may otherwise never confirm.
2. **Step 8.2/8.3 before 8.4?** It gets trees and mushrooms on screen with zero renderer
   changes and proves the generator, at the cost of one commit that draws only 139 of ~19,600
   LookoutPoint objects — the level will look *emptier* than the eventual result before it
   looks fuller. I think that ordering is right (it isolates the data layer from the new
   pipeline), but it is a visibly unsatisfying middle state and you may prefer 8.4 first.
3. **The 32-cell repeat.** Placement tiles exactly every 32 world units, by construction
   (§A.2). It is faithful, and it will be visible on open ground. Transcribe it, or is this a
   place you want an `ACCEPTED` divergence later (e.g. a second displacement octave)?
4. **Scope of the first landing.** Everything above is one level, loaded whole, no streaming.
   Confirm that is still the working assumption (it is what 5.10 / 6.12 already defer).
