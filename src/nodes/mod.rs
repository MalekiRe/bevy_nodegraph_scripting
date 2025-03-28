use crate::Bytecode;
/*use crate::nodes::breakdown_node::BreakdownNode;
use crate::nodes::buildup_node::BuildupNode;
use crate::nodes::for_node::ForNode;
use crate::nodes::function_node::FunctionNode;
use crate::nodes::if_else_node::IfElseNode;
use crate::nodes::primitive_node::PrimitiveNode;
use crate::nodes::query_node::QueryNode;
use crate::nodes::tuple_breakdown_node::TupleBreakdownNode;*/
use crate::nodes::apply_node::ApplyNode;
use crate::nodes::breakdown_node::BreakdownNode;
use crate::nodes::function_node::FunctionNode;
use crate::nodes::ownership_node::OwnershipNode;
use crate::nodes::primitive_node::PrimitiveNode;
use crate::nodes::start_node::StartNode;
use crate::ui::NodeViewer;
use bevy::prelude::{Deref, DerefMut, PartialReflect, World};
use bevy::reflect::func::args::Ownership;
use egui::Ui;
use egui_snarl::ui::{PinInfo, SnarlViewer};
use egui_snarl::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use std::any::Any;
use std::collections::HashMap;
/*pub mod breakdown_node;
pub mod buildup_node;
pub mod for_node;
pub mod function_node;
pub mod if_else_node;
pub mod primitive_node;
pub mod query_node;
pub mod tuple_breakdown_node;*/

pub mod apply_node;
pub mod breakdown_node;
pub mod function_node;
pub mod ownership_node;
pub mod primitive_node;
pub mod start_node;

#[derive(Deref, DerefMut)]
pub struct GraphNode(pub Box<dyn GraphNodeTrait>);

unsafe impl Send for GraphNode {}
unsafe impl Sync for GraphNode {}

impl GraphNode {
    pub fn list() -> Vec<GraphNode> {
        vec![
            GraphNode(Box::new(StartNode::default())),
            GraphNode(Box::new(PrimitiveNode::default())),
            GraphNode(Box::new(FunctionNode::default())),
            GraphNode(Box::new(OwnershipNode::default())),
            GraphNode(Box::new(ApplyNode::default())),
            GraphNode(Box::new(BreakdownNode::default())),
        ]
    }
}

pub trait GraphNodeTrait: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_marker(&self) -> Box<dyn GraphNodeMarketTrait>;
}

impl dyn GraphNodeTrait {
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
}

pub trait GraphNodeMarketTrait {
    fn show_input(
        &self,
        node_viewer: &mut NodeViewer,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo;
    fn show_output(
        &self,
        node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo;

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

    fn title(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> String;
    fn inputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize;
    fn outputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize;

    fn show_body(
        &self,
        node_viewer: &mut NodeViewer,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
    }

    fn show_footer(
        &self,
        node_viewer: &mut NodeViewer,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
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
    }

    fn has_body(&self, node_viewer: &mut NodeViewer, node: &GraphNode) -> bool {
        false
    }

    fn has_footer(&self, node_viewer: &mut NodeViewer, node: &GraphNode) -> bool {
        false
    }

    fn has_header(&self, node_viewer: &mut NodeViewer, node: &GraphNode) -> bool {
        false
    }

    fn resolve_data_dependency(
        &self,
        snarl: &Snarl<GraphNode>,
        bytecode: &mut Vec<Bytecode>,
        scope_map: &mut HashMap<OutPinId, usize>,
        stack_ptr: &mut usize,
        pin: OutPin,
    ) {
        unimplemented!()
    }
    fn resolve_forward_pass_flow_until_finished(
        &self,
        snarl: &Snarl<GraphNode>,
        bytecode: &mut Vec<Bytecode>,
        scope_map: &mut HashMap<OutPinId, usize>,
        stack_ptr: &mut usize,
        node_viewer: &mut NodeViewer,
        world: &mut World,
        pin: InPin,
    ) -> Option<InPinId> {
        unimplemented!()
    }

    fn get_data_in(
        &self,
        in_pin: InPinId,
        node_viewer: &mut NodeViewer,
        snarl: &mut Snarl<GraphNode>,
    ) -> Option<(Box<dyn PartialReflect>, Ownership)> {
        None
    }
    fn get_data_out(
        &self,
        out_pin: OutPinId,
        node_viewer: &mut NodeViewer,
        snarl: &mut Snarl<GraphNode>,
    ) -> Option<(Box<dyn PartialReflect>, Ownership)> {
        None
    }
}

pub trait GraphCompileExt {
    fn resolve_data_dependency(
        &self,
        bytecode: &mut Vec<Bytecode>,
        scope_map: &mut HashMap<OutPinId, usize>,
        stack_ptr: &mut usize,
        pin: OutPinId,
    ) -> usize;
    fn resolve_forward_pass_flow_until_finished(
        &self,
        bytecode: &mut Vec<Bytecode>,
        scope_map: &mut HashMap<OutPinId, usize>,
        stack_ptr: &mut usize,
        node_viewer: &mut NodeViewer,
        world: &mut World,
        pin: InPinId,
    );
}
impl GraphCompileExt for Snarl<GraphNode> {
    fn resolve_data_dependency(
        &self,
        bytecode: &mut Vec<Bytecode>,
        scope_map: &mut HashMap<OutPinId, usize>,
        stack_ptr: &mut usize,
        pin: OutPinId,
    ) -> usize {
        let marker = self.get_node(pin.node).unwrap().get_marker();
        marker.resolve_data_dependency(self, bytecode, scope_map, stack_ptr, self.out_pin(pin));
        *scope_map.get(&pin).unwrap()
    }

    fn resolve_forward_pass_flow_until_finished(
        &self,
        bytecode: &mut Vec<Bytecode>,
        scope_map: &mut HashMap<OutPinId, usize>,
        stack_ptr: &mut usize,
        node_viewer: &mut NodeViewer,
        world: &mut World,
        pin: InPinId,
    ) {
        let mut opt_pin = Some(pin);
        while let Some(pin) = opt_pin {
            let marker = self.get_node(pin.node).unwrap().get_marker();
            opt_pin = marker.resolve_forward_pass_flow_until_finished(
                self,
                bytecode,
                scope_map,
                stack_ptr,
                node_viewer,
                world,
                self.in_pin(pin),
            );
        }
    }
}
