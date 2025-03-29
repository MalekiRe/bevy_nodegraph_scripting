use crate::Bytecode;
use crate::again::{Node, Port, Viewer};
use bevy::prelude::World;
use egui_snarl::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use std::collections::HashMap;

#[derive(Default)]
pub struct StartNode {
    node_id: Option<NodeId>,
}

impl Node for StartNode {
    fn inputs(node: NodeId, snarl_viewer: &mut Viewer, snarl: &Snarl<Box<dyn Node>>) -> usize
    where
        Self: Sized,
    {
        0
    }

    fn outputs(node: NodeId, snarl_viewer: &mut Viewer, snarl: &Snarl<Box<dyn Node>>) -> usize
    where
        Self: Sized,
    {
        1
    }

    fn title() -> String
    where
        Self: Sized,
    {
        "Start".to_string()
    }

    fn input_port(pin: InPin, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) -> Port
    where
        Self: Sized,
    {
        unreachable!()
    }

    fn output_port(pin: OutPin, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) -> Port
    where
        Self: Sized,
    {
        Port::Flow(0)
    }

    fn node_id(&self) -> NodeId {
        self.node_id.unwrap()
    }

    fn set_node_id(&mut self, node: NodeId) {
        self.node_id.replace(node);
    }

    fn compile(pin: InPin, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>, bytecode: &mut Vec<Bytecode>, scope_map: &mut HashMap<OutPinId, usize>, stack_ptr: &mut usize, world: &mut World) -> Option<InPinId>
    where
        Self: Sized,
    {
        unreachable!()
    }
}
