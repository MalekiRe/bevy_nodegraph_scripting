use crate::node::CONNECTION_HEIGHT_PADDING;
use crate::node::shape::{NotchType, ShapeData, ShapeSettings};
use crate::node::total_z_ordering::PushToTop;
use crate::{flow, node};
use bevy::prelude::TransformSystem::TransformPropagate;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::Fill;
use std::ops::{Add, Sub};

pub struct DragSnapNodePlugin;

impl Plugin for DragSnapNodePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ConnectBlocks>()
            .add_observer(
                |event: Trigger<OnAdd, node::Node>, mut commands: Commands| {
                    commands
                        .entity(event.target())
                        .observe(drag_node)
                        .observe(finish_drag_node);
                },
            )
            .add_systems(PostUpdate, handle_connect_blocks.after(TransformPropagate));
    }
}

#[derive(Component)]
struct Temporary {
    parent: Entity,
    child: Entity,
    ideal_position: IdealPosition,
}

enum IdealPosition {
    Child(Vec2),
    Parent(Vec2),
}

#[derive(Event)]
struct ConnectBlocks {
    parent: Entity,
    child: Entity,
}

fn handle_connect_blocks(
    mut commands: Commands,
    mut event_reader: EventReader<ConnectBlocks>,
    mut query: Query<(Entity, &mut Transform, &GlobalTransform)>,
    transforms: Query<&GlobalTransform>,
    node: Query<&node::Node>,
) {
    for ConnectBlocks { parent, child } in event_reader.read() {
        let Ok(parent_transform) = transforms.get(*parent) else {
            continue;
        };
        let Ok((child, mut child_transform, child_global_transform)) = query.get_mut(*child) else {
            continue;
        };
        *child_transform = child_global_transform.reparented_to(parent_transform);
        let node = node.get(child).unwrap().clone_node();
        commands
            .entity(child)
            .insert((ChildOf(*parent), flow::Parent(*parent), node));
    }
}

fn drag_node(
    mut event: Trigger<Pointer<Drag>>,
    mut commands: Commands,
    t: Query<(Entity, &GlobalTransform, &ShapeData, &node::Node), Without<Temporary>>,
    children: Query<&Children>,
    parent: Query<&ChildOf>,
    node_query: Query<&node::Node>,
    temporary: Query<(Entity, &Temporary)>,
) {
    let Ok((e, global_transform, shape_data, node)) = t.get(event.target) else {
        return;
    };
    for (e2, global_transform2, shape_data2, node2) in t.iter() {
        if e == e2 {
            continue;
        }

        let ideal_position = global_transform2.translation().xy().sub(Vec2::new(
            0.0,
            shape_data.height + CONNECTION_HEIGHT_PADDING,
        ));
        let distance = global_transform.translation().xy().distance(ideal_position);

        let ideal_position2 = global_transform2.translation().xy().add(Vec2::new(
            0.0,
            shape_data2.height + CONNECTION_HEIGHT_PADDING,
        ));
        let distance2 = global_transform
            .translation()
            .xy()
            .distance(ideal_position2);
        let mut cloned_shape = shape_data.clone();
        cloned_shape.color = Color::srgba_u8(80, 80, 80, 100);
        if distance < 45.0 {
            if !temporary.is_empty() {
                return;
            }
            match node.notch_type() {
                NotchType::Bottom => return,
                _ => {}
            }
            match node2.notch_type() {
                NotchType::Top => return,
                _ => {}
            }
            for child in children.get(e2).unwrap() {
                if node_query.contains(*child) {
                    return;
                }
            }
            commands.spawn((
                cloned_shape,
                Transform::from_translation(ideal_position.extend(-0.5)),
                Temporary {
                    parent: e2,
                    child: e,
                    ideal_position: IdealPosition::Child(ideal_position),
                },
            ));
            return;
        } else if distance2 <= 45.0 {
            if !temporary.is_empty() {
                return;
            }
            match node2.notch_type() {
                NotchType::Bottom => return,
                _ => {}
            }
            match node.notch_type() {
                NotchType::Top => return,
                _ => {}
            }
            for child in children.get(e).unwrap() {
                if node_query.contains(*child) {
                    return;
                }
            }
            if parent.contains(e2) {
                return;
            }

            commands.spawn((
                cloned_shape,
                Transform::from_translation(ideal_position2.extend(-0.5)),
                Temporary {
                    parent: e,
                    child: e2,
                    ideal_position: IdealPosition::Parent(ideal_position2),
                },
            ));
            return;
        } else {
            for (temporary_e, temporary) in temporary.iter() {
                if (temporary.child == e2 && temporary.parent == e)
                    || (temporary.child == e && temporary.parent == e2)
                {
                    commands.entity(temporary_e).try_despawn();
                }
            }
        }
    }
}

fn finish_drag_node(
    mut trigger: Trigger<Pointer<DragEnd>>,
    mut commands: Commands,
    mut query: Query<&mut Transform, (Without<Temporary>, With<node::Node>)>,
    temporary: Query<(Entity, &Temporary)>,
    asset_server: Res<AssetServer>,
    mut event_writer: EventWriter<ConnectBlocks>,
) {
    let mut awa = 2.0;
    for (i, mut t) in query.iter_mut().enumerate() {
        t.translation.z = (awa * i as f32)
    }
    let len = query.iter().len() as f32 * awa;
    if let Ok(mut transform) = query.get_mut(trigger.target) {
        transform.translation.z = len;
    };
    let Some((
        _,
        Temporary {
            parent,
            child,
            ideal_position,
        },
    )) = temporary.iter().next()
    else {
        return;
    };
    match ideal_position {
        IdealPosition::Child(ideal_position) => {
            let translation = &mut query.get_mut(*child).unwrap().translation;
            translation.x = ideal_position.x;
            translation.y = ideal_position.y;
        }
        IdealPosition::Parent(ideal_position) => {
            let translation = &mut query.get_mut(*parent).unwrap().translation;
            translation.x = ideal_position.x;
            translation.y = ideal_position.y;
        }
    }
    event_writer.write(ConnectBlocks {
        parent: *parent,
        child: *child,
    });
    commands.spawn(AudioPlayer::new(asset_server.load("node_connected.ogg")));
    for (temporary_entity, _) in temporary.iter() {
        commands.entity(temporary_entity).try_despawn();
    }
}
