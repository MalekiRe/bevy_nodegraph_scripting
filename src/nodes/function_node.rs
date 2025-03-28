use crate::nodes::{GraphNode, GraphNodeMarketTrait, GraphNodeTrait};
use crate::ui::{NodeViewer, PinInfoTrait, hello_world, split_u64_to_u8s};
use bevy::prelude::{AppTypeRegistry, IntoFunction, Reflect, ReflectDefault, Struct};
use bevy::reflect::PartialReflect;
use bevy::reflect::func::DynamicFunction;
use bevy::reflect::func::args::Ownership;
use egui::{Color32, ComboBox, Ui};
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin, OutPinId, Snarl};
use std::any::Any;
use std::hash::{DefaultHasher, Hash, Hasher};

pub struct FunctionNode {
    associated_type: Option<Box<dyn Reflect>>,
    function: DynamicFunction<'static>,
}

impl Default for FunctionNode {
    fn default() -> Self {
        Self {
            associated_type: None,
            function: hello_world.into_function(),
        }
    }
}

impl GraphNodeTrait for FunctionNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_marker(&self) -> Box<dyn GraphNodeMarketTrait> {
        Box::new(Marker)
    }
}

struct Marker;
impl GraphNodeMarketTrait for Marker {
    fn show_input(
        &self,
        node_viewer: &mut NodeViewer,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        if pin.id.input == 0 {
            return pin.triangle_pin();
        }
        let function = &snarl
            .get_node(pin.id.node)
            .unwrap()
            .get::<FunctionNode>()
            .unwrap()
            .function;
        let signature = &function.info().signatures()[0];
        let arg = signature.args().get(pin.id.input - 1).unwrap();
        let type_ident = arg.ty().short_path();
        //let name = arg.name().unwrap();
        ui.label(format!("{type_ident}"));
        let mut hasher = DefaultHasher::new();
        let str = arg.ty().type_path_table().path();
        //println!("{}", str);
        str.hash(&mut hasher);
        let awa = hasher.finish();
        let (r, g, b) = split_u64_to_u8s(awa);
        PinInfo::circle().with_fill(Color32::from_rgb(r, g, b))
    }

    fn show_output(
        &self,
        node_viewer: &mut NodeViewer,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        if pin.id.output == 0 {
            return pin.triangle_pin();
        }
        let function = &snarl
            .get_node(pin.id.node)
            .unwrap()
            .get::<FunctionNode>()
            .unwrap()
            .function;
        let signature = &function.info().signatures()[0];
        ui.label(signature.return_info().type_path_table().ident().unwrap());
        let mut hasher = DefaultHasher::new();
        let str = signature.return_info().type_path_table().path();
        //println!("{}", str);
        str.hash(&mut hasher);
        let awa = hasher.finish();
        let (r, g, b) = split_u64_to_u8s(awa);
        PinInfo::circle().with_fill(Color32::from_rgb(r, g, b))
    }

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

