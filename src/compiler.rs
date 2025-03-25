use crate::nodes::GraphNode;
use crate::nodes::breakdown_node::BreakdownType;
use crate::nodes::primitive_node::PrimitiveType;
use crate::nodes::query_node::QueryDataType;
use crate::ui::{NodeTrait, NodeViewer};
use crate::{Bytecode, QueryWrapper, Value};
use bevy::ecs::component::ComponentId;
use bevy::ecs::world::FilteredEntityMut;
use bevy::prelude::{QueryBuilder, World};
use egui_snarl::ui::SnarlViewer;
use egui_snarl::{InPinId, NodeId, OutPin, OutPinId, Snarl};
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub(crate) fn compile(
    world: &mut World,
    node_viewer: &mut NodeViewer,
    snarl: &Snarl<GraphNode>,
) -> Vec<Bytecode> {
    let mut start = None;
    for (i, node) in snarl.nodes().enumerate() {
        match node {
            GraphNode::Start => start = Some(i),
            _ => continue,
        }
    }
    let start = start.unwrap();
    let node = snarl.get_node(NodeId(start)).unwrap();
    match node {
        GraphNode::Start => {}
        _ => panic!(),
    }
    let mut next_pin = snarl.out_pin(OutPinId {
        node: NodeId(start),
        output: 0,
    });

    let mut scope_map: HashMap<OutPinId, usize> = HashMap::new();
    let mut bytecode: Vec<Bytecode> = vec![];
    let mut stack_ptr = 0;
    resolve_forward_pass_flow_until_finished(
        &mut bytecode,
        next_pin,
        &mut scope_map,
        &mut stack_ptr,
        snarl,
        node_viewer,
        world,
    );

    bytecode
}

fn resolve_forward_pass_flow_until_finished(
    bytecode: &mut Vec<Bytecode>,
    mut next_pin: OutPin,
    scope_map: &mut HashMap<OutPinId, usize>,
    stack_ptr: &mut usize,
    snarl: &Snarl<GraphNode>,
    node_viewer: &mut NodeViewer,
    world: &mut World,
) {
    loop {
        let Some(this) = next_pin.remotes.first() else {
            break;
        };
        let this_node = this.node;
        match snarl.get_node(this_node).unwrap() {
            GraphNode::Function(function_node) => {
                for input in 1..function_node.inputs(node_viewer) {
                    let data_dependency = snarl
                        .in_pin(InPinId {
                            node: this_node,
                            input,
                        })
                        .remotes
                        .first()
                        .unwrap()
                        .clone();
                    resolve_data_dependency(
                        bytecode,
                        scope_map,
                        stack_ptr,
                        &snarl,
                        data_dependency,
                    );
                    bytecode.push(Bytecode::Dup(
                        scope_map.get(&data_dependency).unwrap().clone(),
                    ));
                }
                println!("adding function");
                bytecode.push(Bytecode::Call(function_node.0.as_ref().unwrap().clone()));
                *stack_ptr += 1;
            }
            GraphNode::For(for_node) => {
                let data_dependency = snarl
                    .in_pin(InPinId {
                        node: this_node,
                        input: 1,
                    })
                    .remotes
                    .first()
                    .unwrap()
                    .clone();
                resolve_data_dependency(bytecode, scope_map, stack_ptr, &snarl, data_dependency);
                bytecode.push(Bytecode::NextMut);
                let position = bytecode.len();
                bytecode.push(Bytecode::Jump(0));
                let prev_stack = *stack_ptr;
                resolve_forward_pass_flow_until_finished(
                    bytecode,
                    snarl.out_pin(OutPinId {
                        node: this_node,
                        output: 0,
                    }),
                    scope_map,
                    stack_ptr,
                    snarl,
                    node_viewer,
                    world,
                );
                for _ in prev_stack..(*stack_ptr) {
                    bytecode.push(Bytecode::Pop);
                }
                bytecode.push(Bytecode::Jump(position - 1));
                let len_temp = bytecode.len();
                match bytecode.get_mut(position).unwrap() {
                    Bytecode::Jump(jump) => {
                        *jump = len_temp;
                    }
                    _ => unreachable!(),
                }
                next_pin = snarl.out_pin(OutPinId {
                    node: this.node,
                    output: 2,
                });
                *stack_ptr = prev_stack;
                continue;
            }
            GraphNode::Query(query_node) => {
                let mut map: HashMap<TypeId, ComponentId> = HashMap::default();
                for c in world.components().iter_registered() {
                    map.insert(c.type_id().unwrap(), c.id());
                }
                bytecode.push(Bytecode::Query(QueryWrapper::new(
                    query_node.querying.clone(),
                )));
            }
            _ => panic!("flow must be of flow types"),
        }
        next_pin = snarl.out_pin(OutPinId {
            node: this.node,
            output: 0,
        });
    }
}

