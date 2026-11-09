//! The particle system on a device: a stream simulated over
//! frames turns from its start colour to its end colour along its path,
//! a surface in front hides it, and with no rate it dies out.

use glam::{Mat4, Vec3};
use wxsl::core::{abi::MaterialStage, graph::Graph, node::Value};
use wxsl::render::effect::particles;
use wxsl::render::gpu::OffscreenTarget;
use wxsl::render::pass::DEPTH_FORMAT;
use wxsl::render::types::Color;
use wxsl::render::{
    Attachment, DepthAttachment, DrawItem, DrawList, DrawSource, Material, Mesh, PassDesc,
    RenderGraph, Renderer, ResourceDesc, TargetConfig,
};

mod probe;

/// Where the stream starts, and how fast it goes along +x.
const ORIGIN: Vec3 = Vec3::new(-0.8, 0.0, 0.0);
const SPEED: f32 = 1.2;
const LIFETIME: f32 = 1.2;
const STEP: f32 = 1.0 / 30.0;

fn set(renderer: &mut Renderer, label: &str, name: &str, value: Value) {
    renderer
        .set_pass_param(label, name, value)
        .unwrap_or_else(|error| panic!("{label} {name}: {error}"));
}

#[test]
fn a_stream_turns_from_its_start_colour_to_its_end_colour_and_dies_without_a_rate() {
    let Some(gpu) = probe::gpu() else { return };
    let size = probe::SIZE;
    let target = OffscreenTarget::new(&gpu.device, size, size);
    let mut renderer = Renderer::new(
        &gpu.device,
        wxsl::stdlib_library(),
        TargetConfig::new(size, size, target.format()),
    )
    .unwrap();
    let mut graph = RenderGraph::new(target.format());
    let depth = graph.resource(ResourceDesc::color("depth", DEPTH_FORMAT));
    graph.pass(
        PassDesc::geometry(
            "opaque",
            DrawSource::Scene(Default::default()),
            MaterialStage::FORWARD_LIT,
        )
        .with_color(Attachment::clear(RenderGraph::TARGET, Color::BLACK))
        .with_depth(DepthAttachment::clear(depth, 1.0)),
    );
    particles::append(&mut graph, RenderGraph::TARGET, depth, "stream");
    renderer.set_graph(graph).unwrap();

    let simulate = particles::simulate_label("stream");
    let draw = particles::draw_label("stream");
    set(
        &mut renderer,
        &simulate,
        "origin",
        Value::Vec4([ORIGIN.x, ORIGIN.y, ORIGIN.z, 0.0]),
    );
    set(
        &mut renderer,
        &simulate,
        "velocity",
        Value::Vec4([SPEED, 0.0, 0.0, 0.02]),
    );
    set(&mut renderer, &simulate, "rate", Value::F32(400.0));
    set(&mut renderer, &simulate, "lifetime", Value::F32(LIFETIME));
    set(
        &mut renderer,
        &draw,
        "start",
        Value::Vec4([0.0, 0.0, 0.16, 0.0]),
    );
    set(
        &mut renderer,
        &draw,
        "end",
        Value::Vec4([0.16, 0.04, 0.0, 0.0]),
    );
    set(&mut renderer, &draw, "turn", Value::F32(0.5));
    set(&mut renderer, &draw, "start_size", Value::F32(0.04));
    set(&mut renderer, &draw, "end_size", Value::F32(0.04));

    let environment = probe::unlit();
    let mut time = 0.0;
    let mut frame = |renderer: &mut Renderer, draws: &DrawList<'_>| {
        time += STEP;
        set(renderer, &simulate, "time", Value::F32(time));
        set(renderer, &simulate, "dt", Value::F32(STEP));
        probe::render_list_in(&gpu, renderer, &target, draws, &environment).unwrap()
    };
    let none = DrawList::new();
    let mut image = Vec::new();
    for _ in 0..45 {
        image = frame(&mut renderer, &none);
    }
    // Young near the origin, old half a lifetime further on.
    let young = probe::color_at(&image, environment.camera, ORIGIN + Vec3::X * 0.1);
    let old = probe::color_at(
        &image,
        environment.camera,
        ORIGIN + Vec3::X * SPEED * LIFETIME * 0.5,
    );
    assert!(young[2] > young[0] + 20.0, "blue at the origin: {young:?}");
    assert!(old[0] > old[2] + 20.0, "orange further on: {old:?}");
    // Nothing behind the emitter.
    let behind = probe::color_at(&image, environment.camera, ORIGIN - Vec3::X * 0.3);
    assert_eq!(behind, [0.0; 3]);

    // A surface in front of the stream hides it.
    let mut material_graph = Graph::new("black wall");
    material_graph.add_node("output.surface");
    let material = Material::from_graph(&material_graph, &wxsl::stdlib::registry()).unwrap();
    let mesh = Mesh::cube(&gpu.device, 1.0);
    let wall = wxsl::render::single_draw(DrawItem::new(&mesh, &material).with_transform(
        Mat4::from_translation(Vec3::new(ORIGIN.x, 0.0, 1.0)) * Mat4::from_scale(Vec3::splat(0.8)),
    ));
    let hidden = frame(&mut renderer, &wall);
    assert_eq!(
        probe::color_at(&hidden, environment.camera, ORIGIN + Vec3::X * 0.1),
        [0.0; 3],
        "behind the wall"
    );

    // With no rate, every particle dies within its lifetime and a third.
    set(&mut renderer, &simulate, "rate", Value::F32(0.0));
    for _ in 0..((LIFETIME * 1.4 / STEP) as usize) {
        image = frame(&mut renderer, &none);
    }
    assert!(image.chunks_exact(4).all(|pixel| pixel[..3] == [0; 3]));
}
