use crate::nodes::GraphNode;
use crate::ui::{NodeTrait, NodeViewer, circle_pin};
use bevy::prelude::Struct;
use bevy::reflect::PartialReflect;
use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin, Snarl};

pub struct PrimitiveNode {
    pub primitive_type: PrimitiveType,
}

impl Default for PrimitiveNode {
    fn default() -> Self {
        Self {
            primitive_type: PrimitiveType::I32(0),
        }
    }
}

pub enum PrimitiveType {
    I32(i32),
    F32(f32),
    String(String),
}

impl NodeTrait for PrimitiveNode {
    fn show_input(
        node_viewer: &mut NodeViewer,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        circle_pin(!pin.remotes.is_empty())
    }

    fn show_output(
        node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        let node = snarl
            .get_node_mut(pin.id.node)
            .unwrap()
            .primitive_mut()
            .unwrap();
        match &mut node.primitive_type {
            PrimitiveType::I32(val) => {
                ui.label("i32");
                ui.add(egui::DragValue::new(val));
            }
            PrimitiveType::F32(val) => {
                ui.label("f32");
                ui.add(egui::DragValue::new(val));
            }
            PrimitiveType::String(val) => {
                ui.label("String");
                ui.text_edit_singleline(val);
            }
        };
        circle_pin(!pin.remotes.is_empty())
    }

    fn show_node_menu(
        node_viewer: &mut NodeViewer,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
        let node = snarl.get_node_mut(node).unwrap().primitive_mut().unwrap();
        let primitive = &mut node.primitive_type;
        if ui.button("i32").clicked() {
            *primitive = PrimitiveType::I32(0);
            ui.close_menu();
        }
        if ui.button("f32").clicked() {
            *primitive = PrimitiveType::F32(0.0);
            ui.close_menu();
        }
        if ui.button("String").clicked() {
            *primitive = PrimitiveType::String("".to_string());
            ui.close_menu();
        }
    }

    fn title(&self, node_viewer: &mut NodeViewer) -> String {
        "Primitive".to_string()
    }

    fn inputs(&self, node_viewer: &mut NodeViewer) -> usize {
        0
    }

    fn outputs(&self, node_viewer: &mut NodeViewer) -> usize {
        1
    }
}
