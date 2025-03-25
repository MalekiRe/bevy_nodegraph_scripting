use crate::nodes::breakdown_node::BreakdownNode;
use crate::nodes::buildup_node::BuildupNode;
use crate::nodes::for_node::ForNode;
use crate::nodes::function_node::FunctionNode;
use crate::nodes::if_else_node::IfElseNode;
use crate::nodes::primitive_node::PrimitiveNode;
use crate::nodes::query_node::QueryNode;
use crate::nodes::tuple_breakdown_node::TupleBreakdownNode;

pub mod breakdown_node;
pub mod buildup_node;
pub mod for_node;
pub mod function_node;
pub mod primitive_node;
pub mod query_node;
pub mod tuple_breakdown_node;
pub mod if_else_node;

#[derive(Debug)]
pub enum GraphNodeType {
    Breakdown,
    Buildup,
    Primitive,
    Function,
    Start,
    For,
    Query,
    TupleBreakdown,
    IfElse,
}

pub enum GraphNode {
    Breakdown(BreakdownNode),
    Buildup(BuildupNode),
    Primitive(PrimitiveNode),
    Function(FunctionNode),
    For(ForNode),
    Query(QueryNode),
    TupleBreakdown(TupleBreakdownNode),
    IfElse(IfElseNode),
    Start,
}

impl GraphNode {
    pub fn get_type(&self) -> GraphNodeType {
        match self {
            GraphNode::Breakdown(_) => GraphNodeType::Breakdown,
            GraphNode::Buildup(_) => GraphNodeType::Buildup,
            GraphNode::Primitive(_) => GraphNodeType::Primitive,
            GraphNode::Function(_) => GraphNodeType::Function,
            GraphNode::Start => GraphNodeType::Start,
            GraphNode::For(_) => GraphNodeType::For,
            GraphNode::Query(_) => GraphNodeType::Query,
            GraphNode::TupleBreakdown(_) => GraphNodeType::TupleBreakdown,
            GraphNode::IfElse(_) => GraphNodeType::IfElse,
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
    pub fn r#for(&self) -> Option<&ForNode> {
        match self {
            GraphNode::For(node) => Some(node),
            _ => None,
        }
    }
    pub fn query(&self) -> Option<&QueryNode> {
        match self {
            GraphNode::Query(node) => Some(node),
            _ => None,
        }
    }
    pub fn tuple_breakdown(&self) -> Option<&TupleBreakdownNode> {
        match self {
            GraphNode::TupleBreakdown(node) => Some(node),
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
    pub fn for_mut(&mut self) -> Option<&mut ForNode> {
        match self {
            GraphNode::For(node) => Some(node),
            _ => None,
        }
    }
    pub fn query_mut(&mut self) -> Option<&mut QueryNode> {
        match self {
            GraphNode::Query(node) => Some(node),
            _ => None,
        }
    }
    pub fn tuple_breakdown_mut(&mut self) -> Option<&mut TupleBreakdownNode> {
        match self {
            GraphNode::TupleBreakdown(node) => Some(node),
            _ => None,
        }
    }
}
