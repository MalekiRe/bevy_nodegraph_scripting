use crate::node::interaction::drag_snap::DragSnapNodePlugin;
use crate::node::interaction::dragging::DragNodePlugin;
use bevy::app::App;
use bevy::prelude::Plugin;

mod drag_snap;
mod dragging;

pub struct NodeInteractionPlugins;
impl Plugin for NodeInteractionPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((DragNodePlugin, DragSnapNodePlugin));
    }
}
