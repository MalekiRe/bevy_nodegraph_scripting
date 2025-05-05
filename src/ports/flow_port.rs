use bevy::prelude::{Component, Entity};

#[derive(Component, Clone)]
#[relationship(relationship_target = In)]
pub struct Out(pub Entity);

#[derive(Component, Clone)]
#[relationship_target(relationship = Out)]
pub struct In(Vec<Entity>);
