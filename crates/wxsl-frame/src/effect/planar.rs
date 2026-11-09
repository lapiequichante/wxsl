//! Raster planar mirrors: a clipped camera capture and depth-masked composition.
use super::{
    Downlevel, Effect, EffectInput, EffectInputKind, EffectKind, EffectParameter, EffectShader,
};
use crate::{
    graph::FrameGraph,
    pass::{
        Attachment, DepthAttachment, DrawSource, Extent, PassDesc, PassState, PassView, Read,
        ResourceDesc, ResourceId, DEPTH_FORMAT,
    },
    types::{Color, TextureFormat},
};
use wxsl_core::{
    abi::{GBufferTarget, MaterialStage},
    reflection::PlanarReflection,
    scene::TagExpr,
    Value,
};

const INPUTS: [EffectInput; 5] = [
    EffectInput {
        name: "color",
        kind: EffectInputKind::Image,
        description: "Linear main-camera radiance",
        history: 0,
    },
    EffectInput {
        name: "depth",
        kind: EffectInputKind::DepthImage,
        description: "Main-camera depth",
        history: 0,
    },
    EffectInput {
        name: "reflection",
        kind: EffectInputKind::Image,
        description: "Clipped, X-reversed mirror radiance",
        history: 0,
    },
    EffectInput {
        name: "reflection_mid",
        kind: EffectInputKind::Image,
        description: "Filtered mirror radiance at one eighth viewport resolution",
        history: 0,
    },
    EffectInput {
        name: "reflection_rough",
        kind: EffectInputKind::Image,
        description: "Filtered mirror radiance at one thirty-second viewport resolution",
        history: 0,
    },
];

const PARAMETERS: &[EffectParameter] = &[
    EffectParameter {
        name: "plane",
        default: Value::Vec4([0.0, 1.0, 0.0, 0.0]),
    },
    EffectParameter {
        name: "strength",
        default: Value::F32(1.0),
    },
    EffectParameter {
        name: "reflectance",
        default: Value::F32(0.04),
    },
    EffectParameter {
        name: "roughness",
        default: Value::F32(0.1),
    },
    EffectParameter {
        name: "tolerance",
        default: Value::F32(0.025),
    },
    // How far a bumped receiver's rays reach what they reflect, in world units.
    EffectParameter {
        name: "distance",
        default: Value::F32(1.0),
    },
];

/// Compose a planar capture onto pixels on its plane, using dielectric Fresnel.
pub const COMPOSITE: Effect = Effect {
    id: "wxsl.planar_reflection",
    label: "Planar reflection",
    description: "A raster mirror capture, clipped to a world plane and weighted by Fresnel.",
    kind: EffectKind::Screen {
        vertex_entry: "vertex",
        fragment_entry: "fragment",
    },
    inputs: &INPUTS,
    outputs: &[],
    parameters: PARAMETERS,
    shader: EffectShader::Source {
        path: "package::wxsl::planar_reflection",
        wxsl: concat!(
            include_str!("../../shaders/planar_reflection_common.wxsl"),
            include_str!("../../shaders/planar_reflection.wxsl")
        ),
    },
    downlevel: Downlevel::Portable,
};

/// [`COMPOSITE`] on its receivers' pixels only, each by its mapped normal
/// ([`map_receivers`]).
pub const COMPOSITE_MAPPED: Effect = Effect {
    id: "wxsl.planar_reflection_mapped",
    label: "Planar reflection, mapped",
    description: "A raster mirror capture on its receivers, bent by their mapped normals.",
    kind: EffectKind::Screen {
        vertex_entry: "vertex",
        fragment_entry: "fragment_mapped",
    },
    inputs: &[
        INPUTS[0],
        INPUTS[1],
        INPUTS[2],
        INPUTS[3],
        INPUTS[4],
        EffectInput {
            name: "receiver_normal",
            kind: EffectInputKind::Image,
            description: "The receivers' G-buffer normal, z below zero elsewhere",
            history: 0,
        },
    ],
    outputs: &[],
    parameters: PARAMETERS,
    shader: EffectShader::Source {
        path: "package::wxsl::planar_reflection_mapped",
        wxsl: concat!(
            include_str!("../../shaders/planar_reflection_common.wxsl"),
            include_str!("../../shaders/planar_reflection_mapped.wxsl")
        ),
    },
    downlevel: Downlevel::Portable,
};

