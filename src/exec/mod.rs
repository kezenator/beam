mod context;
mod error;
mod exp;
mod func;
mod inbuilt;
mod native;
mod parser;
mod value;

#[cfg(test)]
mod tests;

pub use context::Context;
pub use error::ExecError;
pub use exp::Expression;
pub use func::{ActualArgumentExpressions, ActualArguments, Function};
pub use native::NativeFunctionBuilder;
pub use parser::{parse, SourceLocation};
pub use value::{FromValue, Value};

pub type ExecResult<T> = std::result::Result<T, ExecError>;
