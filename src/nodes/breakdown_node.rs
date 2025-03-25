use crate::nodes::GraphNode;
use crate::ui::{NodeTrait, NodeViewer, circle_pin};
use bevy::prelude::Struct;
use bevy::reflect::PartialReflect;
use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin, Snarl};

pub struct BreakdownNode {
    pub breakdown_type: BreakdownType,
    pub breakdown_thing: Option<Box<dyn Struct>>,
}

pub enum BreakdownType {
    Owned,
    Reference,
    MutReference,
}

impl BreakdownType {
    fn format(&self, s: &str) -> String {
        match self {
            BreakdownType::Owned => s.to_string(),
            BreakdownType::Reference => format!("&{s}"),
            BreakdownType::MutReference => format!("&mut {s}"),
        }
    }
}

impl Default for BreakdownNode {
    fn default() -> Self {
        Self {
            breakdown_type: BreakdownType::Owned,
            breakdown_thing: None,
        }
    }
}

impl NodeTrait for BreakdownNode {
    fn show_input(
        _node_viewer: &mut NodeViewer,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        let GraphNode::Breakdown(breakdown) = snarl.get_node(pin.id.node).unwrap() else {
            unreachable!()
        };
        if let Some(breakdown) = breakdown.breakdown_thing.as_ref() {
            ui.label(breakdown.reflect_type_ident().unwrap());
        }
        circle_pin(!pin.remotes.is_empty())
    }

    fn show_output(
        _node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        let pin_info = circle_pin(!pin.remotes.is_empty());
        let GraphNode::Breakdown(breakdown) = snarl.get_node(pin.id.node).unwrap() else {
            unreachable!()
        };
        let Some(breakdown_struct) = breakdown.breakdown_thing.as_ref() else {
            return pin_info;
        };
        let field_name = breakdown_struct
            .get_represented_struct_info()
            .unwrap()
            .field_at(pin.id.output)
            .unwrap()
            .name();
        ui.label(breakdown.breakdown_type.format(field_name));
        pin_info
    }

    fn show_node_menu(
        node_viewer: &mut NodeViewer,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
        let breakdown = snarl.get_node_mut(node).unwrap().breakdown_mut().unwrap();
        ui.label("Ownership");
        if ui.button("owned").clicked() {
            breakdown.breakdown_type = BreakdownType::Owned;
        }
        if ui.button("&").clicked() {
            breakdown.breakdown_type = BreakdownType::Reference;
        }
        if ui.button("&mut").clicked() {
            breakdown.breakdown_type = BreakdownType::MutReference;
        }
        ui.label("Structs");
        for r#struct in &node_viewer.structs {
            if ui.button(r#struct.reflect_type_ident().unwrap()).clicked() {
                breakdown.breakdown_thing.replace(
                    r#struct
                        .reflect_clone()
                        .unwrap()
                        .reflect_owned()
                        .into_struct()
                        .unwrap(),
                );
            }
        }
    }

    fn title(&self, node_viewer: &mut NodeViewer) -> String {
        "Breakdown".to_string()
    }

    fn inputs(&self, node_viewer: &mut NodeViewer) -> usize {
        1
    }

    fn outputs(&self, node_viewer: &mut NodeViewer) -> usize {
        let Some(breakdown_thing) = self.breakdown_thing.as_ref() else {
            return 0;
        };
        breakdown_thing.field_len()
    }
}
