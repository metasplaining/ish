use serde::{Deserialize, Serialize};

// ── Type annotations ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeAnnotation {
    Simple(String),           // e.g. "int", "string", "bool"
    List(Box<TypeAnnotation>),
    Object(Vec<(String, TypeAnnotation)>),
    Function {
        params: Vec<TypeAnnotation>,
        ret: Box<TypeAnnotation>,
    },
    Union(Vec<TypeAnnotation>),
    Optional(Box<TypeAnnotation>),      // T? — sugar for T | null
    Intersection(Vec<TypeAnnotation>),  // T & U
    Tuple(Vec<TypeAnnotation>),
    Generic {
        base: String,                           // e.g. "List"
        type_args: Vec<TypeAnnotation>,         // e.g. [Simple("int")]
    },
}
