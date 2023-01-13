mod expr;
mod stmt;

pub use expr::{AstPrinter, Expr};
pub use stmt::Stmt;

pub trait Visitor<T, R> {
    fn visit(&mut self, t: T) -> R;
}
