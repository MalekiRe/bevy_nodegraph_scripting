use crate::nodes::{GraphNode, GraphNodeMarketTrait, GraphNodeTrait};
use crate::ui::{NodeViewer, PinInfoTrait, TraitExtTuple};
use bevy::prelude::PartialReflect;
use bevy::reflect::func::args::Ownership;
use bevy::render::render_resource::binding_types::sampler;
use egui::{ComboBox, Ui};
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use std::any::Any;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct OwnershipNode(Ownership);
impl Default for OwnershipNode {
    fn default() -> Self {
        Self {
            0: Ownership::Owned,
        }
    }
}
impl Display for OwnershipNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Ownership::Owned => f.write_str("clone"),
            Ownership::Ref => f.write_str("&"),
            Ownership::Mut => f.write_str("&mut"),
        }
    }
}

impl GraphNodeTrait for OwnershipNode {
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
        if let Some(input) = self.get_data_in(pin.id, node_viewer, snarl) {
            ui.label(input.get_string_rep());
            pin.circle_pin((input.0.as_ref(), input.1))
        } else {
            PinInfo::circle()
        }
    }

    fn get_data_in(
        &self,
        in_pin: InPinId,
        node_viewer: &mut NodeViewer,
        snarl: &mut Snarl<GraphNode>,
    ) -> Option<(Box<dyn PartialReflect>, Ownership)> {
        if let Some(input) = snarl.in_pin(in_pin).remotes.first() {
            let marker = snarl.get_node(input.node).unwrap().get_marker();
            if let Some(data) = marker.get_data_out(*input, node_viewer, snarl) {
                return Some(data);
            }
        }
        None
    }
    fn get_data_out(
        &self,
        out_pin: OutPinId,
        node_viewer: &mut NodeViewer,
        snarl: &mut Snarl<GraphNode>,
    ) -> Option<(Box<dyn PartialReflect>, Ownership)> {
        let Some(data) = self.get_data_in(
            InPinId {
                node: out_pin.node,
                input: 0,
            },
            node_viewer,
            snarl,
        ) else {
            return None;
        };
        Some((
            data.0,
            snarl
                .get_node(out_pin.node)
                .unwrap()
                .get::<OwnershipNode>()
                .unwrap()
                .0
                .clone(),
        ))
    }

    fn show_output(
        &self,
        node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        if let Some(awa) = self.get_data_out(pin.id, node_viewer, snarl) {
            ui.label(awa.get_string_rep());
            pin.circle_pin((awa.0.as_ref(), awa.1))
        } else {
            PinInfo::circle()
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
        "Ownership".to_string()
    }

    fn inputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize {
        1
    }

    fn outputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize {
        1
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
        let ownership = &mut snarl
            .get_node_mut(node)
            .unwrap()
            .get_mut::<OwnershipNode>()
            .unwrap()
            .0;
        let f = format!("{}", ownership);
        ComboBox::from_label("Ownership")
            .selected_text(format!("{}", f).as_str())
            .show_ui(ui, |ui| {
                ui.selectable_value(ownership, Ownership::Owned, Ownership::Owned.to_string());
                ui.selectable_value(ownership, Ownership::Ref, Ownership::Ref.to_string());
                ui.selectable_value(ownership, Ownership::Mut, Ownership::Mut.to_string());
            });
    }

    fn has_header(&self, node_viewer: &mut NodeViewer, node: &GraphNode) -> bool {
        true
    }
}
