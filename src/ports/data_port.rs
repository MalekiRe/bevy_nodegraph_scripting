use crate::node::shape::ShapeSettings;
use crate::type_data::TypeData;
use bevy::app::App;
use bevy::color::palettes::css;
use bevy::ecs::component::HookContext;
use bevy::ecs::system::RunSystemOnce;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::{
    Assets, Circle, Color, Commands, Component, Entity, Mesh, Mesh2d, MeshMaterial2d, OnAdd,
    Plugin, Query, Res, ResMut, Trigger, World,
};
use bevy::reflect::func::args::Ownership;
use bevy::sprite::ColorMaterial;

#[derive(Component, Clone)]
#[relationship_target(relationship = In)]
pub struct Out(Vec<Entity>);

#[derive(Component, Clone)]
#[relationship(relationship_target = Out)]
pub struct In(pub Entity);

#[derive(Component, Clone)]
pub struct OutType(pub TypeData);

#[derive(Component, Clone)]
#[component(immutable)]
pub enum InType {
    Blank,
    TypeData(TypeData),
    Ownership(Ownership),
}

pub struct DataPortPlugin;
impl Plugin for DataPortPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_input_data_port);
        app.add_observer(spawn_output_data_port);
    }
}

fn spawn_output_data_port(
    trigger: Trigger<OnAdd, OutType>,
    mut commands: Commands,
    out_type: Query<&OutType>,
    shape_settings: Res<ShapeSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let out_type = out_type.get(trigger.target()).unwrap();
    let color: Color = out_type.0.clone().into();
    commands.entity(trigger.target()).insert((
        Mesh2d(meshes.add(Circle::new(shape_settings.port_radius - 3.0))),
        MeshMaterial2d(materials.add(color)),
    ));
}

fn spawn_input_data_port(
    trigger: Trigger<OnAdd, InType>,
    mut commands: Commands,
    in_type: Query<&InType>,
    shape_settings: Res<ShapeSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let in_type = in_type.get(trigger.target()).unwrap();
    let color = match in_type {
        InType::Blank => Color::WHITE,
        InType::TypeData(type_data) => type_data.clone().into(),
        InType::Ownership(ownership) => match ownership {
            Ownership::Ref => css::CRIMSON,
            Ownership::Mut => css::ALICE_BLUE,
            Ownership::Owned => css::BROWN,
        }
        .into(),
    };
    commands.entity(trigger.target()).insert((
        Mesh2d(meshes.add(Circle::new(shape_settings.port_radius - 3.0))),
        MeshMaterial2d(materials.add(color)),
    ));
}
