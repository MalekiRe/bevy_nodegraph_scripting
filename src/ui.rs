use crate::nodes::breakdown_node::BreakdownNode;
use crate::nodes::buildup_node::BuildupNode;
use crate::nodes::for_node::ForNode;
use crate::nodes::function_node::FunctionNode;
use crate::nodes::primitive_node::PrimitiveNode;
use crate::nodes::query_node::QueryNode;
use crate::nodes::tuple_breakdown_node::TupleBreakdownNode;
use crate::nodes::{GraphNode, GraphNodeType};
use crate::{Bytecode, compiler};
use bevy::DefaultPlugins;
use bevy::math::Vec3;
use bevy::prelude::{Camera2d, Commands, IntoFunction, Local, Mut, ResMut, Resource, Startup, Struct, Transform, Update, World};
use bevy::reflect::func::{DynamicFunction, ReturnInfo};
use bevy::reflect::{PartialReflect, StructInfo, Type};
use bevy_egui::{EguiContexts, EguiPlugin};
use egui::{Color32, Id, Ui};
use egui_snarl::ui::{
    NodeLayout, PinInfo, PinPlacement, SnarlPin, SnarlStyle, SnarlViewer, SnarlWidget, WireStyle,
};
use egui_snarl::{InPin, InPinId, NodeId, OutPin, Snarl};
use std::ops::Add;
use crate::nodes::if_else_node::IfElseNode;

pub fn uwu() {
    bevy::prelude::App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SnarlResource::default())
        .register_type::<Transform>()
        .add_plugins(EguiPlugin)
        .add_systems(Update, ui_system)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

const FLOW_COLOR: Color32 = Color32::from_rgb(0x00, 0xb0, 0x00);
//const NUMBER_COLOR: Color32 = Color32::from_rgb(0xb0, 0x00, 0x00);
const DATA_COLOR: Color32 = Color32::from_rgb(0xb0, 0x00, 0xb0);
const UNTYPED_COLOR: Color32 = Color32::from_rgb(0xb0, 0xb0, 0xb0);

pub fn triangle_pin(colored: bool) -> PinInfo {
    let mut p = PinInfo::triangle();
    if colored {
        p.with_fill(FLOW_COLOR)
    } else {
        p.with_fill(UNTYPED_COLOR)
    }
}

pub fn circle_pin(colored: bool) -> PinInfo {
    let mut p = PinInfo::circle();
    if colored {
        p.with_fill(DATA_COLOR)
    } else {
        p.with_fill(UNTYPED_COLOR)
    }
}

fn set_x(vec3: &mut Vec3, val: f32) {
    vec3.x = val;
}

fn set_float(f: &mut f32, val: f32) {
    *f = val;
}

#[derive(Resource, Default)]
pub struct SnarlResource(pub Snarl<GraphNode>);

fn compile_thing(world: &mut World) {
    world.resource_scope(|world, snarl: Mut<SnarlResource>| {
        let bytecode = compiler::compile(world, &mut Default::default(), &snarl.0);
        Bytecode::run(world, bytecode);
    });
}

fn ui_system(mut commands: Commands, mut contexts: EguiContexts, mut snarl: ResMut<SnarlResource>) {
    use std::ops::DerefMut;
    let mut node_viewer = NodeViewer::default();
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
        if ui.button("run").clicked() {
            commands.run_system_cached(compile_thing);
        }
        SnarlWidget::new()
            .id(Id::new("snarl-demo"))
            .style(default_style())
            .show(&mut snarl.0, &mut node_viewer, ui);
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
    pub(crate) structs: Vec<Box<dyn Struct>>,
    pub(crate) functions: Vec<DynamicFunction<'static>>,
    pub components: Vec<Box<dyn PartialReflect>>,
}

