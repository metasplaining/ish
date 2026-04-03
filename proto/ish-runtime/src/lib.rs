// ish-runtime: Runtime types shared between the interpreter and compiled packages.

pub mod error;
pub mod value;

pub use error::{ErrorCode, RuntimeError};
pub use value::{
    empty_object, new_compiled_function, new_list, new_object, take_unawaited_future_count,
    FunctionRef, FutureRef, IshFunction, ListRef, ObjectRef, Shim, Value,
};
