#![feature(trait_upcasting)]
use crate::again::{DataType, Node, Port, TypeData, Viewer};
use crate::{Bytecode, Value};
use bevy::prelude::{PartialReflect, World};
use bevy::reflect::DynamicTyped;
use bevy::reflect::func::args::Ownership;
use egui::{DragValue, Ui, Widget};
use egui_snarl::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Default)]
pub struct PrimitiveNode {
    pub primitive_type: PrimitiveType,
    pub node_id: Option<NodeId>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrimitiveType {
    I32(i32),
    F32(f32),
    String(String),
}

impl PrimitiveType {
    pub fn into_reflect(self) -> Box<dyn PartialReflect> {
        match self {
            PrimitiveType::I32(val) => Box::new(val).into_partial_reflect(),
            PrimitiveType::F32(val) => Box::new(val).into_partial_reflect(),
            PrimitiveType::String(val) => Box::new(val).into_partial_reflect(),
        }
    }
}
impl Display for PrimitiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::I32(_) => f.write_str("i32"),
            PrimitiveType::F32(_) => f.write_str("f32"),
            PrimitiveType::String(_) => f.write_str("String"),
        }
    }
}
impl Default for PrimitiveType {
    fn default() -> Self {
        Self::I32(0)
    }
}

impl Node for PrimitiveNode {
    fn inputs(node: NodeId, snarl_viewer: &mut Viewer, snarl: &Snarl<Box<dyn Node>>) -> usize
    where
        Self: Sized,
    {
        1
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
        "Primitive".to_string()
    }

    fn show_header(node: NodeId, ui: &mut Ui, _snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>)
    where
        Self: Sized,
    {
        let node = snarl.get_node_mut(node).unwrap();
        let primitive_node = node.downcast_mut::<PrimitiveNode>().unwrap();
        egui::ComboBox::from_label("Primitive").selected_text(format!("{}", primitive_node.primitive_type)).show_ui(ui, |ui| {
            ui.selectable_value(&mut primitive_node.primitive_type, PrimitiveType::I32(0), "i32");
            ui.selectable_value(&mut primitive_node.primitive_type, PrimitiveType::F32(0.0), "f32");
            ui.selectable_value(&mut primitive_node.primitive_type, PrimitiveType::String(String::new()), "String");
        });
    }

    fn show_output_port(pin: OutPin, ui: &mut Ui, _snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>)
    where
        Self: Sized,
    {
        if pin.id.output == 0 {
            return;
        }
        match &mut snarl.get_node_mut(pin.id.node).unwrap().downcast_mut::<PrimitiveNode>().unwrap().primitive_type {
            PrimitiveType::I32(val) => {
                DragValue::new(val).ui(ui);
            }
            PrimitiveType::F32(val) => {
                DragValue::new(val).ui(ui);
            }
            PrimitiveType::String(val) => {
                ui.text_edit_singleline(val);
            }
        }
    }

    fn input_port(_pin: InPin, _snarl_viewer: &mut Viewer, _snarl: &mut Snarl<Box<dyn Node>>) -> Port
    where
        Self: Sized,
    {
        Port::Flow(0)
    }

    fn output_port(pin: OutPin, _snarl_viewer: &mut Viewer, snarl: &mut Snarl<Box<dyn Node>>) -> Port
    where
        Self: Sized,
    {
        if pin.id.output == 0 {
            return Port::Flow(0);
        }
        match &snarl.get_node(pin.id.node).unwrap().downcast::<PrimitiveNode>().unwrap().primitive_type {
            PrimitiveType::I32(val) => Port::Data(DataType::Data(TypeData(val.reflect_type_info().clone(), Ownership::Owned))),
            PrimitiveType::F32(val) => Port::Data(DataType::Data(TypeData(val.reflect_type_info().clone(), Ownership::Owned))),
            PrimitiveType::String(val) => Port::Data(DataType::Data(TypeData(val.reflect_type_info().clone(), Ownership::Owned))),
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
        scope_map.insert(OutPinId { node: pin.id.node, output: 1 }, *stack_ptr);
        let val = snarl.get_node(pin.id.node).unwrap().downcast::<PrimitiveNode>().unwrap().primitive_type.clone();
        bytecode.push(Bytecode::Push(Value::Box(val.into_reflect())));
        *stack_ptr += 1;
        Some(snarl.out_pin(OutPinId { node: pin.id.node, output: 0 }).remotes.first().unwrap().clone())
    }
}
