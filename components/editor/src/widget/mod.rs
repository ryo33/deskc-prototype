pub mod backend;
pub mod operation;

use bevy_math::Vec2;
use language::code::{node::NumberLiteral, path::NodePath};
use protocol::card_id::CardId;

use crate::physics::shape::Shape;

#[derive(Clone, Debug)]
pub struct Target {
    pub card_id: CardId,
    pub node_path: NodePath,
}

#[derive(Clone, Debug)]
pub struct Widget {
    pub id: String,
    pub position: Vec2,
    pub shape: Option<Shape>,
    pub component: Component,
}

#[derive(Clone, Debug)]
pub enum Component {
    Unit,
    InputString {
        value: String,
        target: Target,
    },
    InputNumber {
        value: NumberLiteral,
        target: Target,
    },
}