impl Default for NodeViewer {
    fn default() -> Self {
        use bevy::prelude::Reflect;
        let add: fn(Vec3, Vec3) -> Vec3 = Vec3::add;
        let vec3_print = (|vec3: Vec3| println!("{}", vec3))
        .into_function()
        .with_name("print_vec3")
            .try_with_overload(|vec3: &Vec3| println!("{}", vec3))
            .unwrap().try_with_overload(|vec3: &mut Vec3| println!("{}", vec3)).unwrap();
        NodeViewer {
            structs: vec![
                PartialReflect::reflect_owned(Box::new(Transform::default()))
                    .into_struct()
                    .unwrap(),
                PartialReflect::reflect_owned(Box::new(Vec3::default()))
                    .into_struct()
                    .unwrap(),
            ],
            functions: vec![
                (|vec3: &mut Vec3| println!("{}", vec3)).into_function()
                    .with_name("print_vec3_mut"),
                set_float.into_function().with_name("set_float"),
                set_x.into_function().with_name("set_x"),
                add.into_function().with_name("add"),
                (|f32: f32| println!("{}", f32))
                    .into_function()
                    .with_name("print_f32"),
                (|| println!("hello world"))
                    .into_function()
                    .with_name("hello_world"),
                (|| println!("owo")).into_function().with_name("owo"),
            ],
            components: vec![Box::new(Transform::default()).into_reflect()],
        }
    }
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

impl SnarlViewer<GraphNode> for NodeViewer {
    fn title(&mut self, node: &GraphNode) -> String {
        match node {
            GraphNode::Breakdown(val) => val.title(self),
            GraphNode::Buildup(val) => val.title(self),
            GraphNode::Primitive(val) => val.title(self),
            GraphNode::Function(val) => val.title(self),
            GraphNode::For(val) => val.title(self),
            GraphNode::Query(val) => val.title(self),
            GraphNode::TupleBreakdown(val) => val.title(self),
            GraphNode::IfElse(val) => val.title(self),
            GraphNode::Start => "Start".to_string(),
        }
    }

    fn inputs(&mut self, node: &GraphNode) -> usize {
        match node {
            GraphNode::Breakdown(val) => val.inputs(self),
            GraphNode::Buildup(val) => val.inputs(self),
            GraphNode::Primitive(val) => val.inputs(self),
            GraphNode::Function(val) => val.inputs(self),
            GraphNode::For(val) => val.inputs(self),
            GraphNode::Query(val) => val.inputs(self),
            GraphNode::TupleBreakdown(val) => val.inputs(self),
            GraphNode::IfElse(val) => val.inputs(self),
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
            GraphNodeType::For => ForNode::show_input(self, pin, ui, snarl),
            GraphNodeType::Query => QueryNode::show_input(self, pin, ui, snarl),
            GraphNodeType::TupleBreakdown => TupleBreakdownNode::show_input(self, pin, ui, snarl),
            GraphNodeType::IfElse => IfElseNode::show_input(self, pin, ui, snarl),
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
            GraphNode::For(val) => val.outputs(self),
            GraphNode::Query(val) => val.outputs(self),
            GraphNode::TupleBreakdown(val) => val.outputs(self),
            GraphNode::IfElse(val) => val.outputs(self),
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
            GraphNodeType::For => ForNode::show_output(self, pin, ui, snarl),
            GraphNodeType::Query => QueryNode::show_output(self, pin, ui, snarl),
            GraphNodeType::TupleBreakdown => TupleBreakdownNode::show_output(self, pin, ui, snarl),
            GraphNodeType::IfElse => IfElseNode::show_output(self, pin, ui, snarl),
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
        if ui.button("For").clicked() {
            snarl.insert_node(pos, GraphNode::For(ForNode::default()));
            ui.close_menu();
        }
        if ui.button("Query").clicked() {
            snarl.insert_node(pos, GraphNode::Query(QueryNode::default()));
        }
        if ui.button("TupleBreakdown").clicked() {
            snarl.insert_node(
                pos,
                GraphNode::TupleBreakdown(TupleBreakdownNode::default()),
            );
        }
        if ui.button("IfElse").clicked() {
            snarl.insert_node(
                pos,
                GraphNode::IfElse(IfElseNode::default())
            );
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
        if ui.button("Delete").clicked() {
            snarl.remove_node(node);
            ui.close_menu();
            return;
        }
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
            GraphNodeType::For => ForNode::show_node_menu(self, node, inputs, outputs, ui, snarl),
            GraphNodeType::Query => {
                QueryNode::show_node_menu(self, node, inputs, outputs, ui, snarl)
            }
            GraphNodeType::TupleBreakdown => {
                TupleBreakdownNode::show_node_menu(self, node, inputs, outputs, ui, snarl)
            }
            GraphNodeType::Start => {}
            GraphNodeType::IfElse => IfElseNode::show_node_menu(self, node, inputs, outputs, ui, snarl),
        }
    }
}
