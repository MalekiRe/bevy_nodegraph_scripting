use crate::nodes::GraphNode;
use crate::ui::{NodeTrait, NodeViewer, circle_pin, triangle_pin};
use bevy::prelude::Struct;
use bevy::reflect::PartialReflect;
use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin, Snarl};

#[derive(Default)]
pub struct ForNode {
    r#type: Option<Box<dyn Struct>>,
}

impl NodeTrait for ForNode {
    fn show_input(
        node_viewer: &mut NodeViewer,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        if pin.id.input == 0 {
            triangle_pin(!pin.remotes.is_empty())
        } else {
            circle_pin(!pin.remotes.is_empty())
        }
    }

    fn show_output(
        node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        match pin.id.output {
            0 => {
                ui.label("iteration");
                triangle_pin(!pin.remotes.is_empty())
            }
            1 => {
                match snarl
                    .get_node(pin.id.node)
                    .unwrap()
                    .r#for()
                    .unwrap()
                    .r#type
                    .as_ref()
                {
                    None => {
                        ui.label("data type");
                    }
                    Some(val) => {
                        ui.label(val.reflect_type_ident().unwrap());
                    }
                }
                circle_pin(!pin.remotes.is_empty())
            }
            2 => {
                ui.label("end");
                triangle_pin(!pin.remotes.is_empty())
            }
            _ => unreachable!(),
        }
    }

    fn show_node_menu(
        node_viewer: &mut NodeViewer,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
        /*let for_node = snarl.get_node_mut(node).unwrap().for_mut().unwrap();
        for r#struct in &node_viewer.structs {
            if ui.button(r#struct.reflect_type_ident().unwrap()).clicked() {
                for_node.r#type.replace(
                    r#struct
                        .reflect_clone()
                        .unwrap()
                        .reflect_owned()
                        .into_struct()
                        .unwrap(),
                );
            }
        }*/
    }

    fn title(&self, node_viewer: &mut NodeViewer) -> String {
        "For".to_string()
    }

    fn inputs(&self, node_viewer: &mut NodeViewer) -> usize {
        2
    }

    fn outputs(&self, node_viewer: &mut NodeViewer) -> usize {
        3
    }
}
