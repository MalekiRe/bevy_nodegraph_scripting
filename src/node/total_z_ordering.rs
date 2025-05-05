use crate::node;
use bevy::prelude::*;

pub struct TotalZOrderingPlugin;

impl Plugin for TotalZOrderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TotalOrdering>();
        app.add_observer(reordering);
        app.add_observer(
            |event: Trigger<OnAdd, node::Node>, mut commands: Commands| {
                commands
                    .entity(event.target())
                    .observe(push_to_top_observer);
                commands.entity(event.target()).observe(
                    |event: Trigger<OnAdd, ChildOf>,
                     mut total_ordering: ResMut<TotalOrdering>,
                     mut transforms: Query<&mut Transform>| {
                        let mut index = None;
                        for (i, e) in total_ordering.0.iter().enumerate() {
                            if *e == event.target() {
                                index = Some(i);
                            }
                        }
                        if let Some(index) = index {
                            total_ordering.0.remove(index);
                        }
                        if let Ok(mut transform) = transforms.get_mut(event.target()) {
                            transform.translation.z = 0.01;
                        }
                    },
                );
                commands.entity(event.target()).observe(
                    |event: Trigger<OnRemove, ChildOf>,
                     mut commands: Commands,
                     total_ordering: ResMut<TotalOrdering>| {
                        commands.entity(event.target()).trigger(PushToTop);
                    },
                );
                commands.entity(event.target()).trigger(PushToTop);
            },
        );
    }
}

#[derive(Event)]
pub struct PushToTop;

#[derive(Event)]
pub struct TriggerReordering;

fn reordering(
    _: Trigger<TriggerReordering>,
    total_ordering: Res<TotalOrdering>,
    mut transforms: Query<&mut Transform, (With<node::Node>, Without<ChildOf>)>,
) {
    for (i, entity) in total_ordering.0.iter().enumerate() {
        let mut transform = transforms.get_mut(*entity).unwrap();
        transform.translation.z = i as f32;
    }
}

#[derive(Resource, Default)]
struct TotalOrdering(Vec<Entity>);

fn push_to_top_observer(
    event: Trigger<PushToTop>,
    mut commands: Commands,
    mut total_ordering: ResMut<TotalOrdering>,
    mut transforms: Query<&mut Transform, (With<node::Node>, Without<ChildOf>)>,
) {
    commands.trigger(TriggerReordering);
    if !transforms.contains(event.target()) {
        return;
    }
    if !total_ordering.0.contains(&event.target()) {
        total_ordering.0.push(event.target());
    }
    let mut index = None;
    for (i, e) in total_ordering.0.iter().enumerate() {
        if *e == event.target() {
            index = Some(i);
            break;
        }
    }
    let index = index.unwrap();
    total_ordering.0.remove(index);
    total_ordering.0.push(event.target());
}
