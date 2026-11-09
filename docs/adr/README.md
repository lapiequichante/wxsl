# Architecture decisions

These records describe the architectural decisions of the WXSL API.
Application-specific decisions belong with their applications. For the complete
current design, see [Architecture](../architecture.md).

| Decision | Status |
|---|---|
| [Separate the API and applications](0001-separate-api-and-applications.md) | Accepted |
| [Workspace boundaries and facade features](0002-workspace-boundaries.md) | Accepted |
| [Own the shading language and compiler](0003-shading-language.md) | Accepted |
| [Derive node definitions from WXSL](0004-node-definitions.md) | Accepted |
| [Compile a material graph for each stage](0005-material-stages.md) | Accepted |
| [Resolve material configuration once](0006-material-configuration.md) | Accepted |
| [Pipelines and scheduling are data](0007-pipeline-documents.md) | Accepted |
| [Effects and bakes are declared units](0008-effects-and-bakes.md) | Accepted |
| [Share lighting between rendering paths](0009-lighting.md) | Accepted |
| [Share host layouts and bind group conventions](0010-host-layout-and-binding.md) | Accepted |
| [Share device-free functionality through a data-only C ABI](0011-data-only-c-abi.md) | Accepted |
| [Keep provenance for permissive shader ports](0012-shader-provenance.md) | Accepted |
| [Select rendering capabilities from the device](0013-fallback-profile.md) | Accepted |
| [Define and check API stability](0014-api-stability.md) | Accepted |
| [Import scenes into a device-free document](0015-scene-import.md) | Accepted |
| [Use one editor and one mutation path](0016-editor-commands.md) | Accepted |
| [Expose editor commands and diagnostics to agents](0017-mcp-and-diagnostics.md) | Accepted |
| [Use tiered transparency and optical layers](0018-transparency.md) | Accepted |
| [Represent lights and shadows as frame data](0019-shadows-and-lights.md) | Accepted |
| [Represent environment lighting as frame resources](0020-environment-resources.md) | Accepted |
| [Validate generated shaders before release](0021-shader-validation.md) | Accepted |
| [Bake precomputed radiance transfer offline](0022-offline-prt.md) | Accepted |
| [Render planar reflections with a mirrored view](0023-planar-reflections.md) | Accepted |
| [Simulate and draw GPU particles through effects](0024-gpu-particles.md) | Accepted |
| [Select the environment BRDF LUT](0025-selectable-environment-brdf.md) | Accepted |

Record new architectural decisions as short files in this directory and link them here.
