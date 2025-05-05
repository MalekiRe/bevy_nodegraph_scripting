use crate::node::node_traits::NodeTrait;
use crate::node::shape::{PortPosition, ShapeData, ShapeSettings};
use crate::ports::PortIndex;
use crate::ports::sided::OtherSide;
use crate::{flow, ports};
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::{
    ChildOf, Children, Commands, Component, Deref, DerefMut, Entity, OnAdd, OnRemove, Query,
    Transform, Trigger, World,
};
use std::ops::Deref;

pub mod interaction;
pub mod node_traits;
pub mod nodes;
pub mod shape;
pub mod total_z_ordering;

#[derive(Deref, DerefMut, Component)]
#[component(immutable)]
#[component(on_insert = on_insert_node)]
#[component(on_add = on_add_node)]
pub struct Node(Box<dyn NodeTrait>);

pub const CONNECTION_HEIGHT_PADDING: f32 = 2.0;

impl Node {
    pub fn new(node: impl NodeTrait) -> Self {
        Self(Box::new(node))
    }
}

fn on_add_node(mut world: DeferredWorld, hook_context: HookContext) {
    let entity = hook_context.entity;
    world
        .commands()
        .queue(move |world: &mut World| handle_adding_observer(world, entity));
    let entity = hook_context.entity;
    world
        .commands()
        .queue(move |world: &mut World| handle_input_ports(world, entity));
}

fn handle_adding_observer(world: &mut World, entity: Entity) {
    world.entity_mut(entity).observe(
        |trigger: Trigger<OnAdd, flow::Parent>, mut commands: Commands, query: Query<&Node>| {
            let n = query.get(trigger.target()).unwrap().clone_node();
            commands.entity(trigger.target()).insert(n);
        },
    );

    world.entity_mut(entity).observe(
        |trigger: Trigger<OnRemove, flow::Parent>, mut commands: Commands, query: Query<&Node>| {
            let n = query.get(trigger.target()).unwrap().clone_node();
            commands.entity(trigger.target()).insert(n);
        },
    );
}

fn on_insert_node(mut world: DeferredWorld, hook_context: HookContext) {
    let entity = hook_context.entity;
    world
        .commands()
        .queue(move |world: &mut World| handle_node_shape(world, entity));
    let entity = hook_context.entity;
    world
        .commands()
        .queue(move |world: &mut World| handle_output_ports(world, entity));
    let entity = hook_context.entity;
    world
        .commands()
        .queue(move |world: &mut World| handle_port_positions(world, entity));
}

fn handle_node_shape(world: &mut World, entity: Entity) {
    let shape_settings = world.resource::<ShapeSettings>().clone();
    let node = world.entity(entity).get::<Node>().unwrap();
    let new_shape_data = shape_settings.shape_data(node.shape_info(entity, &world));
    let Some(previous_shape_data) = world.entity(entity).get::<ShapeData>().cloned() else {
        world.entity_mut(entity).insert(new_shape_data);
        return;
    };
    if world.entity(entity).contains::<ChildOf>() {
        world
            .entity_mut(entity)
            .get_mut::<Transform>()
            .unwrap()
            .translation
            .y = (new_shape_data.height + CONNECTION_HEIGHT_PADDING) * -1.0;
    }
    if previous_shape_data != new_shape_data {
        world.entity_mut(entity).insert(new_shape_data);
    }
}

fn handle_input_ports(world: &mut World, entity: Entity) {
    let node = world.entity(entity).get::<Node>().unwrap().clone_node();
    let input_ports = node.get_inputs();
    let mut sided = vec![];
    for (i, data_port) in &input_ports.data_ports {
        sided.push(
            world
                .spawn((
                    PortPosition::TopRight(*i),
                    data_port.clone(),
                    ports::PortIndex::In(*i),
                    ChildOf(entity),
                ))
                .id(),
        );
    }
    for ((i, data_port), other_side) in input_ports.data_ports.iter().zip(sided) {
        world.spawn((
            PortPosition::TopLeft(*i),
            data_port.clone(),
            OtherSide::new(other_side),
            ports::PortIndex::Out(*i),
            ChildOf(entity),
        ));
    }

    let mut sided = vec![];
    for i in &input_ports.flow_ports {
        sided.push(
            world
                .spawn((
                    PortPosition::TopRight(*i),
                    ports::PortIndex::In(*i),
                    ports::Flow,
                    ChildOf(entity),
                ))
                .id(),
        );
    }
    for ((i), other_side) in input_ports.flow_ports.iter().zip(sided) {
        world.spawn((
            PortPosition::TopLeft(*i),
            OtherSide::new(other_side),
            ports::PortIndex::In(*i),
            ports::Flow,
            ChildOf(entity),
        ));
    }
}

fn handle_port_positions(world: &mut World, entity: Entity) {
    let node = world.entity(entity).get::<Node>().unwrap().clone_node();
    let shape_settings = world.resource::<ShapeSettings>().clone();
    let shape_data = shape_settings.shape_data(node.shape_info(entity, &world));
    let children = world
        .entity(entity)
        .get::<Children>()
        .unwrap()
        .iter()
        .cloned()
        .collect::<Vec<_>>();

    for child in children {
        world.entity(child).contains::<PortIndex>();
        let Some(port_position) = world
            .get_entity(child)
            .unwrap()
            .get::<PortPosition>()
            .cloned()
        else {
            continue;
        };
        world.entity_mut(child).insert(Transform::from_translation(
            shape_data.port_position(port_position).extend(0.1),
        ));
    }
}

fn handle_output_ports(world: &mut World, entity: Entity) {
    let children = world.entity(entity).get::<Children>().unwrap().iter().cloned().collect::<Vec<_>>();
    for child in children {
        let Some(port_position) = world.entity(child).get::<PortPosition>() else { continue };
        match port_position {
            PortPosition::BottomLeft(_) => {}
            PortPosition::BottomRight(_) => {}
            _ => continue,
        }
        world.entity_mut(child).despawn();
    }

    let node = world.entity(entity).get::<Node>().unwrap().clone_node();
    let output_ports = node.get_outputs(entity, world);
    let mut sided = vec![];
    for (i, flow_parent, output_type_opt) in &output_ports.data_ports {
        let mut port_entity = world
            .spawn((
                PortPosition::BottomRight(*i),
                *flow_parent,
                ports::PortIndex::Out(*i),
                ChildOf(entity),
            ));
        if let Some(output_type) = output_type_opt {
            port_entity.insert(output_type.clone());
        }
        sided.push(port_entity.id());
    }
    for ((i, flow_parent, output_type_opt), other_side) in output_ports.data_ports.iter().zip(sided) {
        let mut port_entity = world
            .spawn((
                PortPosition::BottomLeft(*i),
                *flow_parent,
                ports::PortIndex::Out(*i),
                ChildOf(entity),
                OtherSide::new(other_side),
            ));
        if let Some(output_type) = output_type_opt {
            port_entity.insert(output_type.clone());
        }
    }

    let mut sided = vec![];
    for (i, flow_parent) in &output_ports.flow_ports {
        sided.push(
            world
                .spawn((
                    PortPosition::BottomRight(*i),
                    ports::PortIndex::Out(*i),
                    ports::Flow,
                    ChildOf(entity),
                    *flow_parent,
                ))
                .id(),
        );
    }
    for ((i, flow_parent), other_side) in output_ports.flow_ports.iter().zip(sided) {
        world.spawn((
            PortPosition::BottomLeft(*i),
            OtherSide::new(other_side),
            ports::PortIndex::Out(*i),
            ports::Flow,
            ChildOf(entity),
            *flow_parent,
        ));
    }
}
