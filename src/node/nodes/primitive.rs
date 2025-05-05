use crate::flow;
use crate::node::node_traits::{InputPorts, NodeTrait, OutputPorts};
use crate::ports::data_port;
use crate::ports::data_port::InType;
use crate::type_data::TypeData;
use bevy::color::Color;
use bevy::prelude::{Entity, PartialReflect, World};
use bevy::reflect::func::args::Ownership;
use bevy::render::render_graph::Node;
use bevy_egui::egui::Ui;

#[derive(Clone, PartialEq)]
pub struct PrimitiveNode;

impl NodeTrait for PrimitiveNode {
    fn get_outputs(&self, this: Entity, world: &World) -> OutputPorts {
        println!("in get outputs");
        let Some(parent) = world.get::<flow::Parent>(this) else {
            return OutputPorts::default();
        };
        OutputPorts {
            data_ports: vec![(
                0,
                *parent,
                Some(data_port::OutType(TypeData {
                    type_info: String::new().get_represented_type_info().unwrap().clone(),
                    ownership: Ownership::Ref,
                })),
            )],
            flow_ports: vec![],
        }
    }

    fn get_inputs(&self) -> InputPorts {
        InputPorts {
            data_ports: vec![(0, InType::Ownership(Ownership::Owned))],
            flow_ports: vec![],
        }
    }

    fn color(&self) -> Color {
        Color::srgb_u8(251, 170, 47)
    }

    fn display(&mut self, ui: &mut Ui) {
        todo!()
    }

    fn name(&self) -> String {
        "Primitive".to_string()
    }
}
