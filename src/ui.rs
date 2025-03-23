use crate::{compiler, Bytecode};
use bevy::DefaultPlugins;
use bevy::math::Vec3;
use bevy::prelude::{IntoFunction, Local, Struct, Transform, Update};
use bevy::reflect::func::{DynamicFunction, ReturnInfo};
use bevy::reflect::{PartialReflect, StructInfo, Type};
use bevy_egui::{EguiContexts, EguiPlugin};
use egui::{Color32, Id, Ui};
use egui_snarl::ui::{
    NodeLayout, PinInfo, PinPlacement, SnarlPin, SnarlStyle, SnarlViewer, SnarlWidget, WireStyle,
};
use egui_snarl::{InPin, InPinId, NodeId, OutPin, Snarl};
use std::ops::Add;

pub fn uwu() {
    bevy::prelude::App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Update, ui_system)
        .run();
}

const FLOW_COLOR: Color32 = Color32::from_rgb(0x00, 0xb0, 0x00);
//const NUMBER_COLOR: Color32 = Color32::from_rgb(0xb0, 0x00, 0x00);
const DATA_COLOR: Color32 = Color32::from_rgb(0xb0, 0x00, 0xb0);
const UNTYPED_COLOR: Color32 = Color32::from_rgb(0xb0, 0xb0, 0xb0);

fn set_x(vec3: &mut Vec3, val: f32) {
    vec3.x = val;
}

fn set_float(f: &mut f32, val: f32) {
    *f = val;
}

fn ui_system(mut contexts: EguiContexts, mut snarl: Local<Snarl<GraphNode>>) {
    let add: fn(Vec3, Vec3) -> Vec3 = Vec3::add;
    let mut node_viewer = NodeViewer {
        structs: vec![
            PartialReflect::reflect_owned(Box::new(Transform::default()))
                .into_struct()
                .unwrap(),
            PartialReflect::reflect_owned(Box::new(Vec3::default()))
                .into_struct()
                .unwrap(),
        ],
        functions: vec![set_float.into_function().with_name("set_float"), set_x.into_function().with_name("set_x"), add.into_function().with_name("add"), (|vec3: Vec3| {println!("{}", vec3)}).into_function().with_name("print_vec3"), (|f32: f32| {println!("{}", f32)}).into_function().with_name("print_f32")],
    };
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
        if ui.button("run").clicked() {
            let bytecode = compiler::compile(&mut node_viewer, &snarl);
            Bytecode::run(bytecode);
        }
        SnarlWidget::new()
            .id(Id::new("snarl-demo"))
            .style(default_style())
            .show(&mut snarl, &mut node_viewer, ui);
    });
}

const fn default_style() -> SnarlStyle {
    SnarlStyle {
        node_layout: Some(NodeLayout::Basic),
        pin_placement: Some(PinPlacement::Edge),
        pin_size: Some(7.0),
        node_frame: Some(egui::Frame {
            inner_margin: egui::Margin::same(8),
            outer_margin: egui::Margin {
                left: 0,
                right: 0,
                top: 0,
                bottom: 4,
            },
            corner_radius: egui::CornerRadius::same(8),
            fill: egui::Color32::from_gray(30),
            stroke: egui::Stroke::NONE,
            shadow: egui::Shadow::NONE,
        }),
        bg_frame: Some(egui::Frame {
            inner_margin: egui::Margin::ZERO,
            outer_margin: egui::Margin::same(2),
            corner_radius: egui::CornerRadius::ZERO,
            fill: egui::Color32::from_gray(40),
            stroke: egui::Stroke::NONE,
            shadow: egui::Shadow::NONE,
        }),
        ..SnarlStyle::new()
    }
}

pub struct NodeViewer {
    structs: Vec<Box<dyn Struct>>,
    functions: Vec<DynamicFunction<'static>>,
}

