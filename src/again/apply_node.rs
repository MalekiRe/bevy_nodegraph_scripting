use crate::Bytecode;
use crate::again::{DataType, Node, Port, TypeData, Viewer};
use bevy::prelude::World;
use bevy::reflect::func::args::Ownership;
use egui::Ui;
use egui_snarl::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use std::collections::HashMap;

#[derive(Default)]
pub struct ApplyNode {
    node_id: Option<NodeId>,
}

impl Node for ApplyNode {
    fn inputs(node: NodeId, snarl_viewer: &mut Viewer, snarl: &Snarl<Box<dyn Node>>) -> usize
    where
        Self: Sized,
    {
        3
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
        "Apply".to_string()
    }

    fn input_port(pin: InPin, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) -> Port
    where
        Self: Sized,
    {
        if pin.id.input == 0 {
            return Port::Flow(0);
        }
        if pin.id.input == 1 {
            let Some(remote) = pin.remotes.first().cloned() else { return Port::Data(DataType::OwnershipOnly(Ownership::Mut)) };
            let t = snarl.get_node(remote.node).unwrap().get_traits();
            let port = t.output_port_2(snarl.out_pin(remote).clone(), snarl_viewer, snarl);
            if let Port::Data(DataType::Data(TypeData(type_info, ownership))) = port {
                if ownership == Ownership::Mut {
                    return Port::Data(DataType::Data(TypeData(type_info, Ownership::Mut)));
                }
            }
            return Port::Data(DataType::OwnershipOnly(Ownership::Mut));
        }
        let pin = snarl.in_pin(InPinId { node: pin.id.node, input: 1 });
        if let Port::Data(DataType::Data(TypeData(type_info, _))) = Self::input_port(pin, snarl_viewer, snarl) {
            return Port::Data(DataType::Data(TypeData(type_info, Ownership::Owned)));
        }
        Port::Data(DataType::OwnershipOnly(Ownership::Owned))
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
        todo!()
    }
}
