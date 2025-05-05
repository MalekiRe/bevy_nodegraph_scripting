use crate::node::interaction::NodeInteractionPlugins;
use crate::node::shape::NodeShapePlugin;
use crate::node::total_z_ordering::TotalZOrderingPlugin;
use crate::node::{Node, nodes};
use crate::node_menu::NodeMenuPlugin;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_prototype_lyon::plugin::ShapePlugin;

pub mod flow;
pub mod node;
mod node_menu;
pub mod ports;
pub mod type_data;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ShapePlugin,
            NodeShapePlugin,
            MeshPickingPlugin,
            EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
            NodeMenuPlugin,
            NodeInteractionPlugins,
            TotalZOrderingPlugin,
            bevy_framepace::FramepacePlugin,
            ports::PortPlugins,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, mouse_interactions)
        .run();
}

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.insert_resource(ProjectionScale(1.0));
}

#[derive(Resource)]
pub struct ProjectionScale(pub f32);
/// How fast we zoom and how smooth the pan feels.
/// Tweak to taste (1.0 = snap perfectly, < 1.0 = slower drift).
const ZOOM_SPEED: f32 = 0.20; // wheel “strength” → scale change
const SMOOTH_FACTOR: f32 = 0.85; // 1.0 = no smoothing

pub fn mouse_interactions(
    mut mouse_evt: EventReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cam_q: Query<(&mut Projection, &mut Transform, &Camera), With<Camera2d>>,
    mut proj: ResMut<ProjectionScale>,
) {
    // --- early outs ---------------------------------------------------------
    let scroll: f32 = mouse_evt.read().map(|e| e.y).sum();
    if scroll.abs() < f32::EPSILON {
        return;
    }

    let window = windows.single().unwrap();
    let Some(cursor_screen) = window.cursor_position() else {
        return;
    };

    // --- camera components --------------------------------------------------
    let (mut projection, mut cam_tf, camera) = cam_q.single_mut().unwrap();
    let ortho = match &mut *projection {
        Projection::Orthographic(o) => o,
        _ => return, // not 2‑D: ignore
    };

    // --- world position under cursor BEFORE zoom ---------------------------
    let Ok(world_before) =
        camera.viewport_to_world_2d(&GlobalTransform::from(*cam_tf), cursor_screen)
    else {
        return;
    };

    // --- compute new scale --------------------------------------------------
    let old_scale = ortho.scale;
    let new_scale = (ortho.scale * (1.0 - scroll * ZOOM_SPEED)).clamp(0.2, 20.0);
    ortho.scale = new_scale;
    proj.0 = new_scale;

    // --- analytic delta: keep cursor anchored ------------------------------
    //
    // scale affects the *distance from the camera* linearly:
    //   screen_pos = (world - cam_pos) / scale
    // so   world_after = cam_pos + (world_before - cam_pos) * (old_scale / new_scale)
    //
    let cam_pos = cam_tf.translation.truncate(); // xy only
    let factor = old_scale / new_scale;
    let world_after = cam_pos + (world_before - cam_pos) * factor;
    let delta = (world_before - world_after) * SMOOTH_FACTOR;

    cam_tf.translation.x -= delta.x;
    cam_tf.translation.y -= delta.y;
}

// what do i have to do left?
// draw shapes on the ports with the appropriate colors
// make the unchosen side disappear when the chosen side is chosen, and make it reappear when it's unchosen.
// drag to connect ports together
// create a start node
// click to unconnect ports
// add sound effects for grabbing nodes
// add sound effects for connecting nodes
// add sound effect for unconnecting nodes - just play the former in reverse?
// add other important nodes like queries, iterations, if else and match, ownership and construction / deconstruction
// create a compiler that walks through and generates bytecode
// make a virtual machine to run that bytecode
