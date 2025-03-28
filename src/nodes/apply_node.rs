use crate::nodes::{GraphNode, GraphNodeMarketTrait, GraphNodeTrait};
use crate::ui::{NodeViewer, PinInfoTrait, TraitExtTuple};
use bevy::reflect::func::args::Ownership;
use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, InPinId, NodeId, OutPin, Snarl};
use std::any::Any;

#[derive(Default)]
pub struct ApplyNode;

impl GraphNodeTrait for ApplyNode {
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
        if pin.id.input == 0 {
            return pin.triangle_pin();
        }

        let first_data_pin = snarl.in_pin(InPinId {
            node: pin.id.node,
            input: 1,
        });

        let Some(awa) = first_data_pin.remotes.first() else {
            return PinInfo::circle();
        };

        let Some(stuff) =
            snarl
                .get_node(awa.node)
                .unwrap()
                .get_marker()
                .get_data_out(*awa, node_viewer, snarl)
        else {
            return PinInfo::circle();
        };

        if pin.id.input == 1 {
            ui.label(stuff.get_string_rep());
            pin.circle_pin((stuff.0.as_ref(), stuff.1))
        } else {
            let stuff = (stuff.0, Ownership::Owned);
            ui.label(stuff.get_string_rep());
            pin.circle_pin((stuff.0.as_ref(), stuff.1))
        }
    }

    fn show_output(
        &self,
        node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        pin.triangle_pin()
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
        "Apply".to_string()
    }

    fn inputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize {
        3
    }

    fn outputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize {
        1
    }
}
