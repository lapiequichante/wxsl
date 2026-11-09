# Glossary

Terms used elsewhere in this repo's docs without re-explanation.

**WGSL** — WebGPU Shading Language: the shader language `wgpu`/WebGPU
consumes natively. No imports, no conditional compilation.

**WXSL** ("WGSL Extended") — this project's shading language: WGSL plus
templates, imports, `@if`/`@elif`/`@else` conditional translation and
`@macro const` values. `wxsl-lang` compiles it to plain WGSL. It began as
[WESL](https://wesl-lang.dev) and became its own language when WESL's
generics fell short;

**Node graph** — the user-facing/data-model representation of a shader as
connected nodes and sockets (`wxsl_core::graph`). Compiles to a WXSL
module. Distinct from a *render graph*, and the two are easy to confuse
because both are called "the graph" in conversation: a node graph is what a
material *is*, a render graph is how a *frame* is put together.

**Render graph** — the list of passes and the resources they read and write
that make up one frame (`wxsl_frame::graph::FrameGraph`, recorded on wgpu
as `wxsl_render::graph::RenderGraph`, built from
`wxsl_render::pass::PassDesc`). The engine orders the passes by their
dependencies, allocates and reuses the transient targets, rotates the
persistent ones, and records them. A *pipeline* is a render graph; forward
and deferred are two of them.

**Effect** — a pass as a self-describing unit
(`wxsl_render::effect::Effect`): declared inputs and outputs in binding
order, screen or compute entry points, and its shader. A pipeline
document's `pass.screen` names one by id, and a chain of them — lighting
into a colour target, bloom over it — is how a pipeline composes.

**Policy** — how often a pass runs: `per frame` (the default), `once`,
`on resize` or `on demand`
(`wxsl_render::pass::Policy`). The renderer's frame loop
honours it; a pass of any non-default policy may write only stable
storage — a persistent target with no history — which is the scheduler's
check, and is what makes skipping safe. `Renderer::pass_run_count` and
`mark_pass` are the observable halves.

**Channel plan** — the collected G-buffer channel requests, each tagged
with its source: the base layout, a lighting model, or a material
feature (`wxsl_core::lighting::GBufferPlan`). The layout, the
generated pack, the lighting pass and the budget check all read it; a
field claimed twice is an error naming both claimants.

**Material feature** — a registry entry
(`wxsl_core::lighting::MaterialFeature`) that owns a G-buffer channel on
the models' behalf: a macro pin turns it on per material, a pack function
in the feature's module fills the shared model target, and the model that
consumes the feature reads it there. Subsurface ships first.

**Scene** — the document: meshes, materials and instances, as pure
serializable data (`wxsl_core::scene::Scene`). It says what exists, never
how it is drawn — the pipeline belongs to the renderer, not to the
document. Not to be confused with the *environment*.

**Environment** — camera, lights, ambient, exposure and time
(`wxsl_render::environment::Environment`). The other half of a frame,
alongside the draws. Called `Scene` until M1, which is why the rename
happened.

**Parameter** — a value a material's graph declares (`param.value`) that
lives in a uniform buffer rather than in the shader text, so the host
changes it with a buffer write and no recompile. The counterpart to a
*constant*, which is inlined and therefore costs a new shader variant when
it changes.

**Interface** — what a material needs before it can draw: its parameters
and their computed offsets, its textures and samplers, the block it
expects the application to supply, and what it requires of its geometry
(`wxsl_core::resources::MaterialInterface`). Computed from the graph, and
the single source the generated WGSL, the bind groups and the vertex
layout all come from.

**Attribute** — a value the *geometry* supplies, declared by the graph and
read by `input.attribute`: per **vertex**, from a stream the mesh must
carry under that name, or per **instance**, from a value the draw must
supply. The frequency belongs to the declaration and not to the reading
node, so moving one between the two rewires nothing. Distinct from a
*parameter*, which the material owns and fills itself.

**Setting** — a string-valued property of a *node instance* that changes
what it compiles to, such as the name of the parameter a `param.value`
declares (`wxsl_core::node::SettingDef`). Distinct from a node's label and
colour, which are metadata a reader chooses and codegen never sees.

**Terminal** — a graph's output node. `output.surface` (required),
`output.vertex` (an object-space position offset), `output.discard`,
`output.fragment` (the colour written in place of the lit surface), and
one `output.varying` per declared **interpolant**. Each is the root of
its own **partition**.

**Interpolant** — a value the graph's own vertex stage computes and the
fragment stage reads back down an inter-stage location. Declared as an
attribute of frequency `computed`, written by `output.varying`, and read
by the same `input.attribute` node that reads a stream the mesh carries.

**View** — a point of view a pass renders from: the camera, or the one a
light casts its shadow from. Named by `PassDesc::view`, and reached by a
dynamic offset on the frame group's camera binding, so no shader knows
which one it is running under.

**Partition** — the subgraph reachable from one terminal, compiled into
one function. A node reachable from two terminals is compiled into both.
Which partitions a stage compiles is what makes a depth or shadow pass
cheap, and what makes a displaced or perforated object cast a matching
shadow.

**Tags** — open-ended labels a material or an instance is authored with
(`opaque`, `transparent`, `outlined`). A geometry pass draws a *tag
expression* over the draw list, so the material says what it is and the
pass says what it draws.

**Material stage** — which entry point a material graph is compiled for,
and therefore what its fragment stage writes: `forward_lit` (a final
colour), `gbuffer` (the G-buffer struct) or `depth_only` (nothing at all —
no fragment stage). A row in `wxsl_core::abi::MATERIAL_STAGES`, named by a
pass, never by a graph. This replaced the two-valued `RenderPath`, which
had room for exactly two entry points —

**Pipeline** — a render graph. `StockPipeline::{Forward, Deferred}` are the
two this crate ships: forward is a depth prepass plus a shading pass,
deferred is a G-buffer pass plus a fullscreen lighting pass. "Which
pipeline" and "which stage" are separate questions, which is why they are
separate types.

**Shader variant** — a specific compiled WGSL output for one
`(generated source, macro values, stage)` combination, cached by
`wxsl_render::variants`. Switching pipeline or flipping a macro variable
asks for a different variant of the same material. The key holds the
*stage* rather than the pipeline, which is what makes the second swap
between two pipelines free.

**Shader ABI** — the fixed WXSL vocabulary a generated material module is
written against: the `SurfaceContext` it is given, the `Surface` it returns,
the uniform bindings, the vertex stage, the lighting function and the
G-buffer layout. Named in `wxsl_core::abi`, implemented in
`wxsl-stdlib`'s `shaders/wxsl/` — except that the lighting function and
the G-buffer layout are *generated* from the enabled lighting models.

**Lighting model** — one WXSL function of a fixed signature plus a small
integer id, shaded through a registry entry in `wxsl_core::lighting`. `lambert`, `phong`, `pbr`, `clearcoat`, `cloth`, `iridescent`,
`sheen` and `subsurface` ship; a material names its model by name, and a
deferred pipeline enables a **set** of them, which decides whether the
G-buffer carries the shared model target and the dispatch the generated
lighting pass performs.

**Surface** — what a material graph produces: base colour, metallic,
roughness, normal, emissive, occlusion and alpha at one point. A graph
describes a surface, not a whole shader — the entry points, light loop,
lighting-model dispatch and G-buffer packing are around it, not in it.

**Macro variable** — a graph-level knob that changes the *shape* of the
generated shader rather than a value flowing through it: a flag becomes a
WXSL `@if` condition, a number becomes a `const` declaration (so it can be a
loop bound or array size). Declared by node definitions, pinned in the node
format, part of the variant cache key. `wxsl_core::macros`.

**Node format** — the serialized form of a graph (`serde`, JSON in the
demo): nodes with stable ids and their pinned socket values, edges between
sockets, and the graph's macro variables. See
`crates/wxsl/assets/pbr_cube.wxsl.json`.

**LYGIA** — an existing, mature, multi-language (GLSL/HLSL/MSL/WGSL/CUDA)
granular shader function library by Patricio Gonzalez Vivo
([lygia.xyz](https://lygia.xyz)), licensed under the Prosperity Public
License 3.0.0 (not MIT/Apache). `wxsl` does not depend on or port it. The standard library contains original
functions and audited permissive ports; see `crates/wxsl-stdlib/shaders/README.md`.

**Package** — the first, owning segment of a registry id: `math.add`,
`wxsl.bloom`, `wxsl.pbr`. The shipped effects and models live under
`wxsl.`; a document may spell shipped ids bare (`bloom` means
`wxsl.bloom`), a registration may not.

**Capability check** — the device-free comparison of a scene against the
setup it will run under (`wxsl_render::setup`): a setup publishes its
capabilities — stages, channel plan, lighting set, effects — and the check
reports every mismatch by name, at load, before anything is built.

**Bake** — a part of a material precomputed into a small 2D table and
sampled back: the material graph declares "bake *this node's* output",
the table's shader is generated from that subgraph and runs as an
ordinary compute pass (`once`, or `on demand` with `mark_pass`), and the
material either samples the table or evaluates the subgraph inline — the
toggle changes cost, not image. The table is host-owned: the pass list
writes through it as an imported resource, the material samples it like
any texture of its own.

**Velocity** — screen motion per fragment: where this fragment was last
frame, as uv per frame, `y` down. The `velocity` **material stage**'s
vertex entry transforms each vertex against this frame's camera and
instance row *and* the previous frame's (the frame group carries both);
the fragment writes the difference. Consumers read it — TAA to
reproject the history, motion blur to smear along it. A velocity
target's unwritten texel is *no motion*, not the frame's clear colour.

**TAA** — temporal antialiasing, as a policy'd chain: a velocity pass, a
`resource.color` with `history: 1`, and a resolve
(`wxsl.taa`) that blends the frame with its reprojected history and
writes the blend back into the ring. The disocclusion answer is a
clamp — the history survives this frame's neighbourhood box or it does
not get blended — plus a falloff measured against the current pixel,
which is what keeps an unwritten first frame from poisoning the ring.

**ADR** — Architecture Decision Record; see `docs/adr/README.md`.

**Facade crate** — `wxsl`, the crate most consumers depend on directly;
re-exports the other workspace crates behind Cargo features rather than
being depended on itself.
