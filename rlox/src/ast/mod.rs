mod expr;
mod stmt;

pub use expr::{AstPrinter, Expr};
pub use stmt::Stmt;

pub trait Visitor<T, R> {
    fn visit(&self, t: T) -> R;
}
