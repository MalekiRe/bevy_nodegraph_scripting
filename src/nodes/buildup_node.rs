use crate::nodes::GraphNode;
use crate::ui::{NodeTrait, NodeViewer, circle_pin};
use bevy::prelude::Struct;
use bevy::reflect::PartialReflect;
use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin, Snarl};

#[derive(Default)]
pub struct BuildupNode {
    pub buildup: Option<Box<dyn Struct>>,
}

impl NodeTrait for BuildupNode {
    fn show_input(
        _node_viewer: &mut NodeViewer,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        let pin_info = circle_pin(!pin.remotes.is_empty());
        let buildup = snarl.get_node_mut(pin.id.node).unwrap().buildup().unwrap();
        let Some(buildup) = buildup.buildup.as_ref() else {
            return pin_info;
        };
        let s = buildup.get_represented_struct_info().unwrap();
        let name = s.field_at(pin.id.input).unwrap();
        ui.label(name.name());
        pin_info
    }

    fn show_output(
        _node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        let pin_info = circle_pin(!pin.remotes.is_empty());
        let buildup = snarl.get_node(pin.id.node).unwrap().buildup().unwrap();
        let Some(buildup) = buildup.buildup.as_ref() else {
            return pin_info;
        };
        ui.label(buildup.reflect_type_ident().unwrap());
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
        let buildup = snarl.get_node_mut(node).unwrap().buildup_mut().unwrap();
        for r#struct in &node_viewer.structs {
            if ui.button(r#struct.reflect_type_ident().unwrap()).clicked() {
                buildup.buildup.replace(
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
        "Buildup".to_string()
    }

    fn inputs(&self, node_viewer: &mut NodeViewer) -> usize {
        let Some(buildup) = self.buildup.as_ref() else {
            return 0;
        };
        buildup.field_len()
    }

    fn outputs(&self, node_viewer: &mut NodeViewer) -> usize {
        1
    }
}
