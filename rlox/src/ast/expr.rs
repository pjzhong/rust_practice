use crate::token::{Literal, Token};

pub trait Visitor<T, R> {
    fn visit(&self, t: T) -> R;
}

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Variable(Token),
    Unary(Token, Box<Expr>),
    Assign(Token, Box<Expr>),
}

///简化代码编写，不然这种包装写法太长了
impl From<f64> for Expr {
    fn from(a: f64) -> Self {
        Expr::Literal(a.into())
    }
}

impl From<String> for Expr {
    fn from(a: String) -> Self {
        Expr::Literal(a.into())
    }
}

impl From<bool> for Expr {
    fn from(a: bool) -> Self {
        Expr::Literal(a.into())
    }
}

impl From<Literal> for Expr {
    fn from(l: Literal) -> Self {
        Expr::Literal(l)
    }
}

pub struct AstPrinter;

impl AstPrinter {
    fn parenthesize(&self, name: &str, exprs: &[&Expr]) -> String {
        let mut builder = String::new();

        builder.push('(');
        builder.push_str(name);

        for expr in exprs {
            builder.push(' ');
            builder.push_str(&self.visit(expr));
        }

        builder.push(')');
        builder
    }
}

impl Visitor<&Expr, String> for AstPrinter {
    fn visit(&self, expr: &Expr) -> String {
        match expr {
            Expr::Binary(left, operator, right) => {
                self.parenthesize(&operator.lexeme, &[left, right])
            }
            Expr::Grouping(expr) => self.parenthesize("group", &[expr]),
            Expr::Literal(val) => format!("{}", val),
            Expr::Unary(operator, right) => self.parenthesize(&operator.lexeme, &[right]),
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::token::*;

    #[test]
    fn print() {
        let expression = Expr::Binary(
            Expr::Unary(
                Token::new(TokenType::Minus, "-".to_string().into(), Literal::Nil, 1),
                Box::new(123.0.into()),
            )
            .into(),
            Token::new(TokenType::Star, "*".to_string().into(), Literal::Nil, 1),
            Expr::Grouping(Box::new(45.67.into())).into(),
        );

        let printer = AstPrinter;

        assert_eq!("(* (- 123) (group 45.67))", printer.visit(&expression))
    }
}
