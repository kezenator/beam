mod context;
mod error;
mod exp;
mod inbuilt;
mod func;
mod parser;
mod value;

#[cfg(test)]
mod tests;

pub use context::Context;
pub use error::ExecError;
pub use exp::Expression;
pub use func::{ActualArgumentExpressions, ActualArguments, Function};
pub use parser::{parse, SourceLocation};
pub use value::Value;

pub type ExecResult<T> = std::result::Result<T, ExecError>;
