use crate::node;
use bevy::prelude::*;
use bevy_egui::egui::panel::Side;
use bevy_egui::{EguiContextPass, EguiContexts, egui};
use random_number::random;

pub struct NodeMenuPlugin;

impl Plugin for NodeMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AllNodes>();
        app.add_systems(EguiContextPass, sidebar_menu);
    }
}

#[derive(Resource, Deref)]
pub struct AllNodes(pub Vec<node::Node>);

impl Default for AllNodes {
    fn default() -> Self {
        Self(vec![node::Node::new(node::nodes::primitive::PrimitiveNode)])
    }
}

pub fn sidebar_menu(mut commands: Commands, mut ctx: EguiContexts, all_nodes: Res<AllNodes>) {
    let Some(ctx) = ctx.try_ctx_mut() else { return };
    egui::SidePanel::new(Side::Left, "Side").show(ctx, |ui| {
        for node in all_nodes.iter() {
            if ui.button(node.name()).clicked() {
                commands.spawn((
                    node.clone_node(),
                    Transform::from_translation(Vec3::new(
                        random!(-30.0..30.0),
                        random!(-30.0..30.0),
                        1.0,
                    )),
                ));
            }
        }
    });
}
