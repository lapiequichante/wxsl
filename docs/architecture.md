# Architecture

How the pieces fit together and the invariants their implementations share.

## In one paragraph

A material is a **node graph** (authored in the editor, in code, or as a
JSON document). It compiles to **WXSL**, which `wxsl-lang` lowers to WGSL.
A **pipeline** is another graph, over a vocabulary of passes and resources,
which compiles to a list of passes the scheduler orders and allocates. One
material graph compiles into every pass that draws it, so forward,
deferred, shadows and motion vectors never require the author to write
anything twice. Everything up to the device is device-free Rust, shared by
two backends: wgpu in Rust, and Dawn in C++ through a data-only C ABI.

The API and external applications have separate repositories; see the
[repository boundary decision](adr/0001-separate-api-and-applications.md).

## Crates

```mermaid
graph LR
    core["wxsl-core<br/>graph model, documents, codegen"]
    lang["wxsl-lang<br/>WXSL → WGSL compiler"]
    frame["wxsl-frame<br/>frame plans, scheduling"]
    ffi["wxsl-ffi<br/>data-only C ABI"]
    stdlib["wxsl-stdlib<br/>node library"]
    gltf["wxsl-gltf<br/>glTF import"]
    render["wxsl-render<br/>wgpu renderer + UI layer"]
    editor["wxsl-editor<br/>visual editor"]
    mcp["wxsl-mcp<br/>MCP server"]
    prt["wxsl-prt<br/>offline PRT bake"]
    facade["wxsl<br/>facade"]

    lang --> core
    frame --> core
    stdlib --> core
    stdlib -. "build only" .-> lang
    gltf --> core & frame
    ffi --> core & frame & lang & stdlib & gltf
    render --> core & frame & lang
    render -. "gltf feature" .-> gltf
    editor --> core & render & lang
    mcp --> core & editor & render
    prt --> core & gltf & stdlib
    facade --> core
    facade -. features .-> render & editor & stdlib & mcp
```

Arrows point from dependent to dependency.
The dependency boundaries in [AGENTS.md](../AGENTS.md) must hold.
Several edges need explaining:

- **`stdlib → lang` is build-only.** Nodes are derived from their `.wxsl`
  files at build time, so the shipped library carries no compiler.
- **`editor → render`.** The editor draws itself with the renderer.
- **`editor → lang`.** The code panels reuse the compiler's lexer for
  syntax highlighting.
- **`wxsl-gltf` takes the registry as an argument.** It never depends on
  `wxsl-stdlib`, so `wxsl-render` can reuse its geometry reader and C++
  gets the same import through `wxsl-ffi`.

- **`wxsl-prt` is an offline tool.** It bakes precomputed radiance
  transfer on the CPU and rewrites imported graphs to read it. No other
  crate depends on it; the `viewer` example uses it as a dev-dependency.

`wxsl-core`, `wxsl-frame`, `wxsl-lang`, `wxsl-gltf`, `wxsl-prt` and
`wxsl-ffi` never touch a device.
That is what lets the scheduler, layouts and compiler be tested in CI
without a GPU, and shared with Dawn.

### Facade features

| Feature | Default | Adds |
|---|---|---|
| `render` | on | `wxsl-render` |
| `stdlib` | on | `wxsl-stdlib`, and `wxsl::stdlib_library()` to hand it to the renderer |
| `editor` | off | `wxsl-editor` (implies `render`) |
| `images` | off | PNG and JPEG decoding for scene textures (implies `render`) |
| `gltf` | off | glTF/GLB import: `file` meshes, and with `stdlib` whole scenes through `wxsl::asset` (implies `images`) |
| `webgl` | off | wgpu's WebGL2 backend on `wasm32` |
| `mcp` | off | `wxsl-mcp`, an MCP server over the editor's commands (implies `editor`) |

With `default-features = false`, `wxsl` is the graph model alone: no wgpu,
no GUI. The renderer ships no shaders of its own; the application supplies
a `ShaderLibrary`.

## Documents

Everything a user authors is a versioned JSON document. Each one carries
`version` and `abi` fields. A missing field means "this build". A version
newer than the build is refused by name. Saving stamps both fields. Registry
ids are namespaced `package.name`.

There is **one graph model with three domains**.
A node definition lists the domains it may appear in, and
`NodeRegistry::in_domain` filters the palette.

| Domain | Graph means | Compiled by | Into |
|---|---|---|---|
| `surface` | a material | `codegen::generate` | one module per material stage |
| `screen` | a post effect | `codegen::generate_screen` | one fullscreen pass |
| `document` | a pipeline | `wxsl_frame::pipeline_doc::compile` | a `FrameGraph` |

