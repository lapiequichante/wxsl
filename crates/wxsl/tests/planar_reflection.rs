//! Raster mirrors see off-screen geometry, clip below-plane geometry and preserve winding.
mod probe;
use glam::{Mat4, Vec3};
use wxsl::{
    core::{
        abi,
        reflection::PlanarReflection,
        scene::{TagExpr, Tags},
        Graph, Value,
    },
    render::{
        effect::planar,
        pass::{
            Attachment, DepthAttachment, Dimension, DrawSource, Extent, PassDesc, PassState, Read,
            ResourceDesc, DEPTH_FORMAT,
        },
        types::{Color, TextureFormat},
        Camera, DrawItem, Mesh,
    },
};

#[test]
fn planar_composite_compiles_without_a_device() {
    effect_request();
}
fn effect_request() {
    for effect in [planar::COMPOSITE, planar::COMPOSITE_MAPPED] {
        wxsl::render::variants::EffectRequest::new(
            effect,
            &Default::default(),
            &Default::default(),
            &[],
        )
        .compile(&wxsl::stdlib_library())
        .1
        .unwrap();
    }
}

#[test]
fn a_floor_mirror_sees_a_cube_above_the_main_image_and_clips_one_below_the_floor() {
    let Some(gpu) = probe::gpu() else { return };
    let mut harness = probe::Harness::new(gpu);
    let mut graph = wxsl::render::RenderGraph::new(harness.target.format());
    let color = graph.resource(ResourceDesc::color("scene", TextureFormat::Rgba16Float));
    let depth = graph.resource(ResourceDesc::color("depth", DEPTH_FORMAT));
    graph.pass(
        PassDesc::geometry(
            "main",
            DrawSource::Scene(TagExpr::Always),
            abi::MaterialStage::FORWARD_LIT,
        )
        .with_color(Attachment::clear(color, Color::BLACK))
        .with_depth(DepthAttachment::clear(depth, 1.0))
        .with_state(PassState::OPAQUE),
    );
    let sky = graph.resource(
        ResourceDesc::imported("black sky", TextureFormat::Rgba16Float)
            .with_dimension(Dimension::Cube, 6)
            .with_extent(Extent::Fixed {
                width: 1,
                height: 1,
            }),
    );
    let plane = PlanarReflection::new([0.0, 1.0, 0.0], 0.0).unwrap();
    let output = planar::append(
        &mut graph,
        color,
        depth,
        sky,
        TagExpr::tag("reflected"),
        plane,
        0,
        "mirror",
    );
    let mirror = graph.passes().len() - 1;
    graph
        .pass_mut(mirror)
        .unwrap()
        .parameters
        .insert("reflectance".into(), Value::F32(1.0));
    graph.pass(
        PassDesc::screen("present", "wxsl.tonemap")
            .with_reads([Read::current(output)])
            .with_color(Attachment::clear(
                wxsl::render::RenderGraph::TARGET,
                Color::BLACK,
            )),
    );
    harness.renderer.set_graph(graph).unwrap();
    let texture = harness
        .gpu
        .device
        .create_texture(&wxsl::render::wgpu::TextureDescriptor {
            label: Some("black sky"),
            size: wxsl::render::wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wxsl::render::wgpu::TextureDimension::D2,
            format: wxsl::render::wgpu::TextureFormat::Rgba16Float,
            usage: wxsl::render::wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
    harness.renderer.import_resource(
        "black sky",
        texture.create_view(&wxsl::render::wgpu::TextureViewDescriptor {
            dimension: Some(wxsl::render::wgpu::TextureViewDimension::Cube),
            ..Default::default()
        }),
    );
    let material = |emission| {
        let mut graph = Graph::new("emissive");
        let surface = graph.add_node(abi::SURFACE_OUTPUT_ID);
        graph.set_param(surface, "emissive", Value::Vec3(emission));
        harness.material(&graph)
    };
    let black = material([0.0; 3]);
    let red = material([1.0, 0.0, 0.0]);
    let green = material([0.0, 1.0, 0.0]);
    let blue = material([0.0, 0.0, 1.0]);
    let floor = Mesh::plane(&harness.gpu.device, 20.0);
    let cube = Mesh::cube(&harness.gpu.device, 0.6);
    let tags = Tags::from_iter(["reflected"]);
    let draws = [
        DrawItem::new(&floor, &black),
        DrawItem::new(&cube, &blue)
            .with_transform(Mat4::from_translation(Vec3::new(0.0, 0.5, 1.0)))
            .with_tags(&tags),
        DrawItem::new(&cube, &red)
            .with_transform(Mat4::from_translation(Vec3::new(0.0, 3.0, 0.0)))
            .with_tags(&tags),
        DrawItem::new(&cube, &green)
            .with_transform(Mat4::from_translation(Vec3::new(0.0, -0.8, 2.0)))
            .with_tags(&tags),
    ]
    .into_iter()
    .collect();
    let camera = Camera {
        eye: Vec3::new(0.0, 3.0, 5.0),
        target: Vec3::ZERO,
        ..Default::default()
    };
    let clip = camera.view_proj() * Vec3::new(0.0, 2.7, 0.0).extend(1.0);
    assert!(clip.y > clip.w, "cube must be outside the main image");
    let mut environment = probe::unlit();
    environment.camera = camera;
    environment.planar_reflections = vec![plane];
    harness
        .renderer
        .set_pass_param("mirror", "strength", Value::F32(0.0))
        .unwrap();
    let baseline = probe::render_list_in(
        &harness.gpu,
        &mut harness.renderer,
        &harness.target,
        &draws,
        &environment,
    )
    .unwrap();
    harness
        .renderer
        .set_pass_param("mirror", "strength", Value::F32(1.0))
        .unwrap();
    let image = probe::render_list_in(
        &harness.gpu,
        &mut harness.renderer,
        &harness.target,
        &draws,
        &environment,
    )
    .unwrap();
    let foreground: Vec<_> = baseline
        .chunks_exact(4)
        .zip(image.chunks_exact(4))
        .filter(|(p, _)| p[2] > 40 && p[0] < 10 && p[1] < 10)
        .collect();
    assert!(
        foreground.len() > 3,
        "occluder must be visible in the main camera"
    );
    for (before, after) in foreground {
        assert_eq!(
            before, after,
            "floor reflection leaked over the foreground object"
        );
    }
    let red_pixels = image
        .chunks_exact(4)
        .filter(|p| p[0] > 40 && p[1] < 10)
        .count();
    let green_pixels = image.chunks_exact(4).filter(|p| p[1] > 40).count();
    assert!(
        red_pixels > 3,
        "missing off-screen cube reflection: {red_pixels} red pixels"
    );
    assert_eq!(
        green_pixels, 0,
        "geometry below the plane leaked into its reflection"
    );
}
