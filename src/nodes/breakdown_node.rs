use crate::nodes::{GraphNode, GraphNodeMarketTrait, GraphNodeTrait};
use crate::ui::{NodeViewer, PinInfoTrait, TraitExtTuple};
use bevy::prelude::Struct;
use bevy::reflect::func::args::Ownership;
use bevy::reflect::{DynamicTypePath, PartialReflect};
use egui::{ComboBox, Ui};
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use std::any::Any;
use std::fmt::{Display, Formatter};

pub struct BreakdownNode {
    pub breakdown_type: BreakdownType,
    pub num_fields: usize,
}

#[derive(Debug, PartialEq)]
pub enum BreakdownType {
    Owned,
    Reference,
    MutReference,
}

impl Display for BreakdownType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BreakdownType::Owned => f.write_str("owned"),
            BreakdownType::Reference => f.write_str("&"),
            BreakdownType::MutReference => f.write_str("&mut"),
        }
    }
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
            num_fields: 0,
        }
    }
}

impl GraphNodeTrait for BreakdownNode {
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
        node_viewer: &mut NodeViewer,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        snarl
            .get_node_mut(pin.id.node)
            .unwrap()
            .get_mut::<BreakdownNode>()
            .unwrap()
            .num_fields = 0;
        let Some(awa) = self.get_data_in(pin.id, node_viewer, snarl) else {
            return PinInfo::circle();
        };
        snarl
            .get_node_mut(pin.id.node)
            .unwrap()
            .get_mut::<BreakdownNode>()
            .unwrap()
            .num_fields = awa.0.reflect_ref().as_struct().unwrap().field_len();
        ui.label(format!("{}", awa.get_string_rep()));
        pin.circle_pin((awa.0.as_ref(), awa.1))
    }

    fn show_output(
        &self,
        node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        let Some(awa) = self.get_data_out(pin.id, node_viewer, snarl) else {
            return PinInfo::circle();
        };
        ui.label(format!("{}", awa.get_string_rep()));
        pin.circle_pin((awa.0.as_ref(), awa.1))
    }

    fn title(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> String {
        "Breakdown".to_string()
    }

    fn inputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize {
        1
    }

    fn outputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize {
        graph_node.get::<BreakdownNode>().unwrap().num_fields
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
        let node = snarl
            .get_node_mut(node)
            .unwrap()
            .get_mut::<BreakdownNode>()
            .unwrap();
        ComboBox::from_label("Breakdown")
            .selected_text(format!("{}", node.breakdown_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut node.breakdown_type, BreakdownType::Owned, "Owned");
                ui.selectable_value(&mut node.breakdown_type, BreakdownType::Reference, "&");
                ui.selectable_value(
                    &mut node.breakdown_type,
                    BreakdownType::MutReference,
                    "&mut",
                );
            });
    }

    fn has_header(&self, node_viewer: &mut NodeViewer, node: &GraphNode) -> bool {
        true
    }

    fn get_data_in(
        &self,
        in_pin: InPinId,
        node_viewer: &mut NodeViewer,
        snarl: &mut Snarl<GraphNode>,
    ) -> Option<(Box<dyn PartialReflect>, Ownership)> {
        let binding = snarl.in_pin(in_pin);
        let Some(remote) = binding.remotes.first() else {
            return None;
        };
        snarl
            .get_node(remote.node)
            .unwrap()
            .get_marker()
            .get_data_out(*remote, node_viewer, snarl)
    }

    fn get_data_out(
        &self,
        out_pin: OutPinId,
        node_viewer: &mut NodeViewer,
        snarl: &mut Snarl<GraphNode>,
    ) -> Option<(Box<dyn PartialReflect>, Ownership)> {
        let pin = snarl.out_pin(out_pin);
        let Some(((breakdown_struct, ownership))) = self.get_data_in(
            InPinId {
                node: pin.id.node,
                input: 0,
            },
            node_viewer,
            snarl,
        ) else {
            return None;
        };
        let Ok(awa) = breakdown_struct
            .reflect_clone()
            .unwrap()
            .reflect_owned()
            .into_struct()
        else {
            return None;
        };
        let field = awa.field_at(pin.id.output).unwrap();
        let field_type = field
            .reflect_type_path()
            .rsplit_once("::")
            .or_else(|| Some(("", field.reflect_type_path())))
            .unwrap()
            .1;
        let field_name = awa
            .get_represented_struct_info()
            .unwrap()
            .field_at(pin.id.output)
            .unwrap()
            .name();
        let breakdown_type = &snarl
            .get_node(pin.id.node)
            .unwrap()
            .get::<BreakdownNode>()
            .unwrap()
            .breakdown_type;
        let field_type = match breakdown_type {
            BreakdownType::Reference => format!("&{field_type}"),
            BreakdownType::MutReference => format!("&mut {field_type}"),
            BreakdownType::Owned => format!("{field_type}"),
        };
        let ownership = match breakdown_type {
            BreakdownType::Owned => Ownership::Owned,
            BreakdownType::Reference => Ownership::Ref,
            BreakdownType::MutReference => Ownership::Mut,
        };
        Some((field.reflect_clone().unwrap(), ownership))
    }
}
/*
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
*/