pub trait NodeTrait {
    fn show_input(
        node_viewer: &mut NodeViewer,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo;
    fn show_output(
        node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo;

    fn show_node_menu(
        node_viewer: &mut NodeViewer,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    );

    fn title(&self, node_viewer: &mut NodeViewer) -> String;
    fn inputs(&self, node_viewer: &mut NodeViewer) -> usize;
    fn outputs(&self, node_viewer: &mut NodeViewer) -> usize;
}

pub struct BreakdownNode {
    pub breakdown_type: BreakdownType,
    pub breakdown_thing: Option<Box<dyn Struct>>,
}
impl Default for BreakdownNode {
    fn default() -> Self {
        Self {
            breakdown_type: BreakdownType::Owned,
            breakdown_thing: None,
        }
    }
}

impl NodeTrait for BreakdownNode {
    fn show_input(
        _node_viewer: &mut NodeViewer,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        let pin_info = if pin.remotes.is_empty() {
            PinInfo::circle()
        } else {
            PinInfo::circle().with_fill(DATA_COLOR)
        };
        let GraphNode::Breakdown(breakdown) = snarl.get_node(pin.id.node).unwrap() else {
            unreachable!()
        };
        if let Some(breakdown) = breakdown.breakdown_thing.as_ref() {
            ui.label(breakdown.reflect_type_ident().unwrap());
        }
        pin_info
    }

    fn show_output(
        _node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        let pin_info = if pin.remotes.is_empty() {
            PinInfo::circle()
        } else {
            PinInfo::circle().with_fill(DATA_COLOR)
        };
        let GraphNode::Breakdown(breakdown) = snarl.get_node(pin.id.node).unwrap() else {
            unreachable!()
        };
        let Some(breakdown_struct) = breakdown.breakdown_thing.as_ref() else {
            return pin_info;
        };
        let field_name = breakdown_struct
            .get_represented_struct_info()
            .unwrap()
            .field_at(pin.id.output)
            .unwrap()
            .name();
        ui.label(breakdown.breakdown_type.format(field_name));
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
        let breakdown = snarl.get_node_mut(node).unwrap().breakdown_mut().unwrap();
        ui.label("Ownership");
        if ui.button("owned").clicked() {
            breakdown.breakdown_type = BreakdownType::Owned;
        }
        if ui.button("&").clicked() {
            breakdown.breakdown_type = BreakdownType::Reference;
        }
        if ui.button("&mut").clicked() {
            breakdown.breakdown_type = BreakdownType::MutReference;
        }
        ui.label("Structs");
        for r#struct in &node_viewer.structs {
            if ui.button(r#struct.reflect_type_ident().unwrap()).clicked() {
                breakdown.breakdown_thing.replace(
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
        "Breakdown".to_string()
    }

    fn inputs(&self, node_viewer: &mut NodeViewer) -> usize {
        1
    }

    fn outputs(&self, node_viewer: &mut NodeViewer) -> usize {
        let Some(breakdown_thing) = self.breakdown_thing.as_ref() else {
            return 0;
        };
        breakdown_thing.field_len()
    }
}

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
        let pin_info = if pin.remotes.is_empty() {
            PinInfo::circle()
        } else {
            PinInfo::circle().with_fill(DATA_COLOR)
        };
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
        let pin_info = if pin.remotes.is_empty() {
            PinInfo::circle()
        } else {
            PinInfo::circle().with_fill(DATA_COLOR)
        };
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

pub struct PrimitiveNode {
    pub primitive_type: PrimitiveType,
}

impl NodeTrait for PrimitiveNode {
    fn show_input(
        node_viewer: &mut NodeViewer,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        if pin.remotes.is_empty() {
            PinInfo::circle()
        } else {
            PinInfo::circle().with_fill(DATA_COLOR)
        }
    }

    fn show_output(
        node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        let node = snarl
            .get_node_mut(pin.id.node)
            .unwrap()
            .primitive_mut()
            .unwrap();
        match &mut node.primitive_type {
            PrimitiveType::I32(val) => {
                ui.label("i32");
                ui.add(egui::DragValue::new(val));
            }
            PrimitiveType::F32(val) => {
                ui.label("f32");
                ui.add(egui::DragValue::new(val));
            }
            PrimitiveType::String(val) => {
                ui.label("String");
                ui.text_edit_singleline(val);
            }
        };
        if pin.remotes.is_empty() {
            PinInfo::circle()
        } else {
            PinInfo::circle().with_fill(DATA_COLOR)
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
        let node = snarl.get_node_mut(node).unwrap().primitive_mut().unwrap();
        let primitive = &mut node.primitive_type;
        if ui.button("i32").clicked() {
            *primitive = PrimitiveType::I32(0);
            ui.close_menu();
        }
        if ui.button("f32").clicked() {
            *primitive = PrimitiveType::F32(0.0);
            ui.close_menu();
        }
        if ui.button("String").clicked() {
            *primitive = PrimitiveType::String("".to_string());
            ui.close_menu();
        }
    }

    fn title(&self, node_viewer: &mut NodeViewer) -> String {
        "Primitive".to_string()
    }

    fn inputs(&self, node_viewer: &mut NodeViewer) -> usize {
        0
    }

    fn outputs(&self, node_viewer: &mut NodeViewer) -> usize {
        1
    }
}

pub enum GraphNodeType {
    Breakdown,
    Buildup,
    Primitive,
    Function,
    Start,
}

pub enum GraphNode {
    Breakdown(BreakdownNode),
    Buildup(BuildupNode),
    Primitive(PrimitiveNode),
    Function(FunctionNode),
    Start,
}
impl Default for PrimitiveNode {
    fn default() -> Self {
        Self {
            primitive_type: PrimitiveType::I32(0),
        }
    }
}

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
            return if pin.remotes.is_empty() {
                PinInfo::triangle().with_fill(UNTYPED_COLOR)
            } else {
                PinInfo::triangle().with_fill(FLOW_COLOR)
            };
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
        if pin.remotes.is_empty() {
            PinInfo::circle()
        } else {
            PinInfo::circle().with_fill(DATA_COLOR)
        }
    }

    fn show_output(
        node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        if pin.id.output == 0 {
            return if pin.remotes.is_empty() {
                PinInfo::triangle().with_fill(UNTYPED_COLOR)
            } else {
                PinInfo::triangle().with_fill(FLOW_COLOR)
            };
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
        if pin.remotes.is_empty() {
            PinInfo::circle()
        } else {
            PinInfo::circle().with_fill(DATA_COLOR)
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

impl GraphNode {
    pub fn get_type(&self) -> GraphNodeType {
        match self {
            GraphNode::Breakdown(_) => GraphNodeType::Breakdown,
            GraphNode::Buildup(_) => GraphNodeType::Buildup,
            GraphNode::Primitive(_) => GraphNodeType::Primitive,
            GraphNode::Function(_) => GraphNodeType::Function,
            GraphNode::Start => GraphNodeType::Start,
        }
    }
    pub fn breakdown(&self) -> Option<&BreakdownNode> {
        match self {
            GraphNode::Breakdown(node) => Some(node),
            _ => None,
        }
    }
    pub fn buildup(&self) -> Option<&BuildupNode> {
        match self {
            GraphNode::Buildup(node) => Some(node),
            _ => None,
        }
    }
    pub fn primitive(&self) -> Option<&PrimitiveNode> {
        match self {
            GraphNode::Primitive(node) => Some(node),
            _ => None,
        }
    }
    pub fn function(&self) -> Option<&FunctionNode> {
        match self {
            GraphNode::Function(node) => Some(node),
            _ => None,
        }
    }
    pub fn breakdown_mut(&mut self) -> Option<&mut BreakdownNode> {
        match self {
            GraphNode::Breakdown(node) => Some(node),
            _ => None,
        }
    }
    pub fn buildup_mut(&mut self) -> Option<&mut BuildupNode> {
        match self {
            GraphNode::Buildup(node) => Some(node),
            _ => None,
        }
    }
    pub fn primitive_mut(&mut self) -> Option<&mut PrimitiveNode> {
        match self {
            GraphNode::Primitive(node) => Some(node),
            _ => None,
        }
    }
    pub fn function_mut(&mut self) -> Option<&mut FunctionNode> {
        match self {
            GraphNode::Function(node) => Some(node),
            _ => None,
        }
    }
}

pub enum BreakdownType {
    Owned,
    Reference,
    MutReference,
}

impl BreakdownType {
    fn format(&self, s: &str) -> String {
        match self {
            BreakdownType::Owned => s.to_string(),
            BreakdownType::Reference => format!("&{s}"),
            BreakdownType::MutReference => format!("&mut {s}"),
        }
    }
}

pub enum PrimitiveType {
    I32(i32),
    F32(f32),
    String(String),
}

impl SnarlViewer<GraphNode> for NodeViewer {
    fn title(&mut self, node: &GraphNode) -> String {
        match node {
            GraphNode::Breakdown(val) => val.title(self),
            GraphNode::Buildup(val) => val.title(self),
            GraphNode::Primitive(val) => val.title(self),
            GraphNode::Function(val) => val.title(self),
            GraphNode::Start => "Start".to_string(),
        }
    }

    fn inputs(&mut self, node: &GraphNode) -> usize {
        match node {
            GraphNode::Breakdown(val) => val.inputs(self),
            GraphNode::Buildup(val) => val.inputs(self),
            GraphNode::Primitive(val) => val.inputs(self),
            GraphNode::Function(val) => val.inputs(self),
            GraphNode::Start => 0,
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> impl SnarlPin + 'static {
        match snarl.get_node(pin.id.node).unwrap().get_type() {
            GraphNodeType::Breakdown => BreakdownNode::show_input(self, pin, ui, snarl),
            GraphNodeType::Buildup => BuildupNode::show_input(self, pin, ui, snarl),
            GraphNodeType::Primitive => PrimitiveNode::show_input(self, pin, ui, snarl),
            GraphNodeType::Function => FunctionNode::show_input(self, pin, ui, snarl),
            GraphNodeType::Start => unreachable!(),
        }
        .with_wire_style(WireStyle::AxisAligned {
            corner_radius: 10.0,
        })
    }

    fn outputs(&mut self, node: &GraphNode) -> usize {
        match node {
            GraphNode::Breakdown(val) => val.outputs(self),
            GraphNode::Buildup(val) => val.outputs(self),
            GraphNode::Primitive(val) => val.outputs(self),
            GraphNode::Function(val) => val.outputs(self),
            GraphNode::Start => 1,
        }
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> impl SnarlPin + 'static {
        match snarl.get_node(pin.id.node).unwrap().get_type() {
            GraphNodeType::Breakdown => BreakdownNode::show_output(self, pin, ui, snarl),
            GraphNodeType::Buildup => BuildupNode::show_output(self, pin, ui, snarl),
            GraphNodeType::Primitive => PrimitiveNode::show_output(self, pin, ui, snarl),
            GraphNodeType::Function => FunctionNode::show_output(self, pin, ui, snarl),
            GraphNodeType::Start => PinInfo::triangle().with_fill(FLOW_COLOR),
        }
        .with_wire_style(WireStyle::AxisAligned {
            corner_radius: 10.0,
        })
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<GraphNode>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: egui::Pos2, ui: &mut Ui, snarl: &mut Snarl<GraphNode>) {
        ui.label("Add node");
        if ui.button("Breakdown").clicked() {
            snarl.insert_node(pos, GraphNode::Breakdown(BreakdownNode::default()));
        }
        if ui.button("Buildup").clicked() {
            snarl.insert_node(pos, GraphNode::Buildup(BuildupNode::default()));
            ui.close_menu();
        }
        if ui.button("Primitive").clicked() {
            snarl.insert_node(pos, GraphNode::Primitive(PrimitiveNode::default()));
            ui.close_menu();
        }
        if ui.button("Function").clicked() {
            snarl.insert_node(pos, GraphNode::Function(FunctionNode::default()));
            ui.close_menu();
        }
        if ui.button("Start").clicked() {
            snarl.insert_node(pos, GraphNode::Start);
            ui.close_menu();
        }
    }

    fn has_node_menu(&mut self, node: &GraphNode) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
        match snarl.get_node(node).unwrap().get_type() {
            GraphNodeType::Breakdown => {
                BreakdownNode::show_node_menu(self, node, inputs, outputs, ui, snarl)
            }
            GraphNodeType::Buildup => {
                BuildupNode::show_node_menu(self, node, inputs, outputs, ui, snarl)
            }
            GraphNodeType::Primitive => {
                PrimitiveNode::show_node_menu(self, node, inputs, outputs, ui, snarl)
            }
            GraphNodeType::Function => {
                FunctionNode::show_node_menu(self, node, inputs, outputs, ui, snarl)
            }
            GraphNodeType::Start => {}
        }
    }
}