    fn has_header(&self, node_viewer: &mut NodeViewer, node: &GraphNode) -> bool {
        true
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
        let mut changed = false;

        let node = snarl
            .get_node_mut(node)
            .unwrap()
            .get_mut::<FunctionNode>()
            .unwrap();

        struct DynReflectWrapper(Option<Box<dyn Reflect>>);
        impl PartialEq for DynReflectWrapper {
            fn eq(&self, other: &Self) -> bool {
                match (&self.0, &other.0) {
                    (Some(a), Some(b)) => {
                        a.reflect_type_info().type_id() == b.reflect_type_info().type_id()
                    }
                    (None, None) => true,
                    _ => false,
                }
            }
        }

        let mut current_type = DynReflectWrapper(
            node.associated_type
                .as_ref()
                .map(|a| a.reflect_clone().unwrap()),
        );
        let name = match &current_type.0 {
            None => "None",
            Some(val) => val.reflect_type_ident().unwrap(),
        };
        egui::ComboBox::from_label("Type")
            .selected_text(format!("{}", name))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut current_type, DynReflectWrapper(None), "None");
                for f in &node_viewer.function_registry.associated_types {
                    ui.selectable_value(
                        &mut current_type,
                        DynReflectWrapper(Some(f.reflect_clone().unwrap())),
                        f.reflect_type_ident().unwrap(),
                    );
                }
            });
        let set_function_to_default = {
            let temp = DynReflectWrapper(
                node.associated_type
                    .as_ref()
                    .map(|a| a.reflect_clone().unwrap()),
            );
            if temp != current_type { true } else { false }
        };
        node.associated_type = current_type.0;

        struct DynFunctionWrapper(DynamicFunction<'static>);
        impl PartialEq for DynFunctionWrapper {
            fn eq(&self, other: &Self) -> bool {
                self.0.name().unwrap().eq(other.0.name().unwrap())
            }
        }
        let mut current_function = DynFunctionWrapper(node.function.clone());

        if set_function_to_default {
            changed = true;
            if let Some(selected) = &node.associated_type {
                for f in node_viewer
                    .function_registry
                    .associated_functions
                    .get(&selected.reflect_type_info().type_id())
                    .unwrap()
                {
                    current_function.0 = f.1.clone();
                }
            } else {
                for f in &node_viewer.function_registry.freestanding_functions {
                    current_function.0 = f.1.clone();
                }
            }
        }

        egui::ComboBox::from_label("Function")
            .selected_text(format!(
                "{}",
                node.function
                    .name()
                    .unwrap()
                    .rsplit_once("::")
                    .or_else(|| Some(("", node.function.name().unwrap())))
                    .unwrap()
                    .1
                    .to_string()
            ))
            .show_ui(ui, |ui| {
                if let Some(selected) = &node.associated_type {
                    for f in node_viewer
                        .function_registry
                        .associated_functions
                        .get(&selected.reflect_type_info().type_id())
                        .unwrap()
                    {
                        ui.selectable_value(
                            &mut current_function,
                            DynFunctionWrapper(f.1.clone()),
                            f.0,
                        );
                    }
                } else {
                    for f in &node_viewer.function_registry.freestanding_functions {
                        ui.selectable_value(
                            &mut current_function,
                            DynFunctionWrapper(f.1.clone()),
                            f.0,
                        );
                    }
                }
            });
        if DynFunctionWrapper(node.function.clone()) != current_function {
            changed = true;
        }
        node.function = current_function.0;
        if changed {
            for input in inputs {
                snarl.drop_inputs(input.id);
            }
            for output in outputs {
                snarl.drop_outputs(output.id);
            }
        }
    }

    fn title(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> String {
        "Function".to_string()
    }

    fn inputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize {
        graph_node
            .get::<FunctionNode>()
            .unwrap()
            .function
            .info()
            .signatures()[0]
            .arg_count()
            + 1
    }

    fn outputs(&self, graph_node: &GraphNode, node_viewer: &mut NodeViewer) -> usize {
        let ret_info = graph_node
            .get::<FunctionNode>()
            .unwrap()
            .function
            .info()
            .signatures()[0]
            .return_info()
            .clone();
        if ret_info.ty().is::<()>() { 1 } else { 2 }
    }

    fn get_data_out(
        &self,
        out_pin: OutPinId,
        node_viewer: &mut NodeViewer,
        snarl: &mut Snarl<GraphNode>,
    ) -> Option<(Box<dyn PartialReflect>, Ownership)> {
        let signature = &snarl
            .get_node(out_pin.node)
            .unwrap()
            .get::<FunctionNode>()
            .unwrap()
            .function
            .info()
            .signatures()[0];
        let arg = signature.return_info();
        let registry = node_viewer.registry.clone();
        let binding = registry.read();
        let data = binding.get(arg.type_id()).unwrap().clone();
        let default = data.data::<ReflectDefault>().unwrap().default();
        Some((default.into_partial_reflect(), arg.ownership()))
    }
}

/*
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
}*/
