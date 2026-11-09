# 0025. Select the environment BRDF LUT independently of materials

Status: Accepted

## Context

Applications need to compare specular environment responses without changing their material graphs or frame resource layouts.

## Decision

Expose `effect::ibl::BrdfLut` with UE4 Schlick-GGX, Disney isotropic separable Smith-GGX, and Filament height-correlated Smith-GGX choices. All use GGX half-vector sampling, perceptual roughness squared, Schlick Fresnel and the same two-channel split-sum representation. These choices concern the specular LUT, not complete engine or Disney material models.

`Renderer::set_brdf_lut` selects the bake variant. A changed selection rebakes the existing texture before the next frame; setting the same choice leaves it cached. Compute and fragment bakes share the integral and macro. No host ABI layout changes are needed. Direct point-light shading and environment prefiltering remain independent. Existing consumers of the LUT, including area-light Fresnel weighting, see the selected response.

## Consequences

UE4 remains the default. Correct importance sampling requires `G * (V·H) / ((N·H) * (N·V))`; replacing the incorrect `N·L` numerator changes existing environment highlights. Selecting a model can incur one shader compilation and bake. Probes and cached mirrors in applications must refresh when their shading changes.

The implementation evaluates the published mathematical models independently. References: [Karis, Real Shading in UE4](https://cdn2.unrealengine.com/Resources/files/2013SiggraphPresentationsNotes-26915738.pdf), [Disney shading](https://www.disneyanimation.com/publications/physically-based-shading-at-disney/), and [Filament, specular visibility](https://google.github.io/filament/Filament.html).
