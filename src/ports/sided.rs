use bevy::prelude::Entity;
use evergreen_relations::prelude::*;
/// An undirected 1:1 relationship between entities.
#[derive(Relation)]
#[relation(source = OtherSideOf, target = OtherSideOf)]
pub struct LeftRightRelation;

pub type OtherSide = Related<OtherSideOf>;

#[derive(Relatable)]
#[relatable(Entity in LeftRightRelation, opposite = OtherSideOf)]
pub struct OtherSideOf;
