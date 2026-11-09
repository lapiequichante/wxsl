//! Environment BRDF selection: independent hemisphere integration, runtime
//! switching and equality of compute and fragment bakes for every model.
use glam::Vec3;
use wxsl::render::effect::{ibl::BrdfLut, Downlevel, Effect, EffectKind, EffectShader};
use wxsl::render::{
    Attachment, DrawList, GpuContext, OffscreenTarget, PassDesc, RenderGraph, Renderer,
    TargetConfig,
};
mod probe;
const SIZE: u32 = 64;
const VIEW: Effect = Effect {
    id: "test.brdf_view",
    label: "BRDF view",
    description: "Inspect the renderer's BRDF table",
    kind: EffectKind::Screen {
        vertex_entry: "vs",
        fragment_entry: "fs",
    },
    inputs: &[],
    outputs: &[],
    parameters: &[],
    downlevel: Downlevel::Portable,
    shader: EffectShader::Source {
        path: "package::test::brdf_view",
        wxsl: r#"
import package::wxsl::bindings::environment_brdf;
@vertex fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
    return vec4f(f32(i32(index & 1u) * 4 - 1), f32(i32(index >> 1u) * 4 - 1), 0.0, 1.0);
}
@fragment fn fs(@builtin(position) at: vec4f) -> @location(0) vec4f {
    return vec4f(environment_brdf(at.x / 64.0, at.y / 64.0), 0.0, 1.0);
}
"#,
    },
};

// Uniform solid-angle quadrature over light directions, independent of the
// bake's GGX half-vector importance sampling and its Jacobian.
fn reference(model: BrdfLut, nv: f32, roughness: f32) -> [f32; 2] {
    let v = Vec3::new((1.0 - nv * nv).sqrt(), 0.0, nv);
    let alpha = (roughness * roughness).max(0.001);
    let a2 = alpha * alpha;
    let mut sum = [0.0; 2];
    for z in 0..256 {
        let nl = (z as f32 + 0.5) / 256.0;
        for azimuth in 0..512 {
            let phi = std::f32::consts::TAU * (azimuth as f32 + 0.5) / 512.0;
            let radius = (1.0 - nl * nl).sqrt();
            let h = (v + Vec3::new(radius * phi.cos(), radius * phi.sin(), nl)).normalize();
            let distribution = a2 / (std::f32::consts::PI * (1.0 + (a2 - 1.0) * h.z * h.z).powi(2));
            let root = |n: f32| (a2 + (1.0 - a2) * n * n).sqrt();
            let geometry = match model {
                BrdfLut::Ue4 => {
                    let k = alpha * 0.5;
                    nv * nl / ((nv * (1.0 - k) + k) * (nl * (1.0 - k) + k))
                }
                BrdfLut::Disney => 4.0 * nv * nl / ((nv + root(nv)) * (nl + root(nl))),
                BrdfLut::Filament => 2.0 * nv * nl / (nl * root(nv) + nv * root(nl)),
            };
            let fresnel = (1.0 - v.dot(h)).powi(5);
            let weight = distribution * geometry / (4.0 * nv);
            sum[0] += (1.0 - fresnel) * weight;
            sum[1] += fresnel * weight;
        }
    }
    sum.map(|s| s * std::f32::consts::TAU / (256.0 * 512.0))
}

fn images(gpu: &GpuContext) -> Vec<Vec<u8>> {
    let target = OffscreenTarget::new(&gpu.device, SIZE, SIZE);
    let mut renderer = Renderer::new(
        &gpu.device,
        wxsl::stdlib_library(),
        TargetConfig::new(SIZE, SIZE, target.format()),
    )
    .unwrap();
    renderer.add_effect(VIEW);
    let mut graph = RenderGraph::new(target.format());
    graph.pass(
        PassDesc::screen("view", VIEW.id).with_color(Attachment::clear(
            RenderGraph::TARGET,
            wxsl::render::types::Color::BLACK,
        )),
    );
    renderer.set_graph(graph).unwrap();
    let draws = DrawList::new();
    let mut results = Vec::new();
    for model in [
        BrdfLut::Ue4,
        BrdfLut::Disney,
        BrdfLut::Filament,
        BrdfLut::Ue4,
    ] {
        renderer.set_brdf_lut(model);
        let image = probe::render_list(gpu, &mut renderer, &target, &draws).unwrap();
        for x in [8u32, 32, 48] {
            let y = 48;
            let expected = reference(
                model,
                (x as f32 + 0.5) / SIZE as f32,
                (y as f32 + 0.5) / SIZE as f32,
            );
            let offset = ((y * SIZE + x) * 4) as usize;
            for channel in 0..2 {
                let actual = image[offset + channel] as f32 / 255.0;
                assert!(
                    (actual - expected[channel]).abs() < 0.018,
                    "{model:?} at {x}: got {actual}, expected {}",
                    expected[channel]
                );
            }
        }
        let repeated = probe::render_list(gpu, &mut renderer, &target, &draws).unwrap();
        assert_eq!(image, repeated, "same model must retain its LUT");
        results.push(image);
    }
    assert_ne!(results[0], results[1], "UE4 and Disney must differ");
    assert_ne!(
        results[1], results[2],
        "correlation must affect the integral"
    );
    assert_eq!(results[0], results[3], "switching back must restore UE4");
    results
}

#[test]
fn every_model_matches_quadrature_and_switches_on_both_bake_paths() {
    let Some(gpu) = probe::gpu() else { return };
    let native = images(&gpu);
    let Some(fallback) = probe::webgl2_gpu() else {
        return;
    };
    let fragment = images(&fallback);
    for (a, b) in native.iter().zip(&fragment) {
        assert!(
            a.iter().zip(b).all(|(x, y)| x.abs_diff(*y) <= 1),
            "compute and fragment disagree"
        );
    }
}
