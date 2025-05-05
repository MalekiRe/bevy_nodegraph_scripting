use bevy::app::App;

pub mod connection;
pub mod data_port;
pub mod flow_port;
pub mod sided;

#[derive(bevy::prelude::Component, Clone, Copy)]
pub enum PortIndex {
    Out(usize),
    In(usize),
}

#[derive(bevy::prelude::Component)]
pub struct Flow;

pub struct PortPlugins;
impl bevy::prelude::Plugin for PortPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(data_port::DataPortPlugin);
    }
}
