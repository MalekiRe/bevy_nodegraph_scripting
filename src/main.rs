mod compiler;
mod ui;

use crate::ui::uwu;
use bevy::prelude::*;
use bevy::reflect::Enum;
use bevy::reflect::func::args::Ownership;
use bevy::reflect::func::{ArgList, DynamicFunction, Return};

fn main() {
    uwu();
}



#[derive(Debug)]
pub enum Value {
    Mut(*mut dyn PartialReflect),
    Ref(*const dyn PartialReflect),
    Box(Box<dyn PartialReflect>),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Mut(val) => Value::Mut(val.clone()),
            Value::Ref(val) => Value::Ref(val.clone()),
            Value::Box(val) => Value::Box(val.reflect_clone().unwrap()),
        }
    }
}

#[derive(Debug)]
pub enum Bytecode {
    Pop,
    Push(Value),
    Dup(usize),
    Ref(usize),
    Mut(usize),
    DupField(usize, usize),
    RefField(usize, usize),
    MutField(usize, usize),
    Call(DynamicFunction<'static>),
    Apply(usize),
}

impl Bytecode {
    pub fn run(bytecode: Vec<Bytecode>) {
        let mut stack = vec![];
        let mut potentially_garbage_data = vec![];
        for bytecode in bytecode {
            //println!("stack: {:?}", stack);
            match bytecode {
                Bytecode::Pop => {
                    stack.pop().unwrap();
                }
                Bytecode::Push(value) => stack.push(value),
                Bytecode::Dup(index) => {
                    stack.push(stack.get(index).unwrap().clone());
                }
                Bytecode::Call(function) => {
                    let mut args = ArgList::new();

                    let mut counts = 0;
                    for count in function.arg_count().iter() {
                        if counts < count {
                            counts = count;
                        }
                    }
                    let info = function.info();

                    for arg in info.signatures()[0].args() {
                        match stack.pop().unwrap() {
                            Value::Mut(mut_val) => {
                                args.push_mut(unsafe { &mut *mut_val });
                            }
                            Value::Ref(ref_val) => {
                                args.push_ref(unsafe { &*ref_val });
                            }
                            Value::Box(val) => match arg.ownership() {
                                Ownership::Ref => {
                                    potentially_garbage_data.push(Box::leak(val) as *const _ as *mut dyn PartialReflect);
                                    args.push_ref(unsafe {
                                        &*potentially_garbage_data.last().unwrap().clone()
                                    });
                                }
                                Ownership::Mut => {
                                    potentially_garbage_data.push(Box::leak(val) as *const _ as *mut dyn PartialReflect);
                                    args.push_mut(unsafe {
                                        &mut *potentially_garbage_data.last().unwrap().clone()
                                    });
                                }
                                Ownership::Owned => {
                                    args.push_boxed(val);
                                }
                            },
                        }
                    }
                    let ret = function.call(args).unwrap();
                    match ret {
                        Return::Owned(ret) => {
                            stack.push(Value::Box(ret));
                        }
                        Return::Ref(ret) => {
                            stack.push(Value::Ref(ret));
                        }
                        Return::Mut(mutable) => {
                            stack.push(Value::Mut(mutable));
                        }
                    }
                }
                Bytecode::RefField(_, _) => todo!(),
                Bytecode::MutField(stack_pos, field) => {
                    let field = match stack.get_mut(stack_pos).unwrap() {
                        Value::Mut(_) => todo!(),
                        Value::Ref(_) => unreachable!(),
                        Value::Box(val) => {
                            let mut s = val.reflect_mut().as_struct().unwrap();
                            let field = s.field_at_mut(field).unwrap() as *mut dyn PartialReflect;
                            field
                        }
                    };
                    stack.push(Value::Mut(field));
                }
                Bytecode::Ref(_) => todo!(),
                Bytecode::Mut(_) => todo!(),
                Bytecode::DupField(_, _) => {}
                Bytecode::Apply(field) => {
                    let applier = stack.pop().unwrap();
                    let mut receiver = stack.pop().unwrap();
                    match &mut receiver {
                        Value::Mut(_) => todo!(),
                        Value::Ref(_) => unreachable!(),
                        Value::Box(val) => {
                            let mut s = val.reflect_mut().as_struct().unwrap();
                            let applier = match applier {
                                Value::Mut(_) => unreachable!(),
                                Value::Ref(_) => unreachable!(),
                                Value::Box(val) => val,
                            };
                            s.field_at_mut(field).unwrap().apply(applier.as_partial_reflect());
                        }
                    }
                    stack.push(receiver);
                }
            }
        }
        for garbage_data in potentially_garbage_data {
            drop(unsafe { Box::from_raw(garbage_data) });
        }
    }
}