fn resolve_data_dependency(
    bytecode: &mut Vec<Bytecode>,
    scope_map: &mut HashMap<OutPinId, usize>,
    stack_ptr: &mut usize,
    snarl: &Snarl<GraphNode>,
    out_pin_id: OutPinId,
) {
    if !scope_map.contains_key(&out_pin_id) {
        if let Some(primitive) = snarl.get_node(out_pin_id.node).unwrap().primitive() {
            match &primitive.primitive_type {
                PrimitiveType::I32(val) => {
                    bytecode.push(Bytecode::Push(Value::Box(Box::new(val.clone()))))
                }
                PrimitiveType::F32(val) => {
                    bytecode.push(Bytecode::Push(Value::Box(Box::new(val.clone()))))
                }
                PrimitiveType::String(val) => {
                    bytecode.push(Bytecode::Push(Value::Box(Box::new(val.clone()))))
                }
            }
            scope_map.insert(out_pin_id, *stack_ptr);
            *stack_ptr += 1;
            return;
        }
        match snarl.get_node(out_pin_id.node).unwrap() {
            GraphNode::Breakdown(breakdown) => {
                let out_pin_id_of_breakdown_input = snarl
                    .in_pin(InPinId {
                        node: out_pin_id.node,
                        input: 0,
                    })
                    .remotes
                    .first()
                    .unwrap()
                    .clone();
                resolve_data_dependency(
                    bytecode,
                    scope_map,
                    stack_ptr,
                    snarl,
                    out_pin_id_of_breakdown_input,
                );
                let position = *scope_map.get(&out_pin_id_of_breakdown_input).unwrap();
                match breakdown.breakdown_type {
                    BreakdownType::Owned => {
                        bytecode.push(Bytecode::DupField(position, out_pin_id.output))
                    }
                    BreakdownType::Reference => {
                        bytecode.push(Bytecode::RefField(position, out_pin_id.output))
                    }
                    BreakdownType::MutReference => {
                        bytecode.push(Bytecode::MutField(position, out_pin_id.output))
                    }
                }
                scope_map.insert(out_pin_id, *stack_ptr);
                *stack_ptr += 1;
            }
            GraphNode::Buildup(buildup) => {
                let buildup = buildup.buildup.as_ref().unwrap();
                // we resolve all the data dependencies first so that way we can just get everything
                // so in the latter for loop we can bring everything to the top
                for (i, field) in buildup.iter_fields().enumerate() {
                    let in_pin = snarl.in_pin(InPinId {
                        node: out_pin_id.node,
                        input: i,
                    });
                    let remote = in_pin.remotes.first().unwrap().clone();
                    resolve_data_dependency(bytecode, scope_map, stack_ptr, snarl, remote);
                }
                bytecode.push(Bytecode::Push(Value::Box(
                    Box::new(buildup.reflect_clone().unwrap()).into_partial_reflect(),
                )));
                for (i, field) in buildup.iter_fields().enumerate() {
                    let in_pin = snarl.in_pin(InPinId {
                        node: out_pin_id.node,
                        input: i,
                    });
                    let remote = in_pin.remotes.first().unwrap().clone();
                    bytecode.push(Bytecode::Dup(scope_map.get(&remote).unwrap().clone()));
                    bytecode.push(Bytecode::Apply(i));
                }
                scope_map.insert(out_pin_id, *stack_ptr);
                *stack_ptr += 1;
            }
            GraphNode::Query(_query) => {
                scope_map.insert(out_pin_id, *stack_ptr);
                *stack_ptr += 1;
            }
            GraphNode::For(_for) => {
                scope_map.insert(out_pin_id, *stack_ptr);
                *stack_ptr += 1;
            }
            GraphNode::TupleBreakdown(tuple_breakdown) => {
                bytecode.push(Bytecode::ListBreakdown(tuple_breakdown.length));
                scope_map.insert(out_pin_id, *stack_ptr);
                *stack_ptr += tuple_breakdown.length;
            }
            unreachable => unreachable!("{:?}", unreachable.get_type()),
        }
    }
}
