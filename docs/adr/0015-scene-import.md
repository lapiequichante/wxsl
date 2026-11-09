# 0015. Import scenes into a device-free document

Status: Accepted

## Context

glTF scenes need to be shared by the editor, renderer and C ABI without coupling the importer to wgpu.

## Decision

Import geometry and materials into scene documents in `wxsl-gltf`, using a registry supplied by the caller. Keep scene resources beside the renderer; convert scenes to draw lists, with instancing and per-view culling where applicable.

## Consequences

Import is reusable without the renderer; callers supply compatible node definitions.
