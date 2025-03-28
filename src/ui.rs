/*use crate::nodes::breakdown_node::BreakdownNode;
use crate::nodes::buildup_node::BuildupNode;
use crate::nodes::for_node::ForNode;
use crate::nodes::function_node::FunctionNode;
use crate::nodes::if_else_node::IfElseNode;
use crate::nodes::primitive_node::PrimitiveNode;
use crate::nodes::query_node::QueryNode;
use crate::nodes::tuple_breakdown_node::TupleBreakdownNode;*/
use crate::nodes::GraphNode;
use crate::{Bytecode, compiler};
use bevy::DefaultPlugins;
use bevy::math::Vec3;
use bevy::prelude::{
    AppTypeRegistry, Camera2d, Commands, IntoFunction, Local, Mut, ReflectDefault, Res, ResMut,
    Resource, Startup, Struct, Transform, Update, World,
};
use bevy::reflect::func::args::Ownership;
use bevy::reflect::func::{DynamicFunction, ReturnInfo};
use bevy::reflect::{DynamicTypePath, PartialReflect, Reflect, StructInfo, Type};
use bevy_egui::{EguiContexts, EguiPlugin};
use egui::{Color32, Id, Ui};
use egui_snarl::ui::{
    NodeLayout, PinInfo, PinPlacement, SnarlPin, SnarlStyle, SnarlViewer, SnarlWidget, WireStyle,
};
use egui_snarl::{InPin, InPinId, NodeId, OutPin, Snarl};
use std::any::{Any, TypeId};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{Add, AddAssign};

pub fn uwu() {
    bevy::prelude::App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SnarlResource::default())
        .register_type::<Transform>()
        .register_type::<Vec3>()
        .register_type_data::<Vec3, ReflectDefault>()
        .register_type_data::<f32, ReflectDefault>()
        .register_type_data::<i32, ReflectDefault>()
        .register_type_data::<String, ReflectDefault>()
        .add_plugins(EguiPlugin)
        .add_systems(Update, ui_system)
        .add_systems(Startup, setup)
        .run();
}

pub struct FunctionRegistry {
    pub associated_functions: HashMap<TypeId, BTreeMap<String, DynamicFunction<'static>>>,
    pub associated_types: Vec<Box<dyn Reflect>>,
    pub freestanding_functions: BTreeMap<String, DynamicFunction<'static>>,
}

pub fn hello_world() {
    println!("hello world!");
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        let mut this = Self {
            associated_functions: Default::default(),
            associated_types: Default::default(),
            freestanding_functions: Default::default(),
        };
        fn print(string: &str) {
            println!("{}", string);
        }
        fn print_2(string: String) {
            println!("{}", string);
        }
        fn print_3(string: &String) {
            println!("{}", string);
        }
        let print = print_2
            .into_function()
            .with_name("print")
            .with_overload(print)
            .with_overload(print_3);
        this.register_freestanding(hello_world);
        this.freestanding_functions
            .insert("print".to_string(), print);
        this.register_associated(Vec3::default(), Vec3::default);
        this.register_associated(Vec3::default(), Vec3::to_string);
        this.register_associated(Vec3::default(), <Vec3 as AddAssign<Vec3>>::add_assign);
        this
    }
}

pub trait TraitExtTuple {
    fn get_string_rep(&self) -> String;
}
impl TraitExtTuple for (Box<dyn PartialReflect>, Ownership) {
    fn get_string_rep(&self) -> String {
        format!(
            "{}{}",
            match self.1 {
                Ownership::Owned => "",
                Ownership::Ref => "&",
                Ownership::Mut => "&mut ",
            },
            self.0.reflect_type_ident().unwrap()
        )
    }
}

