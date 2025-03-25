use crate::nodes::GraphNode;
use crate::ui::{NodeTrait, NodeViewer, circle_pin};
use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin, Snarl};

#[derive(Default)]
pub struct TupleBreakdownNode {
    pub length: usize,
}

impl NodeTrait for TupleBreakdownNode {
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
        ui.label(format!("{}", pin.id.output));
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
        if ui.button("+").clicked() {
            snarl
                .get_node_mut(node)
                .unwrap()
                .tuple_breakdown_mut()
                .unwrap()
                .length += 1;
            ui.close_menu();
        }
        if ui.button("-").clicked() {
            snarl
                .get_node_mut(node)
                .unwrap()
                .tuple_breakdown_mut()
                .unwrap()
                .length -= 1;
        }
    }

    fn title(&self, node_viewer: &mut NodeViewer) -> String {
        "TupleBreakdown".to_string()
    }

    fn inputs(&self, node_viewer: &mut NodeViewer) -> usize {
        1
    }

    fn outputs(&self, node_viewer: &mut NodeViewer) -> usize {
        self.length
    }
}