/// Append a half-resolution forward mirror and composite it over `color`.
///
/// Set `Environment::planar_reflections[mirror]` to the same plane, below
/// `PassView::MIRRORS`. `draws` must
/// exclude the receiver, recursive mirrors and screen-only overlay meshes.
/// The capture shades opaque forward materials; optical peels are not recursed.
/// `radiance` is the environment cube used behind reflected geometry.
#[allow(clippy::too_many_arguments)]
pub fn append(
    graph: &mut FrameGraph,
    color: ResourceId,
    depth: ResourceId,
    radiance: ResourceId,
    draws: TagExpr,
    plane: PlanarReflection,
    mirror: u32,
    label: &str,
) -> ResourceId {
    let view = PassView::Reflection { mirror };
    let extent = Extent::Viewport { scale: 0.5 };
    let capture = graph.resource(
        ResourceDesc::color(format!("{label} capture"), TextureFormat::Rgba16Float)
            .with_extent(extent),
    );
    let mirror_depth = graph
        .resource(ResourceDesc::color(format!("{label} depth"), DEPTH_FORMAT).with_extent(extent));
    graph.pass(
        PassDesc::geometry(
            format!("{label} capture"),
            DrawSource::Scene(draws),
            MaterialStage::FORWARD_LIT,
        )
        .with_view(view)
        .with_color(Attachment::clear(capture, Color::TRANSPARENT))
        .with_depth(DepthAttachment::clear(mirror_depth, 1.0))
        .with_state(PassState::OPAQUE),
    );
    let background = graph.resource(
        ResourceDesc::color(format!("{label} background"), TextureFormat::Rgba16Float)
            .with_extent(extent),
    );
    graph.pass(
        PassDesc::screen(format!("{label} background"), super::ibl::BACKGROUND.id)
            .with_view(view)
            .with_color(Attachment::clear(background, Color::TRANSPARENT))
            .with_reads([
                Read::current(capture),
                Read::current(mirror_depth),
                Read::current(radiance),
            ]),
    );
    // Filter adjacent texels before widening the footprint; sparse large offsets duplicate emitters.
    let mut filtered = background;
    let mut mid = background;
    for level in 1..=4 {
        let next = graph.resource(
            ResourceDesc::color(
                format!("{label} filter {level}"),
                TextureFormat::Rgba16Float,
            )
            .with_extent(Extent::Viewport {
                scale: 0.5 / (1u32 << level) as f32,
            }),
        );
        graph.pass(
            PassDesc::screen(format!("{label} filter {level}"), super::BLOOM_DOWN.id)
                .with_view(view)
                .with_color(Attachment::clear(next, Color::TRANSPARENT))
                .with_reads([Read::current(filtered)]),
        );
        filtered = next;
        if level == 2 {
            mid = next;
        }
    }
    let output = graph.resource(ResourceDesc::color(label, TextureFormat::Rgba16Float));
    graph.pass(
        PassDesc::screen(label, COMPOSITE.id)
            .with_color(Attachment::clear(output, Color::TRANSPARENT))
            .with_reads([
                Read::current(color),
                Read::current(depth),
                Read::current(background),
                Read::current(mid),
                Read::current(filtered),
            ])
            .with_parameter("plane", Value::Vec4(plane.equation())),
    );
    output
}

