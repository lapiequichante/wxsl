//! Particles: a GPU particle system, as two effects over one stable
//! storage buffer.
//!
//! [`SIMULATE`] is a compute pass that advances every particle of the
//! buffer by the frame's step and respawns the slots whose turn has come at
//! the emitter; [`DRAW`] draws each live particle as a camera-facing
//! sprite stretched along its velocity, its radiance moving from a start
//! colour to an end colour over its life, added onto a frame and faded
//! where it nears the opaque depth. [`append`] adds both under one name.
//!
//! The buffer persists across frames and nothing else writes it, so the
//! particles keep where they were born when the emitter moves. Slots
//! respawn in turn, slot `i` at `i / rate` seconds past each period of
//! `PARTICLES / rate`: no counter, no atomics, and a frame's births are
//! spread over its step as a stream rather than bunched at its end. A
//! compute effect does not see the frame group, so the host
//! hands the simulation its clock (`time`, `dt`) as parameters, beside the
//! emitter, every frame.

use super::{
    Downlevel, Effect, EffectInput, EffectInputKind, EffectKind, EffectOutput, EffectOutputShape,
    EffectParameter, EffectShader,
};
use crate::graph::FrameGraph;
use crate::pass::{Attachment, PassDesc, PassState, Read, ResourceDesc, ResourceId, ADDITIVE};
use wxsl_core::node::Value;

/// How many particles one system holds: its buffer's slots and its
/// sprites' budget. At a `rate` of `r` a second, a particle can live at
/// most `PARTICLES / r` seconds before its slot is reborn.
pub const PARTICLES: u32 = 4096;

/// One particle's bytes: its position and age, its velocity and lifetime,
/// two `vec4f`s. Keep in sync with `particles_simulate.wxsl`.
pub const PARTICLE_BYTES: u64 = 32;

/// The simulation's invocations per workgroup. Keep in sync with
/// `particles_simulate.wxsl`.
const WORKGROUP: u32 = 64;

/// A parameter and its default.
const fn parameter(name: &'static str, default: Value) -> EffectParameter {
    EffectParameter { name, default }
}

/// Advance the particles by `dt` seconds to the host's `time` and respawn
/// the slots due: `rate` a second (0 for none) at `origin` (xyz, within
/// `w` metres of it), moving at `velocity` (xyz, metres a second) give or
/// take `w` of its speed in any direction, living `lifetime` seconds give
/// or take a third; slowed by `drag` (per second) and rising by `lift`
/// (metres a second squared, hot gas's buoyancy). Both random balls are
/// `widen.w` times as long along the unit axis `widen.xyz`. An emitter moving at
/// `carry` (xyz, metres a second) gives its particles its velocity and
/// spreads a step's births back along its path. Positions are the frame's,
/// the camera's space.
pub const SIMULATE: Effect = Effect {
    id: "wxsl.particles_simulate",
    label: "particles simulate",
    description:
        "Advance a particle buffer by the frame's step and respawn the slots due at the emitter.",
    kind: EffectKind::Compute {
        entry: "simulate",
        workgroups: [PARTICLES / WORKGROUP, 1, 1],
    },
    inputs: &[],
    outputs: &[EffectOutput {
        name: "particles",
        shape: EffectOutputShape::Buffer,
        description: "The particles: position and age, velocity and lifetime, two vec4f each.",
    }],
    parameters: &[
        parameter("origin", Value::Vec4([0.0, 0.0, 0.0, 0.0])),
        parameter("velocity", Value::Vec4([0.0, 1.0, 0.0, 0.2])),
        parameter("carry", Value::Vec4([0.0; 4])),
        parameter("widen", Value::Vec4([1.0, 0.0, 0.0, 1.0])),
        parameter("rate", Value::F32(0.0)),
        parameter("lifetime", Value::F32(1.0)),
        parameter("drag", Value::F32(0.0)),
        parameter("lift", Value::F32(0.0)),
        parameter("time", Value::F32(0.0)),
        parameter("dt", Value::F32(0.0)),
    ],
    shader: EffectShader::Source {
        path: "package::wxsl::particles_simulate",
        wxsl: include_str!("../../shaders/particles_simulate.wxsl"),
    },
    // Compute over a storage buffer.
    downlevel: Downlevel::NativeOnly,
};

