use crate::nodes::GraphNode;
use crate::ui::{NodeTrait, NodeViewer, circle_pin, triangle_pin};
use bevy::prelude::Struct;
use bevy::reflect::PartialReflect;
use bevy::reflect::func::DynamicFunction;
use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin, Snarl};

#[derive(Default)]
pub struct FunctionNode(pub Option<DynamicFunction<'static>>);

impl NodeTrait for FunctionNode {
    fn show_input(
        node_viewer: &mut NodeViewer,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        if pin.id.input == 0 {
            return triangle_pin(!pin.remotes.is_empty());
        }
        let node = snarl
            .get_node_mut(pin.id.node)
            .unwrap()
            .function_mut()
            .unwrap();
        if let Some(function) = &node.0 {
            let signature = &function.info().signatures()[0];
            let arg = signature.args().get(pin.id.input - 1).unwrap();
            let type_ident = arg.ty().short_path();
            /*let name = arg.name().unwrap();*/
            ui.label(format!("{type_ident}"));
        }
        circle_pin(!pin.remotes.is_empty())
    }

    fn show_output(
        node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        if pin.id.output == 0 {
            return triangle_pin(!pin.remotes.is_empty());
        }
        let node = snarl
            .get_node_mut(pin.id.node)
            .unwrap()
            .function_mut()
            .unwrap();
        if let Some(function) = &node.0 {
            let signature = &function.info().signatures()[0];
            ui.label(signature.return_info().type_path_table().ident().unwrap());
        }
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
        for function in &node_viewer.functions {
            if ui.button(function.name().unwrap().to_string()).clicked() {
                let node = snarl.get_node_mut(node).unwrap().function_mut().unwrap();
                node.0.replace(function.clone());
            }
        }
    }

    fn title(&self, node_viewer: &mut NodeViewer) -> String {
        if let Some(function) = self.0.as_ref() {
            if let Some(name) = function.name() {
                return name.to_string();
            }
        }
        "Function".to_string()
    }

    fn inputs(&self, node_viewer: &mut NodeViewer) -> usize {
        let Some(function) = self.0.as_ref() else {
            return 1;
        };
        function.info().signatures()[0].arg_count() + 1
    }

    fn outputs(&self, node_viewer: &mut NodeViewer) -> usize {
        let Some(function) = self.0.as_ref() else {
            return 1;
        };
        let ret_info = function.info().signatures()[0].return_info().clone();
        if ret_info.ty().is::<()>() { 1 } else { 2 }
    }
}
