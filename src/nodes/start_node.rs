use std::any::Any;
use egui::Ui;
use egui_snarl::{InPin, OutPin, Snarl};
use egui_snarl::ui::PinInfo;
use crate::nodes::{GraphNode, GraphNodeMarketTrait, GraphNodeTrait};
use crate::ui::{NodeViewer, FLOW_COLOR};

#[derive(Default)]
pub struct StartNode;

impl GraphNodeTrait for StartNode {
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
    fn show_input(&self, node_viewer: &mut NodeViewer, pin: &InPin, ui: &mut Ui, snarl: &mut Snarl<GraphNode>) -> PinInfo {
        unreachable!()
    }

    fn show_output(&self, node_viewer: &mut NodeViewer, pin: &OutPin, ui: &mut Ui, snarl: &mut Snarl<GraphNode>) -> PinInfo {
        PinInfo::triangle().with_fill(FLOW_COLOR)
    }

    fn title(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> String {
        "Start".to_owned()
    }

    fn inputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize {
        0
    }

    fn outputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize {
        1
    }
}