/// Limit the mirror composited by pass `composite` (from [`append`]) to the
/// `receivers`' pixels, and bend each lookup by their mapped normal.
///
/// A pass draws the receivers' `GBUFFER` stage over the composite's depth,
/// without writing it; `layout` is the G-buffer layout their materials pack,
/// the graph's own if it has one.
/// Geometry on the plane that is not a receiver is left unreflected.
pub fn map_receivers(
    graph: &mut FrameGraph,
    composite: usize,
    receivers: TagExpr,
    layout: &[GBufferTarget],
) -> Result<(), String> {
    let pass = graph
        .passes()
        .get(composite)
        .ok_or("no such mirror composite")?;
    if !matches!(&pass.kind, crate::pass::PassKind::Screen { effect } if effect == COMPOSITE.id) {
        return Err(format!("{} is not a planar composite", pass.label));
    }
    let (label, depth) = (pass.label.clone(), pass.reads[1].resource);
    if graph.gbuffer_layout().is_empty() {
        graph.set_gbuffer_layout(layout.to_vec());
    }
    let layout = graph.gbuffer_layout().to_vec();
    let normal = layout
        .iter()
        .position(|target| target.field == "normal")
        .ok_or("the G-buffer has no normal target")?;
    let targets: Vec<ResourceId> = layout
        .iter()
        .map(|target| {
            graph.resource(ResourceDesc::color(
                format!("{label} receivers {}", target.field),
                crate::pipeline::gbuffer_format(target.precision),
            ))
        })
        .collect();
    // Roughness is never negative: z = -1 marks the pixels no receiver covers.
    let unmarked = Color {
        r: 0.0,
        g: 0.0,
        b: -1.0,
        a: 0.0,
    };
    graph.pass(
        PassDesc::geometry(
            format!("{label} receivers"),
            DrawSource::Scene(receivers),
            MaterialStage::GBUFFER,
        )
        .with_colors(targets.iter().enumerate().map(|(index, &target)| {
            Attachment::clear(
                target,
                if index == normal {
                    unmarked
                } else {
                    Color::TRANSPARENT
                },
            )
        }))
        .with_depth(DepthAttachment::load(depth))
        .with_state(
            PassState::OPAQUE.with_depth_test(crate::types::CompareFunction::LessEqual, false),
        ),
    );
    let pass = graph
        .pass_mut(composite)
        .ok_or("no such mirror composite")?;
    pass.kind = crate::pass::PassKind::Screen {
        effect: COMPOSITE_MAPPED.id.to_string(),
    };
    pass.reads.push(Read::current(targets[normal]));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pass::{Dimension, PassKind};

    #[test]
    fn mapped_receivers_draw_their_normals_over_the_main_depth_before_the_composite() {
        let mut graph = FrameGraph::new(TextureFormat::Rgba16Float);
        let color = graph.resource(ResourceDesc::color("scene", TextureFormat::Rgba16Float));
        let depth = graph.resource(ResourceDesc::color("depth", DEPTH_FORMAT));
        graph.pass(
            PassDesc::geometry(
                "main",
                DrawSource::Scene(TagExpr::Always),
                MaterialStage::FORWARD_LIT,
            )
            .with_color(Attachment::clear(color, Color::BLACK))
            .with_depth(DepthAttachment::clear(depth, 1.0))
            .with_state(PassState::OPAQUE),
        );
        let sky = graph.resource(
            ResourceDesc::imported("sky", TextureFormat::Rgba16Float)
                .with_dimension(Dimension::Cube, 6)
                .with_extent(Extent::Fixed {
                    width: 1,
                    height: 1,
                }),
        );
        let plane = PlanarReflection::new([-1.0, 0.0, 0.0], -3.0).unwrap();
        let output = append(
            &mut graph,
            color,
            depth,
            sky,
            TagExpr::Always,
            plane,
            1,
            "panes",
        );
        let composite = graph.passes().len() - 1;
        assert!(map_receivers(&mut graph, 0, TagExpr::Always, &[]).is_err());
        map_receivers(
            &mut graph,
            composite,
            TagExpr::tag("pane"),
            wxsl_core::abi::GBUFFER_BASE_TARGETS,
        )
        .unwrap();
        assert!(graph.passes()[..composite]
            .iter()
            .all(|pass| pass.view != PassView::Camera || pass.label == "main"));
        let receivers = graph.passes().last().unwrap();
        assert!(matches!(&receivers.kind,
            PassKind::Geometry { source: DrawSource::Scene(tags), stage } if *stage == MaterialStage::GBUFFER && *tags == TagExpr::tag("pane")));
        assert!(!receivers.written().any(|written| written == depth));
        let normal = receivers.color[1].resource;
        let mapped = &graph.passes()[composite];
        assert!(
            matches!(&mapped.kind, PassKind::Screen { effect } if effect == COMPOSITE_MAPPED.id)
        );
        assert_eq!(mapped.reads.last().unwrap().resource, normal);
        assert_eq!(mapped.color[0].resource, output);
        // Declared last, the receivers still run before the composite that reads them.
        let schedule = graph.schedule().unwrap();
        let at = |index| schedule.order().iter().position(|&pass| pass == index);
        assert!(at(graph.passes().len() - 1) < at(composite));
        assert!(at(0) < at(graph.passes().len() - 1));
    }
}
