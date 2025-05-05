use crate::{ProjectionScale, flow, node};
use bevy::prelude::*;
use std::ops::Mul;

pub struct DragNodePlugin;

impl Plugin for DragNodePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(
            |event: Trigger<OnAdd, node::Node>, mut commands: Commands| {
                commands.entity(event.target()).observe(drag_node);
            },
        );
    }
}

pub fn drag_node(
    mut e: Trigger<Pointer<Drag>>,
    mut commands: Commands,
    projection_scale: Res<ProjectionScale>,
    mut transforms: Query<(&mut Transform, &mut GlobalTransform), With<node::Node>>,
    parent: Query<Entity, With<ChildOf>>,
) {
    let Ok((mut transform, mut global_transform)) = transforms.get_mut(e.target) else {
        return;
    };
    if parent.contains(e.target) {
        commands.entity(e.target).remove::<ChildOf>();
        *transform = global_transform.reparented_to(&GlobalTransform::default());
        commands.entity(e.target).remove::<flow::Parent>();
    }
    let delta = Vec2::new(e.delta.x, -e.delta.y).mul(Vec2::splat(projection_scale.0));
    let mut g_transform = global_transform.compute_transform();
    transform.translation.y += delta.y;
    transform.translation.x += delta.x;
    g_transform.translation.y += delta.y;
    g_transform.translation.x += delta.x;
    *global_transform = GlobalTransform::from(g_transform);
}
