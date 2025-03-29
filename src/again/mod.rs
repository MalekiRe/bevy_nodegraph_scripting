mod apply_node;
mod ownership_node;
mod primitive_node;
mod start_node;

use crate::Bytecode;
use crate::again::apply_node::ApplyNode;
use crate::again::ownership_node::OwnershipNode;
use crate::again::primitive_node::PrimitiveNode;
use crate::again::start_node::StartNode;
use crate::ui::split_u64_to_u8s;
use bevy::prelude::{AppTypeRegistry, World};
use bevy::reflect::func::args::Ownership;
use bevy::reflect::{DynamicTypePath, TypeInfo};
use egui::{Color32, Pos2, Ui};
use egui_snarl::ui::{PinInfo, SnarlPin, SnarlViewer, WireStyle};
use egui_snarl::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use std::hash::{DefaultHasher, Hash, Hasher};
// All input flow nodes should be pure RED unless all data inputs are satifised
// Data input types must match or connections do not work
// When trying to connect a flow node, the data inputs must be within scope of that node.
// So all non-within-scope flow inputs turn red.
// So color and shape are determined by factors that are always outside
// So we don't return a pin info from show input.

#[derive(Debug, Clone)]
pub enum Port {
    Flow(Scope),
    Data(DataType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Blank,
    Data(TypeData),
    OwnershipOnly(Ownership),
}

impl Into<PinInfo> for DataType {
    fn into(self) -> PinInfo {
        match self {
            DataType::Blank => PinInfo::circle().with_fill(Color32::WHITE),
            DataType::Data(data) => {
                let mut hasher = DefaultHasher::new();
                data.to_string().hash(&mut hasher);
                let awa = hasher.finish();
                let (r, g, b) = split_u64_to_u8s(awa);
                match data.1 {
                    Ownership::Ref => PinInfo::circle(),
                    Ownership::Mut => PinInfo::circle(),
                    Ownership::Owned => PinInfo::circle(),
                }
                .with_fill(Color32::from_rgb(r, g, b))
            }
            DataType::OwnershipOnly(ownership) => match ownership {
                Ownership::Ref => PinInfo::circle(),
                Ownership::Mut => PinInfo::circle(),
                Ownership::Owned => PinInfo::circle(),
            }
            .with_fill(Color32::WHITE),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypeData(TypeInfo, Ownership);

impl Display for TypeData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.1 {
            Ownership::Ref => f.write_str("&")?,
            Ownership::Mut => f.write_str("&mut ")?,
            Ownership::Owned => {}
        };
        f.write_str(self.0.type_path())?;
        Ok(())
    }
}

impl PartialEq for TypeData {
    fn eq(&self, other: &Self) -> bool {
        if self.1 != other.1 {
            return false;
        }
        self.0.type_id() == other.0.type_id()
    }
}

impl PartialEq for Port {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Port::Flow(scope_a), Port::Flow(scope_b)) => scope_a == scope_b,
            (Port::Data(scope_a), Port::Data(scope_b)) => scope_a == scope_b,
            (_, _) => false,
        }
    }
}

/// This is an ever-incrementing value that determines the scope of a flow node.
/// For a for loop for example, it increments when inside it, and then decrements when it ends
/// But the next for loop it is actually the current scope + 2 instead of reusing the value
pub type Scope = usize;

pub trait Node: Node2 + Any {
    fn inputs(node: NodeId, snarl_viewer: &mut Viewer, snarl: &Snarl<Box<dyn Node>>) -> usize
    where
        Self: Sized;
    fn outputs(node: NodeId, snarl_viewer: &mut Viewer, snarl: &Snarl<Box<dyn Node>>) -> usize
    where
        Self: Sized;

    fn title() -> String
    where
        Self: Sized;
    fn show_header(_node: NodeId, ui: &mut Ui, _snarl_viewer: &mut Viewer, _snarl: &mut Snarl<Box<dyn Node>>)
    where
        Self: Sized,
    {
        ui.label(Self::title());
    }
    fn show_input_port(pin: InPin, ui: &mut Ui, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>)
    where
        Self: Sized,
    {
    }
    fn show_output_port(pin: OutPin, ui: &mut Ui, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>)
    where
        Self: Sized,
    {
    }
    fn input_port(pin: InPin, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) -> Port
    where
        Self: Sized;
    fn output_port(pin: OutPin, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) -> Port
    where
        Self: Sized;
    fn node_id(&self) -> NodeId;
    fn set_node_id(&mut self, node: NodeId);
    fn compile(pin: InPin, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>, bytecode: &mut Vec<Bytecode>, scope_map: &mut HashMap<OutPinId, usize>, stack_ptr: &mut usize, world: &mut World) -> Option<InPinId>
    where
        Self: Sized;
}

