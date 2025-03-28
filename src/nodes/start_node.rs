use crate::nodes::{GraphNode, GraphNodeMarketTrait, GraphNodeTrait};
use crate::ui::{FLOW_COLOR, NodeViewer};
use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, OutPin, Snarl};
use std::any::Any;

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
    fn show_input(
        &self,
        _node_viewer: &mut NodeViewer,
        _pin: &InPin,
        _ui: &mut Ui,
        _snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        unreachable!()
    }

    fn show_output(
        &self,
        _node_viewer: &mut NodeViewer,
        _pin: &OutPin,
        _ui: &mut Ui,
        _snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        PinInfo::triangle().with_fill(FLOW_COLOR)
    }

    fn title(&self, _graph_node: &GraphNode, _node_viewer: &mut NodeViewer) -> String {
        "Start".to_owned()
    }

    fn inputs(&self, _graph_node: &GraphNode, _node_viewer: &mut NodeViewer) -> usize {
        0
    }

    fn outputs(&self, _graph_node: &GraphNode, _node_viewer: &mut NodeViewer) -> usize {
        1
    }
}