The **scene document** (`wxsl_core::scene`) lists the meshes, materials and
instances that exist, and says nothing about how they are drawn.
`RenderSetup::check` checks a scene against the pipeline it will run
under, and reports every mismatch.

## Materials

### Terminals and stages

A material graph has several outputs:

- **`output.surface`**, required: a `Surface` (base colour, metallic,
  roughness, normal, emissive, occlusion, alpha).
- **`output.vertex`**, optional: an object-space offset added to each
  vertex.
- **`output.discard`**, optional: an alpha test.
- **`output.fragment`**, optional: the colour written in place of the lit
  surface, fed from `output.surface`'s `color` output through any nodes
  (fog, a grade). Under deferred such a material shades forward.
- **`output.varying`**, one per declared interpolant.

Codegen partitions the graph by what each output reaches. A **material
stage** decides which parts are compiled: a depth or shadow pass gets the
vertex offset and the discard test, never the shading. That is why a
displaced object casts a displaced shadow.

Stages are rows in `abi::MATERIAL_STAGES`:

- `forward_lit`, `gbuffer`, `depth_only`;
- `velocity`;
- the peel and refraction stages.

A pass names its stage; the graph never does.

**Stage analysis** decides where each node runs.
By default a node runs in the earliest stage that can produce it, and its
value crosses to the fragment stage as a synthesized interpolant. A node
that uses derivatives is fragment-only, and placing it anywhere else is a
named error.

### What a material declares

`wxsl_core::resources::MaterialInterface` is computed from the reachable
graph. Codegen and the bind-group builder both read it, so the two cannot
disagree.

- **Parameters** (`param.value`) are fields of a uniform buffer. Changing
  one is a buffer write, not a recompile. A `const`, by contrast, is
  inlined.
- **Textures and samplers** are bound by name.
- **The user block** is shaped by the material and filled by the
  application.
- **Attributes** come per vertex (a named mesh stream), per instance (a row
  supplied with the draw), or computed (the graph's vertex stage writes
  them).

There is no `#[repr(C)]` mirror of these layouts: `BufferLayout` computes
WGSL's alignment rules and is the only thing that knows them.

Everything that is configuration rather than a node is one value,
`MaterialConfig`:

- macros;
- the lighting model's name;
- the shadow flags;
- tags;
- forward shading;
- `max_layers`;
- feature channels.

`MaterialConfig::resolve` is the only place a name becomes an id.

### Macro variables

A value that must be known at compile time, such as a loop bound or a code
switch, is a macro (`wxsl_core::macros`).

- A `Flag` becomes an `@if` feature.
- An `Int` or `Float` becomes a module `const`.

Precedence, weakest first:

1. the file's default;
2. the node's default;
3. the graph's pin;
4. the application's override.

The whole set is part of the variant cache key.

## Lighting

**Lighting models** are WXSL functions with a fixed signature, under
`shaders/lighting/models/`, registered in `lighting::DEFAULT_MODELS`.

- A material names its model.
- A pipeline enables a `LightingSet`. The set determines:
  - the G-buffer layout;
  - the generated dispatch: a direct call when forward, a `switch` on the
    id the normal target carries when deferred;
  - the attachment-budget check.
- **Every model's data shares one target**, read by the pixel's id, so the
  G-buffer is 24 bytes per sample, or 32 with model data, whatever the set.
- **Material features** such as subsurface fill that target for the models
  that consume them.

The shading function, the G-buffer pack and the lighting pass are
generated from the set; never edit the generated text. Forward and deferred
call the same generated `shade_surface`, and tests assert they produce the
same image.

A material may also **shade forward under a deferred pipeline**. Its
radiance passes through the G-buffer unchanged. This is the route for
screen-space and derivative effects.

**Lights** come in two kinds:

- **Up to sixteen slots in the scene uniform**, for any type: directional,
  point, spot, rectangle. These lights can cast shadows, drawn into one
  depth atlas of shared tiles:
  - cascades for directional lights;
  - six faces for point lights;
  - sixteen samples for area lights;
  - cookies on spotlights.

- **Thousands of ranged lights without shadows.** The host bins them into a
  16 × 9 × 24 grid over the camera frustum.

**Ambient light and the environment** have three parts:

- **The split-sum BRDF table** is baked once by a compute effect.
- Select the specular LUT with `Renderer::set_brdf_lut`: UE4 (default),
  Disney isotropic GGX, or Filament correlated GGX. A changed choice is
  rebaked before rendering; the material graph and host layouts are unchanged.

- **Environment cubes** are explicit frame resources, prefiltered from an
  HDR image or from the sky node through face passes.
- **Without them**, the analytic environment remains.