impl dyn Node {
    pub fn downcast<T: 'static>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref()
    }
    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        (self as &mut dyn Any).downcast_mut()
    }
}

pub(crate) trait Node2 {
    fn inputs_2(&self, node: NodeId, snarl_viewer: &mut Viewer, snarl: &Snarl<Box<dyn Node>>) -> usize;

    fn outputs_2(&self, node: NodeId, snarl_viewer: &mut Viewer, snarl: &Snarl<Box<dyn Node>>) -> usize;

    fn title_2(&self) -> String;
    fn show_header_2(&self, node: NodeId, ui: &mut Ui, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>);
    fn show_input_port_2(&self, pin: InPin, ui: &mut Ui, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>);
    fn show_output_port_2(&self, pin: OutPin, ui: &mut Ui, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>);
    fn input_port_2(&self, pin: InPin, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) -> Port;
    fn output_port_2(&self, pin: OutPin, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) -> Port;

    fn get_traits(&self) -> Box<dyn Node2>;
}

impl<T: 'static + Node + Default> Node2 for T {
    fn inputs_2(&self, node: NodeId, snarl_viewer: &mut Viewer, snarl: &Snarl<Box<dyn Node>>) -> usize {
        T::inputs(node, snarl_viewer, snarl)
    }

    fn outputs_2(&self, node: NodeId, snarl_viewer: &mut Viewer, snarl: &Snarl<Box<dyn Node>>) -> usize {
        T::outputs(node, snarl_viewer, snarl)
    }

    fn title_2(&self) -> String {
        T::title()
    }

    fn show_header_2(&self, node: NodeId, ui: &mut Ui, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) {
        T::show_header(node, ui, snarl_viewer, snarl)
    }

    fn show_input_port_2(&self, pin: InPin, ui: &mut Ui, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) {
        T::show_input_port(pin, ui, snarl_viewer, snarl)
    }

    fn show_output_port_2(&self, pin: OutPin, ui: &mut Ui, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) {
        T::show_output_port(pin, ui, snarl_viewer, snarl)
    }

    fn input_port_2(&self, pin: InPin, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) -> Port {
        T::input_port(pin, snarl_viewer, snarl)
    }

    fn output_port_2(&self, pin: OutPin, snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) -> Port {
        T::output_port(pin, snarl_viewer, snarl)
    }

    fn get_traits(&self) -> Box<dyn Node2> {
        Box::new(T::default())
    }
}

pub struct Viewer {
    pub(crate) inputs_list: HashMap<NodeId, usize>,
    pub(crate) outputs_list: HashMap<NodeId, usize>,
    pub registry: AppTypeRegistry,
}

impl SnarlViewer<Box<dyn Node>> for Viewer {
    fn title(&mut self, node: &Box<dyn Node>) -> String {
        node.title_2()
    }

    fn show_header(&mut self, node: NodeId, inputs: &[InPin], outputs: &[OutPin], ui: &mut Ui, snarl: &mut Snarl<Box<dyn Node>>) {
        let traits = snarl.get_node(node).unwrap().get_traits();
        traits.show_header_2(node, ui, self, snarl);
    }

    fn inputs(&mut self, node: &Box<dyn Node>) -> usize {
        *self.inputs_list.get(&node.node_id()).unwrap()
    }

    fn has_node_menu(&mut self, node: &Box<dyn Node>) -> bool {
        true
    }

    fn show_node_menu(&mut self, node: NodeId, inputs: &[InPin], outputs: &[OutPin], ui: &mut Ui, snarl: &mut Snarl<Box<dyn Node>>) {
        if ui.button("Delete").clicked() {
            snarl.remove_node(node);
            ui.close_menu();
            return;
        }
    }

    fn has_graph_menu(&mut self, pos: Pos2, snarl: &mut Snarl<Box<dyn Node>>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: Pos2, ui: &mut Ui, snarl: &mut Snarl<Box<dyn Node>>) {
        let nodes: Vec<Box<dyn Node>> = vec![Box::new(StartNode::default()), Box::new(PrimitiveNode::default()), Box::new(OwnershipNode::default()), Box::new(ApplyNode::default())];
        for node in nodes {
            if ui.button(node.title_2()).clicked() {
                snarl.insert_node(pos, node);
                ui.close_menu();
            }
        }
    }

