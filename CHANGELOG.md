# Changelog

Changes to the stable surface: what each crate exports at its root, the
document formats and the C ABI.
A change to a stable item updates its `api/<crate>.txt` snapshot and adds an
entry here. Module paths and `wxsl-frame` are unstable and change without
an entry.

## [Unreleased]

### Added

- Select specular environment integration with `Renderer::set_brdf_lut`:
  UE4 Schlick-GGX (default), Disney separable Smith-GGX, or Filament
  height-correlated Smith-GGX. Both bake paths share the selection.

- GPU particles: `effect::particles`, the shipped `wxsl.particles_simulate`
  (compute) and `wxsl.particles_draw` (sprites) effects over one stable
  buffer, added by `particles::append`. The shipped effect registry now holds
  46 effects.

- `ValueType::Texture3d` (`texture_3d<f32>`) and the `texture.texture_3d`
  node: a material binds a volume by name. An exhaustive `match` on
  `ValueType` needs the new arm.

- `wxsl_prt::Bvh` and `wxsl_prt::Hit`: the baker's triangle hierarchy, for
  offline ray queries beside a bake (the garage's window light).

- `Environment::planar_reflection` and a raster planar-reflection recipe,
  using a validated plane from `wxsl_core::reflection::PlanarReflection`
  without runtime ray tracing. Full `Environment` literals need
  `planar_reflection: None`.

- Outside penumbrae: `Light::with_penumbra(Penumbra::Outside)` puts a soft
  shadow's penumbra outside its hard edge, its inside staying dark, and
  filters a contact shadow inside a far caster's penumbra at its own
  width. Its filter is twice as wide, for the penumbra to keep a centred
  one's width. In the C header, the light row's first `_padding3` float is now
  `penumbra`.

- Disc lights (`Light::disc`, `LightKind::Disc`): a one-sided round
  emitter, integrated as an octagon by the LTC models and sampled as a
  masked rectangle by the others. Unshadowed, and bounded by `range`.

- `Renderer::set_environment_scale`: scale the environment's radiance from
  the next frame without installing the pass list again.

- `PipelineConfig::shadow_atlas_size` and `Environment::shadow_atlas_size`:
  the shadow atlas edge, a power of two from 512 to 8192, 2048 by default
  (`abi::valid_shadow_atlas_size`, `abi::shadow_tile_in`).
  `Renderer::shadow_atlas_size` reports the installed pass list's, and a
  frame whose environment disagrees is `RenderError::ShadowAtlasSize`; a
  document asking for an invalid edge is `PipelineError::ShadowAtlasSize`.
  Struct literals of either type need the new field.

- `wxsl.iridescent` has an ambient, `ambient_iridescent`: the film now
  colours the environment's reflection too, not only direct light.

- `wxsl_prt::ground` and `GroundGrid`: a flat grid under a model, as a
  `.glb` in its space, and `wxsl_prt::bake_receiver`, which bakes such a
  surface against the model without it shadowing anything. The binary
  gains `--ground`, writing `<model>_ground.glb` and its bakes.

- `Light::with_cascade_split` (`shadow_cascade_split`): how logarithmic a
  cascaded light's splits are, 0.5 by default as before. And
  `Light::with_caster_reach` (`shadow_caster_reach`): how far towards the
  light its casters are drawn, `shadow_distance` by default.

- `Light::with_shadow_near` (`shadow_near`): where a point or spot light's
  shadow views start, a thousandth of `shadow_distance` (at least 0.01) by
  default as before.

- Local probes. `Environment::probe` (a `LocalProbe`) adds six cube-face
  views, `PassView::Probe`. `ibl::append_probe` captures them every frame
  through the new `wxsl.probe_face` effect, then filters and convolves
  them. `PassDesc::with_environment` lets one pass bind those cubes in
  place of the frame's. `wxsl.ibl_diffuse` and `wxsl.ibl_specular` take a
  `samples` parameter; its default, 0, keeps a bake's quadrature.
  `Environment` gains the `probe` field, so a literal one needs
  `probe: None`.

- Static shadows. `Light::with_static_shadow(frame)` draws a spot
  light's shadow tile once and keeps it while the light holds still in
  its casters' frame. `Light::with_shadow_casters` narrows any light's
  shadow to the draws a tag expression matches.
  `Renderer::invalidate_static_shadows` redraws them. `Light` is now
  `Clone` and no longer `Copy`.

- GPU skinning. `output.vertex` takes optional object-space `normal` and
  `tangent` inputs, `input.object_tangent` reads the mesh's tangent, and
  `animation.skin` blends four joints from a bone texture. The glTF import
  keeps skins and clips (`ImportedScene::skins`, `animations`,
  `material_skins`), imports `JOINTS_0`/`WEIGHTS_0` as the `joints` and
  `weights` streams, and skins a skinned primitive's material from
  `bone_texture` at `bone_frame`, which `BoneTexture::bake` fills.

- `ColorSpace::Normal` (`"normal"` in a scene document) and the
  `lighting.normal_spread_roughness` node. A normal map's mips keep their
  mean direction and, in alpha, the roughness the averaged-away detail
  adds; the glTF importer reads normal and coat-normal textures that way
  and widens the matching roughness, so mip transitions stop showing on
  flake paint and brushed metal.
- `output.fragment`, and a `color` output on `output.surface`: nodes
  between the two change a material's lit colour, under either pipeline.
- An `exposure` input on the tonemap nodes, and an `exposure` parameter on
  the `tonemap` effect. Both default to 1.
- Every shipped lighting model in `default_set()`, now that they fit.
- `double_sided` on a material (`MaterialConfig`, and the scene
  document's material entries): both faces are drawn, a back face lit
  with its normal flipped. Opaque passes stop culling it; blended passes
  draw its back faces, then its front faces. glTF's `doubleSided` sets it
  and is no longer a warning.
- The `wxsl.gltf_clearcoat` lighting model, and `clearcoat`,
  `clearcoat_roughness` and `clearcoat_normal` inputs on `output.surface`:
  a clear coat with glTF's parameters. The glTF importer maps
  `KHR_materials_clearcoat` onto it, factors, textures and coat normal
  included, instead of onto `wxsl.clearcoat`.
- `diffuse_occlusion` and `specular_occlusion` inputs on `output.surface`,
  both 1 by default. They scale the environment maps' diffuse and
  specular light on the forward path; deferred draws without them.
- The `wxsl-prt` crate and its `wxsl-prt` binary. They bake precomputed
  radiance transfer for a glTF file on the CPU: shadows, plus diffuse
  bounces coloured by each material's albedo sampled at the hit's UV.
  The bake is written in grayscale and in colour, to `<model>.gray.prt`
  and `<model>.color.prt`. It also rewires imported materials to read
  the bake through the new `lighting.prt_*` and `lighting.sh_irradiance`
  nodes. The `viewer` example loads both files, and `P` cycles off,
  grayscale and colour. `--albedo` (`Albedo::base_color_only`) colours
  the bounces by the base colour alone, ignoring metallic.
- `ImportedScene::material_sources`: the glTF material each imported
  material was built from.
- `lighting.prt_window`: an environment coefficient's luminance, damped
  by SH band, as the PRT diffuse nodes read it.
- `input.normal_matrix`: the vertex stage's object-to-world matrix for
  directions, so a graph can turn the normal its vertex output replaces
  into world space.
- `transmission` and `thickness` inputs on `output.surface`, both 1 by
  default. `pass.refraction` mixes `1 - transmission` of the glass's
  diffuse base colour into what it lets through, and composes a
  thickness of 0 as thin-walled glass: one interface, no bending, tinted
  by the base colour, over its alpha. The `refraction_surface` stage
  writes a third target. The glTF importer maps `transmissionFactor`,
  and a material without `KHR_materials_volume` is thin-walled.
- Contact-hardening shadows: `Light::with_source_radius` gives a point,
  spot or directional light a source size, and its shadow is sharp where
  it touches its caster and wider farther away. The light row's `radius`
  carries it (a directional light's as the tangent of its angular
  radius); the C layout is unchanged.
- IES profiles on point and spot lights: `Light::with_profile` names a
  cookie layer holding a profile baked by `IesProfile::texels`
  (`wxsl_render::environment`). In the C header, the light row's
  `_padding2` is now `profile_layer`, and `UINT32_MAX` means none.
- Tube lights (`Light::tube`, `LightKind::Tube`) and textured area
  lights: `Light::with_cookie` also takes a rectangle or a tube, which
  then emits that cookie layer. An unshadowed rectangle and a tube place
  their samples around the surface's falloff and specular lobe, so
  glossy reflections are continuous and prefiltered by roughness. The
  host layout's `LightUniform._padding1` is now `radius`.
- `LightingModel::area`: a model may integrate unshadowed rectangles and
  tubes analytically with linearly transformed cosines. `wxsl.pbr` and
  `wxsl.lambert` do. The environment LUT is now 128² in four quadrants,
  the LTC table beside the split-sum pair, and
  `abi::ENVIRONMENT_LUT_SIZE` follows.

### Changed

- `pass.screen.wxsl.motion_blur` is a reconstruction filter: a document
  also wires the opaque `depth` and a `neighbors` image, written by the
  new `wxsl.motion_tiles` and `wxsl.motion_neighbors` screen effects into
  two `pair` targets at `effect::MOTION_TILES_SCALE`. A fast foreground
  now smears over a still background.
- Cascaded shadows are fitted without the camera's near plane, so their
  texel size holds while it moves and their shadows no longer pop; the
  splits move slightly. The last cascade fades out instead of ending on a
  line.
- Thin-walled glass (`pass.refraction`) reflects through an iridescent
  film: `iridescence_strength`, `_thickness` and `_ior` on a thin optical
  layer turn its Fresnel spectral, so `KHR_materials_iridescence` on a
  transmissive glTF material shows. The film is quantised to 10 nm,
  0.1 IOR and quarter strengths.
- `wxsl-prt` bakes the scene posed at the first frame of its first clip,
  skinned on the CPU (`BakeOptions::clip`, `--clip N`, `--rest`), and
  `apply` decodes a skinned material against its skinned normal. The
  `viewer` uses bakes on skinned models.
- `wxsl-prt` bounces are stronger: three by default, up from one; each
  adds the surface's specular reflectance to its diffuse colour
  (`Albedo::sample`); the back of a double-sided material bounces light;
  and an alpha-masked material occludes only where it is solid.
- The PRT diffuse nodes divide on the environment's windowed luminance,
  which removes the green and purple fringes SH ringing caused under a
  low sun.
- The clip position is `@invariant`, so a skinned mesh's depth prepass
  and colour pass agree to the bit; on Metal its triangles flickered
  black.
- A material may declare six per-vertex attributes, up from four.
- The G-buffer stores the normal octahedrally, with the lighting model's
  id beside it, and every model's data in one shared `model` target:
  24 bytes per sample, or 32 with model data, whatever the set.
  The `lighting` target and the per-model targets are gone.
- The editor's shortcuts follow the keyboard layout, and its glyphs are
  always generated on the GPU (the toolbar's `msdf` button, the `G` key
  and the example's `--msdf` option are gone).

### Fixed

- Correct the BRDF LUT sampling weight to use `V·H`. Existing environment
  highlights and area-light Fresnel weighting can change.
- SSR uses the scene's declared environment mip count instead of
  `textureNumLevels`, keeping its shader portable to WebGL2.

- A shadow is biased along the geometry's normal rather than the shading
  normal (forward only; the G-buffer keeps the shading one): a normal map
  grained its edges under a grazing light.
- `generative.simplex3` hashed its lattice without the cell's `y` and `x`
  offsets, so its noise repeated along those axes in straight-edged
  blocks.

## [0.1.0] - 2026-10-16

The first release. The workspace's crates share one version.

### Added

- **Materials.** Node graphs compiled to WXSL, then to WGSL, one module
  per material stage, over a library of about 100 nodes (`wxsl-stdlib`).
- **The WXSL compiler** (`wxsl-lang`): WGSL plus imports, conditional
  compilation, macro constants and templates.
- **Pipelines as documents.** Forward and deferred presets, effects as
  nodes (bloom, TAA, depth of field, motion blur, refraction, depth
  peeling), compiled to a pass list the renderer schedules and records.
- **The renderer** (`wxsl-render`): wgpu on native, WebGPU and WebGL2;
  shadows, image-based lighting, clustered lights, instanced draws and
  per-view culling for large scenes.
- **Scenes and glTF.** Scene documents, and GLB import into them with
  generated material graphs (`wxsl-gltf`, the `gltf` feature).
- **The editor** (`wxsl-editor`, the `editor` feature): materials,
  pipelines and scenes, with undoable commands, debug views, a pixel
  probe and a frame panel.
- **An MCP server** (`wxsl-mcp`, the `mcp` feature) that lets an agent
  drive the editor through its commands.
- **The C ABI** (`wxsl-ffi`) and the Dawn backend for C and C++ hosts.

[Unreleased]: https://github.com/lapiequichante/wxsl/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/lapiequichante/wxsl/releases/tag/v0.1.0
