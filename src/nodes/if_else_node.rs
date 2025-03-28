use crate::nodes::GraphNode;
use crate::ui::{NodeTrait, NodeViewer, circle_pin, triangle_pin};
use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin, Snarl};

#[derive(Default)]
pub struct IfElseNode {}

impl NodeTrait for IfElseNode {
    fn show_input(
        node_viewer: &mut NodeViewer,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        if pin.id.input == 0 {
            triangle_pin(!pin.remotes.is_empty())
        } else {
            ui.label("Condition");
            circle_pin(!pin.remotes.is_empty())
        }
    }

    fn show_output(
        node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        if pin.id.output == 0 {
            ui.label("If");
        } else {
            ui.label("Else");
        }
        triangle_pin(!pin.remotes.is_empty())
    }

    fn show_node_menu(
        node_viewer: &mut NodeViewer,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
    }

    fn title(&self, node_viewer: &mut NodeViewer) -> String {
        "If Else".to_string()
    }

    fn inputs(&self, node_viewer: &mut NodeViewer) -> usize {
        2
    }

    fn outputs(&self, node_viewer: &mut NodeViewer) -> usize {
        2
    }
}
