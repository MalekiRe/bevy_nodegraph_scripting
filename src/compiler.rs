use crate::ui::{BreakdownType, GraphNode, NodeTrait, NodeViewer, PrimitiveType};
use crate::{Bytecode, Value};
use egui_snarl::ui::SnarlViewer;
use egui_snarl::{InPinId, NodeId, OutPinId, Snarl};
use std::collections::HashMap;

pub(crate) fn compile(node_viewer: &mut NodeViewer, snarl: &Snarl<GraphNode>) -> Vec<Bytecode> {
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

    loop {
        let Some(this) = next_pin.remotes.first() else {
            break;
        };
        let this_node = this.node;
        let function_node = snarl.get_node(this_node).unwrap().function().unwrap();
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
                &mut bytecode,
                &mut scope_map,
                &mut stack_ptr,
                &snarl,
                data_dependency,
            );
            bytecode.push(Bytecode::Dup(
                scope_map.get(&data_dependency).unwrap().clone(),
            ));
        }
        bytecode.push(Bytecode::Call(function_node.0.as_ref().unwrap().clone()));
        stack_ptr += 1;
        next_pin = snarl.out_pin(OutPinId {
            node: this.node,
            output: 0,
        });
    }
    bytecode
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
                let out_pin_id_of_breakdown_input = snarl.in_pin(InPinId {
                    node: out_pin_id.node,
                    input: 0,
                }).remotes.first().unwrap().clone();
                resolve_data_dependency(bytecode, scope_map, stack_ptr, snarl, out_pin_id_of_breakdown_input);
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
                bytecode.push(Bytecode::Push(Value::Box(Box::new(buildup.reflect_clone().unwrap()).into_partial_reflect())));
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
            _ => unreachable!(),
        }
    }
}
