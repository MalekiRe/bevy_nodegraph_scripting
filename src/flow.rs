use bevy::prelude::{Component, Entity};

#[derive(Component, Clone, Copy)]
#[relationship(relationship_target = Child)]
pub struct Parent(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = Parent)]
pub struct Child(Vec<Entity>);