    fn show_input(&mut self, pin: &InPin, ui: &mut Ui, snarl: &mut Snarl<Box<dyn Node>>) -> impl SnarlPin + 'static {
        let node = snarl.get_node(pin.id.node).unwrap();
        let inputs = self.inputs(node);
        let traits = node.get_traits();
        let ret = match traits.input_port_2(pin.clone(), self, snarl) {
            //TODO do scope matching
            Port::Flow(_) => {
                for input in 0..inputs {
                    let in_pin = snarl.in_pin(InPinId { node: pin.id.node, input });
                    match traits.input_port_2(in_pin, self, snarl) {
                        Port::Data(DataType::Blank) | Port::Data(DataType::OwnershipOnly(_)) => return PinInfo::triangle().with_fill(Color32::RED),
                        _ => {}
                    }
                }
                PinInfo::triangle().with_fill(Color32::GREEN)
            }
            Port::Data(data_type) => {
                if let DataType::Data(type_data) = &data_type {
                    ui.label(type_data.to_string());
                }
                if let DataType::OwnershipOnly(ownership) = &data_type {
                    let s = match ownership {
                        Ownership::Ref => "&",
                        Ownership::Mut => "&mut",
                        Ownership::Owned => "",
                    };
                    ui.label(s);
                }
                data_type.into()
            }
        }
        .with_wire_style(WireStyle::AxisAligned { corner_radius: 30.0 });
        traits.show_input_port_2(pin.clone(), ui, self, snarl);
        ret
    }

    fn outputs(&mut self, node: &Box<dyn Node>) -> usize {
        *self.outputs_list.get(&node.node_id()).unwrap()
    }

    fn show_output(&mut self, pin: &OutPin, ui: &mut Ui, snarl: &mut Snarl<Box<dyn Node>>) -> impl SnarlPin + 'static {
        let node = snarl.get_node(pin.id.node).unwrap();
        let outputs = self.outputs(node);
        let traits = node.get_traits();
        let ret = match traits.output_port_2(pin.clone(), self, snarl) {
            //TODO do scope matching
            Port::Flow(_) => {
                for output in 0..outputs {
                    let out_pin = snarl.out_pin(OutPinId { node: pin.id.node, output });
                    match traits.output_port_2(out_pin, self, snarl) {
                        Port::Data(DataType::Blank) | Port::Data(DataType::OwnershipOnly(_)) => return PinInfo::triangle().with_fill(Color32::RED),
                        _ => {}
                    }
                }
                PinInfo::triangle().with_fill(Color32::GREEN)
            }
            Port::Data(data_type) => {
                if let DataType::Data(type_data) = &data_type {
                    ui.label(type_data.to_string());
                }
                if let DataType::OwnershipOnly(ownership) = &data_type {
                    let s = match ownership {
                        Ownership::Ref => "&",
                        Ownership::Mut => "&mut",
                        Ownership::Owned => "",
                    };
                    ui.label(s);
                }
                data_type.into()
            }
        }
        .with_wire_style(WireStyle::AxisAligned { corner_radius: 30.0 });
        traits.show_output_port_2(pin.clone(), ui, self, snarl);
        ret
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<Box<dyn Node>>) {
        let from_node = snarl.get_node(from.id.node).unwrap().get_traits();
        let to_node = snarl.get_node(to.id.node).unwrap().get_traits();
        if let Port::Flow(_) = to_node.input_port_2(to.clone(), self, snarl) {
            let inputs = self.inputs(&snarl.get_node(to.id.node).unwrap());
            for input in 0..inputs {
                let in_pin = snarl.in_pin(InPinId { node: to.id.node, input });
                match to_node.input_port_2(in_pin, self, snarl) {
                    Port::Data(DataType::Blank) | Port::Data(DataType::OwnershipOnly(_)) => {
                        return;
                    }
                    _ => {}
                }
            }
            snarl.connect(from.id, to.id);
        } else {
            if match (from_node.output_port_2(from.clone(), self, snarl), to_node.input_port_2(to.clone(), self, snarl)) {
                ((Port::Data(DataType::Data(_))), Port::Data(DataType::Blank)) => true,
                (Port::Data(DataType::Data(a)), Port::Data(DataType::Data(b))) => a == b,
                (Port::Data(DataType::Data(a)), Port::Data(DataType::OwnershipOnly(b))) => a.1 == b,
                _ => false,
            } {
                snarl.connect(from.id, to.id);
            }
        }
    }
}
