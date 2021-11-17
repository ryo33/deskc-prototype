use crate::meta::{Id, WithMeta};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Handler {
    pub input: WithMeta<Type>,
    pub output: WithMeta<Type>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Number,
    String,
    Trait(Vec<WithMeta<Self>>),
    // Handlers do not need to be spanned because it has not leading token.
    Effectful {
        class: Vec<Handler>,
        ty: Box<WithMeta<Self>>,
        handlers: Vec<Handler>,
    },
    Effect {
        class: Vec<Handler>,
        handler: Box<Handler>,
    },
    Infer,
    This,
    Product(Vec<WithMeta<Self>>),
    Sum(Vec<WithMeta<Self>>),
    Function {
        parameter: Box<WithMeta<Self>>,
        body: Box<WithMeta<Self>>,
    },
    Array(Box<WithMeta<Self>>),
    Set(Box<WithMeta<Self>>),
    Let {
        definition: Box<WithMeta<Self>>,
        body: Box<WithMeta<Self>>,
    },
    Variable(Id),
    BoundedVariable {
        bound: Box<WithMeta<Self>>,
        identifier: String,
    },
}