impl FunctionRegistry {
    pub fn register_associated<T: Reflect, Marker, F>(&mut self, r#type: T, function: F)
    where
        F: IntoFunction<'static, Marker> + 'static,
    {
        let t = r#type.reflect_clone().unwrap();
        if !self
            .associated_functions
            .contains_key(&r#type.reflect_type_info().type_id())
        {
            self.associated_functions
                .insert(r#type.reflect_type_info().type_id(), BTreeMap::new());
        }
        let f = function.into_function();
        self.associated_functions
            .get_mut(&r#type.reflect_type_info().type_id())
            .unwrap()
            .insert(
                f.name()
                    .unwrap()
                    .to_string()
                    .rsplit_once("::")
                    .unwrap()
                    .1
                    .to_string(),
                f,
            );
        for t in &self.associated_types {
            if t.reflect_type_ident().unwrap() == r#type.reflect_type_ident().unwrap() {
                return;
            }
        }
        self.associated_types.push(Box::new(r#type));
    }
    pub fn register_freestanding<Marker, F>(&mut self, function: F)
    where
        F: IntoFunction<'static, Marker> + 'static,
    {
        let f = function.into_function();
        self.freestanding_functions
            .insert(f.name().unwrap().to_string(), f);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub const FLOW_COLOR: Color32 = Color32::from_rgb(0x00, 0xb0, 0x00);
//const NUMBER_COLOR: Color32 = Color32::from_rgb(0xb0, 0x00, 0x00);
const DATA_COLOR: Color32 = Color32::from_rgb(0xb0, 0x00, 0xb0);
const UNTYPED_COLOR: Color32 = Color32::from_rgb(0xb0, 0xb0, 0xb0);

pub trait PinInfoTrait {
    fn triangle_pin(&self) -> PinInfo;
    fn circle_pin(&self, data_type: (&dyn PartialReflect, Ownership)) -> PinInfo;
}
impl PinInfoTrait for InPin {
    fn triangle_pin(&self) -> PinInfo {
        let mut p = PinInfo::triangle();
        if self.remotes.is_empty() {
            p.with_fill(UNTYPED_COLOR)
        } else {
            p.with_fill(FLOW_COLOR)
        }
    }

    fn circle_pin(&self, data_type: (&dyn PartialReflect, Ownership)) -> PinInfo {
        let mut hasher = DefaultHasher::new();
        let mut d = match data_type.1 {
            Ownership::Ref => String::from("&"),
            Ownership::Mut => String::from("&mut "),
            Ownership::Owned => String::from(""),
        };
        d.push_str(data_type.0.reflect_type_path());
        d.hash(&mut hasher);
        let awa = hasher.finish();
        let (r, g, b) = split_u64_to_u8s(awa);
        PinInfo::circle().with_fill(Color32::from_rgb(r, g, b))
    }
}

pub fn split_u64_to_u8s(value: u64) -> (u8, u8, u8) {
    // Extract different parts of the u64
    // We'll take the lower bits, middle bits, and upper bits

    // Extract lower 8 bits (0-7)
    let mut first_u8 = value as u8; // This automatically wraps

    // Extract middle bits (24-31)
    let mut second_u8 = (value >> 10) as u8; // Shift and mask, then wrap

    // Extract upper bits (56-63)
    let mut third_u8 = (value >> 54) as u8; // Shift and mask, then wrap

    match third_u8 % 3 {
        0 => {
            while (first_u8 as u32 + second_u8 as u32 + third_u8 as u32) < 255 {
                first_u8 = first_u8.saturating_add(1);
            }
        }
        1 => {
            while (first_u8 as u32 + second_u8 as u32 + third_u8 as u32) < 255 {
                second_u8 = second_u8.saturating_add(1);
            }
        }
        _ => {
            while (first_u8 as u32 + second_u8 as u32 + third_u8 as u32) < 255 {
                third_u8 = third_u8.saturating_add(1);
            }
        }
    }

    (first_u8, second_u8, third_u8)
}

impl PinInfoTrait for OutPin {
    fn triangle_pin(&self) -> PinInfo {
        let mut p = PinInfo::triangle();
        if self.remotes.is_empty() {
            p.with_fill(UNTYPED_COLOR)
        } else {
            p.with_fill(FLOW_COLOR)
        }
    }

    fn circle_pin(&self, data_type: (&dyn PartialReflect, Ownership)) -> PinInfo {
        let mut hasher = DefaultHasher::new();
        let mut d = match data_type.1 {
            Ownership::Ref => String::from("&"),
            Ownership::Mut => String::from("&mut "),
            Ownership::Owned => String::from(""),
        };
        d.push_str(data_type.0.reflect_type_path());
        d.hash(&mut hasher);
        let awa = hasher.finish();
        let (r, g, b) = split_u64_to_u8s(awa);
        PinInfo::circle().with_fill(Color32::from_rgb(r, g, b))
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

fn ui_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut snarl: ResMut<SnarlResource>,
    app_type_registry: Res<AppTypeRegistry>,
) {
    use std::ops::DerefMut;
    let mut node_viewer = NodeViewer::default();
    node_viewer.registry = app_type_registry.clone();
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
    pub function_registry: FunctionRegistry,
    pub registry: AppTypeRegistry,
}

impl Default for NodeViewer {
    fn default() -> Self {
        use bevy::prelude::Reflect;
        let add: fn(Vec3, Vec3) -> Vec3 = Vec3::add;
        let vec3_print = (|vec3: Vec3| println!("{}", vec3))
            .into_function()
            .with_name("print_vec3")
            .try_with_overload(|vec3: &Vec3| println!("{}", vec3))
            .unwrap()
            .try_with_overload(|vec3: &mut Vec3| println!("{}", vec3))
            .unwrap();
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
                (|vec3: &mut Vec3| println!("{}", vec3))
                    .into_function()
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
            function_registry: Default::default(),
            registry: AppTypeRegistry::default(),
        }
    }
}

impl SnarlViewer<GraphNode> for NodeViewer {
    fn title(&mut self, node: &GraphNode) -> String {
        node.get_marker().title(node, self)
    }

    fn inputs(&mut self, node: &GraphNode) -> usize {
        node.get_marker().inputs(node, self)
    }

    fn show_body(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
        snarl
            .get_node(node)
            .unwrap()
            .get_marker()
            .show_body(self, node, inputs, outputs, ui, snarl);
    }

    fn has_body(&mut self, node: &GraphNode) -> bool {
        node.get_marker().has_body(self, node)
    }

    fn has_footer(&mut self, node: &GraphNode) -> bool {
        node.get_marker().has_footer(self, node)
    }

    fn show_footer(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
        snarl
            .get_node(node)
            .unwrap()
            .get_marker()
            .show_footer(self, node, inputs, outputs, ui, snarl);
    }

    fn show_header(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
        let marker = snarl.get_node(node).unwrap().get_marker();
        if marker.has_header(self, snarl.get_node(node).unwrap()) {
            marker.show_header(self, node, inputs, outputs, ui, snarl);
        } else {
            ui.label(self.title(&snarl[node]));
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> impl SnarlPin + 'static {
        snarl
            .get_node(pin.id.node)
            .unwrap()
            .get_marker()
            .show_input(self, pin, ui, snarl)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn outputs(&mut self, node: &GraphNode) -> usize {
        node.get_marker().outputs(node, self)
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> impl SnarlPin + 'static {
        snarl
            .get_node(pin.id.node)
            .unwrap()
            .get_marker()
            .show_output(self, pin, ui, snarl)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<GraphNode>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: egui::Pos2, ui: &mut Ui, snarl: &mut Snarl<GraphNode>) {
        ui.label("Add node");
        for node in GraphNode::list() {
            let marker = node.get_marker();
            if ui.button(marker.title(&node, self)).clicked() {
                snarl.insert_node(pos, node);
                ui.close_menu();
            }
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
        snarl
            .get_node(node)
            .unwrap()
            .get_marker()
            .show_node_menu(self, node, inputs, outputs, ui, snarl);
    }
}
