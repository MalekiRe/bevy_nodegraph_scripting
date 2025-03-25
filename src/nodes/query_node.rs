use crate::nodes::GraphNode;
use crate::ui::{NodeTrait, NodeViewer, circle_pin, triangle_pin};
use bevy::reflect::PartialReflect;
use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin, Snarl};
use std::fmt::{Display, Formatter, Pointer};

#[derive(Default)]
pub struct QueryNode {
    pub querying: Vec<QueryDataType>,
}

pub enum QueryDataType {
    Entity,
    Ref(Box<dyn PartialReflect>),
    Mut(Box<dyn PartialReflect>),
}
impl Clone for QueryDataType {
    fn clone(&self) -> Self {
        match self {
            QueryDataType::Entity => QueryDataType::Entity,
            QueryDataType::Ref(val) => QueryDataType::Ref(val.reflect_clone().unwrap()),
            QueryDataType::Mut(val) => QueryDataType::Mut(val.reflect_clone().unwrap()),
        }
    }
}

impl Display for QueryDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryDataType::Entity => write!(f, "Entity"),
            QueryDataType::Ref(val) => write!(f, "&{}", val.reflect_type_ident().unwrap()),
            QueryDataType::Mut(val) => write!(f, "&mut {}", val.reflect_type_ident().unwrap()),
        }
    }
}

impl NodeTrait for QueryNode {
    fn show_input(
        _node_viewer: &mut NodeViewer,
        pin: &InPin,
        _ui: &mut Ui,
        _snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        triangle_pin(!pin.remotes.is_empty())
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
        let pin_info = circle_pin(!pin.remotes.is_empty());
        let mut s = String::from("(");
        for val in &snarl
            .get_node(pin.id.node)
            .unwrap()
            .query()
            .unwrap()
            .querying
        {
            s.push_str(&format!("{}, ", val));
        }
        s.push_str(")");
        ui.label(s);

        pin_info
    }

    fn show_node_menu(
        node_viewer: &mut NodeViewer,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
        let query_node = snarl.get_node_mut(node).unwrap().query_mut().unwrap();
        if ui.button("Entity").clicked() {
            query_node.querying.push(QueryDataType::Entity);
            ui.close_menu();
        }
        for component in &node_viewer.components {
            if ui
                .button(format!("&{}", component.reflect_type_ident().unwrap()))
                .clicked()
            {
                query_node
                    .querying
                    .push(QueryDataType::Ref(component.reflect_clone().unwrap()));
                ui.close_menu();
            }
            if ui
                .button(format!("&mut {}", component.reflect_type_ident().unwrap()))
                .clicked()
            {
                query_node
                    .querying
                    .push(QueryDataType::Mut(component.reflect_clone().unwrap()));
                ui.close_menu();
            }
        }
    }

    fn title(&self, node_viewer: &mut NodeViewer) -> String {
        "Query".to_string()
    }

    fn inputs(&self, node_viewer: &mut NodeViewer) -> usize {
        1
    }

    fn outputs(&self, node_viewer: &mut NodeViewer) -> usize {
        2
    }
}
