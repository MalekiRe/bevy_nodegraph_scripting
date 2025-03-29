use crate::Bytecode;
use crate::again::{DataType, Node, Port, TypeData, Viewer};
use bevy::prelude::World;
use bevy::reflect::func::args::Ownership;
use egui::{ComboBox, Ui};
use egui_snarl::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub struct OwnershipNode {
    pub node_id: Option<NodeId>,
    pub ownership: Ownership,
}

impl Display for OwnershipNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.ownership {
            Ownership::Owned => f.write_str("clone"),
            Ownership::Ref => f.write_str("&"),
            Ownership::Mut => f.write_str("&mut"),
        }
    }
}

impl Default for OwnershipNode {
    fn default() -> Self {
        Self { node_id: None, ownership: Ownership::Owned }
    }
}

impl Node for OwnershipNode {
    fn inputs(node: NodeId, snarl_viewer: &mut Viewer, snarl: &Snarl<Box<dyn Node>>) -> usize
    where
        Self: Sized,
    {
        2
    }
    fn outputs(node: NodeId, snarl_viewer: &mut Viewer, snarl: &Snarl<Box<dyn Node>>) -> usize
    where
        Self: Sized,
    {
        2
    }

    fn title() -> String
    where
        Self: Sized,
    {
        "Ownership".to_string()
    }

    fn show_header(node: NodeId, ui: &mut Ui, _snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>)
    where
        Self: Sized,
    {
        let ownership = snarl.get_node_mut(node).unwrap().downcast_mut::<OwnershipNode>().unwrap();
        let f = format!("{}", ownership);
        let ownership = &mut ownership.ownership;
        ComboBox::from_label("Ownership").selected_text(format!("{}", f).as_str()).show_ui(ui, |ui| {
            ui.selectable_value(ownership, Ownership::Owned, "clone");
            ui.selectable_value(ownership, Ownership::Ref, "&");
            ui.selectable_value(ownership, Ownership::Mut, "&mut");
        });
    }

    fn input_port(pin: InPin, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) -> Port
    where
        Self: Sized,
    {
        if pin.id.input == 0 {
            return Port::Flow(0);
        }
        let Some(remote) = pin.remotes.first() else {
            return Port::Data(DataType::Blank);
        };
        let remote = snarl.out_pin(*remote);
        snarl.get_node(remote.id.node).unwrap().get_traits().output_port_2(remote, snarl_viewer, snarl)
    }

    fn output_port(pin: OutPin, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) -> Port
    where
        Self: Sized,
    {
        if pin.id.output == 0 {
            return Port::Flow(0);
        }
        let in_pin = snarl.in_pin(InPinId { node: pin.id.node, input: 1 });
        let ownership = snarl.get_node(pin.id.node).unwrap().downcast::<Self>().unwrap().ownership.clone();
        match Self::input_port(in_pin, snarl_viewer, snarl) {
            Port::Data(DataType::Data(data)) => Port::Data(DataType::Data(TypeData(data.0, ownership))),
            _ => Port::Data(DataType::OwnershipOnly(ownership)),
        }
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
        let input_port = Self::input_port(snarl.in_pin(InPinId { node: pin.id.node, input: 1 }).clone(), snarl_viewer, snarl);

        todo!()
    }
}