**Light stays linear** until the last pass. Every pass writes linear
radiance; the `tonemap` effect at the end of the chain applies the curve
and the display encoding.
Nothing thresholds or blends encoded values.

## Pipelines

A pipeline is **data**: `PassDesc`s over `ResourceDesc`s, held in a
`FrameGraph` (`wxsl_frame`), which `wxsl_render`'s `RenderGraph` wraps
to record it on wgpu.
It is usually authored as a document over the pipeline vocabulary
(`wxsl_core::pipeline`), which contains:

- sources;
- resources (`resource.color`, `resource.depth`, `resource.buffer`, …);
- passes (`pass.geometry`, `pass.shadow`, `pass.screen`, `pass.compute.*`,
  …);
- `output.present`.

The stock forward and deferred pipelines are preset documents in
`crates/wxsl-frame/assets/presets/`. The hand-built
`forward_graph`/`deferred_graph` exist only as the reference the parity
tests compare against.

### Effects

An **effect** is a declared unit. It has:

- inputs and outputs, in binding order;
- a screen or compute entry point;
- a shader (a `.wxsl` file, generated, or a screen graph);
- **parameters** declared as `{name, default}`. Their uniform layout is
  computed, and the host tunes them through `Renderer::set_pass_param`
  without recompiling.

A pass names an effect by id. A compute effect, or a screen effect with
unusual wiring, gets a derived `pass.compute.<id>` or `pass.screen.<id>`
node whose sockets are its declaration.
Applications register more effects with `Renderer::add_effect`. A
multi-pass pyramid such as bloom or depth of field is a single document
node that expands with scoped names.

A particle system combines a compute simulation over stable storage with an
additive sprites effect that reads opaque depth. The host supplies emitter
parameters and time through the effect API; both effects require the native
profile. See [GPU particles](adr/0024-gpu-particles.md).

### Scheduling

`FrameGraph::schedule` is pure and tested without a device. It:

- orders passes by what they read and write, with declaration order as the
  tie-break;
- validates everything it can check: attachment counts, formats, cycles,
  resources nothing writes;
- assigns physical textures.

Resource properties:

- **Persistence:** `Transient` textures are reused once their last reader
  has run. `Persistent { history: n }` keeps a ring, so a pass can read
  earlier frames without creating a cycle.
- **Shape:** textures may be 2D, arrays, cubes or 3D, with mips.
- **Buffers** are resources too, and never alias.

A pass has a **policy**: per frame, once, on resize or on demand. A
non-default policy may only write stable storage.

### Patterns built on these

- **Bakes.** A material subgraph is turned into a compute effect that
  writes a host-owned table. The material then samples that table.
- **Motion.** The velocity stage reads this frame's and the previous
  frame's transforms. TAA and motion blur are history chains over it.
- **Transparency.**
  - It is tiered: sorted by default, with dual depth peeling up to a
    material's `max_layers`.
  - Draw order is data, per pass.
  - Refraction composes peeled optical layers.

### Planar reflections

`wxsl_core::reflection::PlanarReflection` validates a mirror plane without
a GPU or maths dependency. `Environment::planar_reflection` supplies its
frame-local position; `PassView::Reflection` mirrors the camera and clips
against the plane. `effect::planar::append` captures selected geometry and
composes it onto the depth-masked receiver, with Fresnel and roughness.
It uses ordinary raster passes, including on the web.

### Switching pipeline

- `Renderer::set_pipeline` switches immediately and compiles during the
  next frame.
- `Renderer::request_pipeline` compiles the missing variants on a worker
  thread, keeps presenting the current pipeline, and then swaps.
  `swap_progress` reports how far it has got.

## A frame

The application gives the renderer three things:

- **an `Environment`:** the camera, lights, exposure and time. Positions
  are measured from an `f64` floating origin;
- **a `DrawList`:** each item is a mesh, a material, a transform, tags,
  bindings and optionally a world box. A geometry pass draws a tag
  expression (`opaque && !outlined`);
- **a target view.**

The renderer never holds a scene. Converting a scene document into a draw
list is `wxsl_render::scene::SceneResources`'s job, a loader beside the
renderer that the facade re-exports as `wxsl::scene`.
Large scenes split the work:
the scene orders its draws by material and mesh and gives each a box; the
renderer draws adjacent identical draws as one instanced call, and skips a
draw whose box is outside a pass's view (the camera's or a shadow tile's).

**Bind groups** are allocated by update frequency:

| # | Group | Holds |
|---|---|---|
| 0 | frame | camera, lights, shadow atlas, light grid, instance rows |
| 1 | material | the graph's declared parameters and textures |
| 2 | user | the application's block, if the material declares one |
| 3 | pass | what the pass reads, then its effect parameters |

