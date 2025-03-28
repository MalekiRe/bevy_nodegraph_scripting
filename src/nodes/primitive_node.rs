use crate::nodes::{GraphNode, GraphNodeMarketTrait, GraphNodeTrait};
use crate::ui::{NodeViewer, PinInfoTrait};
use crate::{Bytecode, Value};
use bevy::prelude::PartialReflect;
use bevy::reflect::func::args::Ownership;
use egui::{DragValue, Ui, Widget};
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Default)]
pub struct PrimitiveNode {
    pub primitive_type: PrimitiveType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrimitiveType {
    I32(i32),
    F32(f32),
    String(String),
}

impl PrimitiveType {
    pub fn as_reflect(&self) -> Box<dyn PartialReflect> {
        match self.clone() {
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

impl GraphNodeTrait for PrimitiveNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_marker(&self) -> Box<dyn GraphNodeMarketTrait> {
        Box::new(Marker)
    }
}

struct Marker;
impl GraphNodeMarketTrait for Marker {
    fn show_input(
        &self,
        _node_viewer: &mut NodeViewer,
        _pin: &InPin,
        _ui: &mut Ui,
        _snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        unreachable!()
    }

    fn has_header(&self, node_viewer: &mut NodeViewer, node: &GraphNode) -> bool {
        true
    }
    fn show_header(
        &self,
        node_viewer: &mut NodeViewer,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
        let node = snarl.get_node_mut(node).unwrap();
        let primitive_node = node.get_mut::<PrimitiveNode>().unwrap();
        egui::ComboBox::from_label("Primitive")
            .selected_text(format!("{}", primitive_node.primitive_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut primitive_node.primitive_type,
                    PrimitiveType::I32(0),
                    "i32",
                );
                ui.selectable_value(
                    &mut primitive_node.primitive_type,
                    PrimitiveType::F32(0.0),
                    "f32",
                );
                ui.selectable_value(
                    &mut primitive_node.primitive_type,
                    PrimitiveType::String(String::new()),
                    "String",
                );
            });
        ui.end_row();
    }

    fn show_output(
        &self,
        _node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        match &mut snarl
            .get_node_mut(pin.id.node)
            .unwrap()
            .get_mut::<PrimitiveNode>()
            .unwrap()
            .primitive_type
        {
            PrimitiveType::I32(val) => {
                ui.label("i32");
                DragValue::new(val).ui(ui);
                pin.circle_pin((val, Ownership::Owned))
            }
            PrimitiveType::F32(val) => {
                ui.label("f32");
                DragValue::new(val).ui(ui);
                pin.circle_pin((val, Ownership::Owned))
            }
            PrimitiveType::String(val) => {
                ui.label("String");
                ui.text_edit_singleline(val);
                pin.circle_pin((val, Ownership::Owned))
            }
        }
    }

    fn show_node_menu(
        &self,
        node_viewer: &mut NodeViewer,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
    }

    fn title(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> String {
        "Primitive".to_string()
    }

    fn inputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize {
        0
    }

    fn outputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize {
        1
    }

    fn resolve_data_dependency(
        &self,
        bytecode: &mut Vec<Bytecode>,
        scope_map: &mut HashMap<OutPinId, usize>,
        stack_ptr: &mut usize,
        snarl: &Snarl<GraphNode>,
        out_pin_id: OutPinId,
    ) {
        let primitive_type = snarl
            .get_node(out_pin_id.node)
            .unwrap()
            .get::<PrimitiveNode>()
            .unwrap()
            .primitive_type
            .clone();

        bytecode.push(Bytecode::Push(Value::Box(primitive_type.as_reflect())));
        scope_map.insert(out_pin_id, *stack_ptr);
        *stack_ptr += 1;
    }

    fn get_data_out(
        &self,
        out_pin: OutPinId,
        node_viewer: &mut NodeViewer,
        snarl: &mut Snarl<GraphNode>,
    ) -> Option<(Box<dyn PartialReflect>, Ownership)> {
        let node = snarl
            .get_node(out_pin.node)
            .unwrap()
            .get::<PrimitiveNode>()
            .unwrap();
        Some((node.primitive_type.as_reflect(), Ownership::Owned))
    }
}