/// The live particles as sprites `start_size` metres in radius at birth
/// and `end_size` at death, stretched along their velocity by how far
/// they go in `stretch` seconds; their radiance (linear rgb) moves from
/// `start` to `end` over the first `turn` of their life and fades out
/// with it, pre-exposed like any surface; faded where they come within
/// `softness` metres of the opaque depth.
pub const DRAW: Effect = Effect {
    id: "wxsl.particles_draw",
    label: "particles draw",
    description:
        "Add a particle buffer's live particles onto the frame as soft, depth-faded sprites.",
    kind: EffectKind::Sprites {
        vertex_entry: "vertex",
        fragment_entry: "fragment",
        vertices: 6,
        instances: PARTICLES,
    },
    inputs: &[
        EffectInput {
            name: "particles",
            kind: EffectInputKind::Buffer,
            description: "The particles the simulation wrote.",
            history: 0,
        },
        EffectInput {
            name: "depth",
            kind: EffectInputKind::DepthImage,
            description: "The opaque depth the sprites fade against.",
            history: 0,
        },
    ],
    outputs: &[],
    parameters: &[
        parameter("start", Value::Vec4([0.2, 0.4, 4.0, 0.0])),
        parameter("end", Value::Vec4([4.0, 1.2, 0.2, 0.0])),
        parameter("turn", Value::F32(0.3)),
        parameter("start_size", Value::F32(0.03)),
        parameter("end_size", Value::F32(0.08)),
        parameter("stretch", Value::F32(0.0)),
        parameter("softness", Value::F32(0.05)),
    ],
    shader: EffectShader::Source {
        path: "package::wxsl::particles_draw",
        wxsl: include_str!("../../shaders/particles_draw.wxsl"),
    },
    // Reads storage in the vertex stage.
    downlevel: Downlevel::NativeOnly,
};

/// The label of the pass simulating the system `name`, where the host
/// sets [`SIMULATE`]'s parameters.
pub fn simulate_label(name: &str) -> String {
    format!("{name} simulate")
}

/// The label of the pass drawing the system `name`, where the host sets
/// [`DRAW`]'s parameters.
pub fn draw_label(name: &str) -> String {
    format!("{name} draw")
}

/// A particle system named `name`, added onto `color` and faded against
/// `depth`: its buffer, stable storage, [`SIMULATE`]d under
/// [`simulate_label`] then [`DRAW`]n under [`draw_label`]. Nothing is
/// emitted until the host sets a `rate`. Returns the buffer.
pub fn append(
    graph: &mut FrameGraph,
    color: ResourceId,
    depth: ResourceId,
    name: &str,
) -> ResourceId {
    let particles = graph
        .resource(ResourceDesc::buffer(name, u64::from(PARTICLES) * PARTICLE_BYTES).persistent(0));
    graph.pass(PassDesc::compute(simulate_label(name), SIMULATE.id).with_write(particles));
    graph.pass(
        PassDesc::screen(draw_label(name), DRAW.id)
            .with_color(Attachment::load(color))
            .with_state(PassState::FULLSCREEN.with_blend(ADDITIVE))
            .with_reads([Read::current(particles), Read::current(depth)]),
    );
    particles
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pass::{DepthAttachment, DrawSource, DEPTH_FORMAT};
    use crate::types::{Color, TextureFormat};
    use wxsl_core::abi::MaterialStage;

    #[test]
    fn a_system_simulates_before_it_draws_into_stable_storage() {
        let mut graph = FrameGraph::new(TextureFormat::Rgba8UnormSrgb);
        let depth = graph.resource(ResourceDesc::color("depth", DEPTH_FORMAT));
        graph.pass(
            PassDesc::geometry(
                "opaque",
                DrawSource::Scene(Default::default()),
                MaterialStage::FORWARD_LIT,
            )
            .with_color(Attachment::clear(FrameGraph::TARGET, Color::BLACK))
            .with_depth(DepthAttachment::clear(depth, 1.0)),
        );
        let particles = append(&mut graph, FrameGraph::TARGET, depth, "flames");
        let schedule = graph.schedule().expect("schedules");
        let order: Vec<&str> = schedule
            .order()
            .iter()
            .map(|&index| graph.passes()[index].label.as_str())
            .collect();
        let at = |label: &str| order.iter().position(|pass| *pass == label).unwrap();
        assert!(at(&simulate_label("flames")) < at(&draw_label("flames")));
        assert!(matches!(
            graph.resource_desc(particles).map(|desc| &desc.persistence),
            Some(crate::pass::Persistence::Persistent { history: 0 })
        ));
    }

    #[test]
    fn the_simulation_covers_every_slot_the_sprites_draw() {
        let EffectKind::Compute { workgroups, .. } = SIMULATE.kind else {
            panic!("a compute effect")
        };
        let EffectKind::Sprites { instances, .. } = DRAW.kind else {
            panic!("a sprites effect")
        };
        assert_eq!(workgroups[0] * WORKGROUP, instances);
        assert!(include_str!("../../shaders/particles_simulate.wxsl")
            .contains(&format!("@workgroup_size({WORKGROUP})")));
    }
}