`abi::BIND_GROUPS` is the single declaration of these groups. The fixed
host layouts (scene uniform, lights, vertices) are generated as Rust, C and
WXSL from `wxsl_core::host`.

## Backends and profiles

- **wgpu** (`wxsl-render`) allocates pools, records passes and caches
  variants. The neutral ↔ wgpu type mapping is exhaustive (`types`).
- **Dawn** (`dawn/`) implements only the device half in C++. It
  consumes plans, WGSL and layouts exported by `wxsl-ffi` (JSON in, data
  out), or
  loads that ABI at runtime to compile.
  The C++ never reimplements scheduling, compilation or layouts. Parity
  scripts compare its images with wgpu's.
- **The profile is decided by the device, not by a feature**. Without
  compute or storage buffers (WebGL2, GLES):
  - instance rows are read from data textures;
  - compute effects that have a fragment form run as screen passes;
  - native-only effects are refused by name.

The release gate compiles and validates every shipped shader through the C
ABI under every profile.

## The editor

The editor draws itself with the renderer: its chrome is a WXSL shader, its
text is MSDF generated in-tree, and its preview is the real `Renderer` on
the real pipelines.

It is windowing-agnostic: it takes `UiEvent`s and records into an encoder.
`examples/editor.rs` is the winit glue.

It has three canvas modes over one renderer:

- **the material graph;**
- **the pipeline document,** compiled live on every edit;
- **the scene:** a `.scene.json` or an imported glTF file in a viewport,
  with an outliner and an inspector. A click picks an instance on the CPU
  and opens its material in the material canvas, whose edits recompile
  that material in the scene.

It also offers pass stepping, resource inspection and shader hot reload.

Every change to a document is a serializable `Command`, applied by
`Editor::apply`; the UI's gestures emit them too. Undo restores a snapshot
taken before each command, and `Query` is the read-only side.

`wxsl-mcp` serves those commands and queries to agents over the Model
Context Protocol, on local HTTP beside the window or on stdio with no
window. Its transport threads plan each call and hand it to the render
thread, which the host drains once per frame with `Server::serve`. An
agent's steps are marked in the history, and its paths stay under a
project root. An
agent learns the vocabulary from two resources that cannot drift: a node
reference generated from the registries (also `docs/reference/nodes.md`)
and an authoring guide whose snippets are run as tests. A function it
writes is validated by the device when saved, and an invalid shader is a
reported error, never a panic.

Debugging a scene uses the same queries.
- **Surface views** (albedo, normal, uv, mip level, instance and
  material ids, ...) replace the shading function in a forward stage, and
  the lighting pass's output in deferred.
- **Frame views** (light count per cluster, cascades, shadow atlas) are
  editor effects appended to the running pass list. Wireframe is the
  renderer's line rasterization of the camera's passes; overdraw rewrites
  the forward passes to add a constant without a depth test.
- **The pixel probe** reads every texture layer and buffer a frame
  writes, right after the pass that writes it, so it works on any pass
  list.
- **The frame panel** lists each pass's time, draws and triangles.

## WXSL and the compiler

WGSL has no imports, no conditional compilation and no generics; the node
library needs all three. WXSL adds templates, imports, `@if` and
`@macro const`.

`wxsl-lang` runs these passes, in this order:

| Pass | Job |
|---|---|
| `lexer` | tokens, plus template disambiguation |
| `grammar` | LALRPOP, over the token stream |
| `cond` | evaluates `@if` and binds macros, per module, before renaming |
| `resolve` | inlines imports and mangles names by origin |
| `mono` | instantiates templates on the flat module |
| dead-code elimination | keeps only what the root reaches |
| `emit` | WGSL; refuses anything that is still WXSL |

The order is an invariant, and `resolve` enforces it. `node` sits off this
pipeline: it parses one file and returns the `NodeDefinition` that file
describes.

`wxsl-lang` does not validate its WGSL output. The device, naga in tests,
or the release gate does.

## The node library

`wxsl-stdlib/shaders/<category>/` holds one `.wxsl` file per node, and **the
file is the node**:

- the signature gives the sockets;
- the type-parameter bounds give the generic types;
- the comments give the label, the doc and the defaults.

Arithmetic operators are inline expressions, one generic node per
operation.
`shaders/wxsl/` holds the shader ABI rather
than nodes.

The code is original or ported from audited permissive sources.
The authoring rules are in `crates/wxsl-stdlib/shaders/README.md`.

## See also

- [`glossary.md`](glossary.md): the terms used here.
- [`adr/README.md`](adr/README.md): the decision log.
- [`../AGENTS.md`](../AGENTS.md): commands, tests, conventions.
