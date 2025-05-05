use crate::flow;
use crate::node::Node;
use crate::node::shape::{NotchType, PortDrawType, ShapeInfo};
use crate::ports::{data_port, flow_port};
use bevy::color::Color;
use bevy::prelude::{Entity, World};
use bevy_egui::egui::Ui;
use std::any::Any;

#[derive(Default)]
pub struct OutputPorts {
    pub data_ports: Vec<(usize, flow::Parent, Option<data_port::OutType>)>,
    pub flow_ports: Vec<(usize, flow::Parent)>,
}

#[derive(Default)]
pub struct InputPorts {
    pub data_ports: Vec<(usize, data_port::InType)>,
    pub flow_ports: Vec<usize>,
}

pub trait NodeTrait: Any + Send + Sync + NodeTraitEq + NodeClone {
    fn get_outputs(&self, this: Entity, world: &World) -> OutputPorts;

    fn get_inputs(&self) -> InputPorts;

    fn notch_type(&self) -> NotchType {
        NotchType::TopAndBottom
    }

    fn shape_info(&self, this: Entity, world: &World) -> ShapeInfo {
        let OutputPorts {
            data_ports,
            flow_ports,
        } = self.get_outputs(this, world);
        let bottom_ports = data_ports
            .iter()
            .map(|_| PortDrawType::Data)
            .chain(flow_ports.iter().map(|_| PortDrawType::Flow))
            .collect();
        let InputPorts {
            data_ports,
            flow_ports,
        } = self.get_inputs();
        let top_ports = data_ports
            .iter()
            .map(|_| PortDrawType::Data)
            .chain(flow_ports.iter().map(|_| PortDrawType::Flow))
            .collect();
        ShapeInfo {
            top_ports,
            bottom_ports,
            notch_type: self.notch_type(),
            color: self.color(),
        }
    }

    fn color(&self) -> Color;

    fn display(&mut self, ui: &mut Ui);

    fn name(&self) -> String;
}

pub trait NodeTraitEq {
    fn eq(&self, other: &dyn NodeTrait) -> bool;
}

impl<T: PartialEq + NodeTrait> NodeTraitEq for T {
    fn eq(&self, other: &dyn NodeTrait) -> bool {
        let Some(other) = other.downcast::<T>() else {
            return false;
        };
        (other as &dyn PartialEq<T>).eq(self)
    }
}

pub trait NodeClone {
    fn clone_node(&self) -> Node;
}

impl<T: Clone + NodeTrait> NodeClone for T {
    fn clone_node(&self) -> Node {
        Node(Box::new(self.clone()))
    }
}

impl dyn NodeTrait + 'static {
    pub fn downcast<T: Any>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }
}
