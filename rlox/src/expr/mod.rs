use crate::token::{Literal, Token};

pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Option<Literal>),
    Unary(Token, Box<Expr>),
}

///简化代码编写，不然这种包装写法太长了
impl From<f64> for Box<Expr> {
    fn from(a: f64) -> Self {
        Box::new(Expr::Literal(Some(a.into())))
    }
}

impl From<String> for Box<Expr> {
    fn from(a: String) -> Self {
        Box::new(Expr::Literal(Some(a.into())))
    }
}

impl From<bool> for Box<Expr> {
    fn from(a: bool) -> Self {
        Box::new(Expr::Literal(Some(a.into())))
    }
}

trait Visitor<T> {
    fn visit_expr(&self, expr: &Expr) -> T;
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::token::*;

    struct AstPrinter;

    impl AstPrinter {
        fn parenthesize(&self, name: &str, exprs: &[&Expr]) -> String {
            let mut builder = String::new();

            builder.push('(');
            builder.push_str(name);

            for expr in exprs {
                builder.push(' ');
                builder.push_str(&self.visit_expr(expr));
            }

            builder.push(')');
            builder
        }
    }

    impl Visitor<String> for AstPrinter {
        fn visit_expr(&self, expr: &Expr) -> String {
            match expr {
                Expr::Binary(left, operator, right) => {
                    self.parenthesize(&operator.lexeme, &[&left, &right])
                }
                Expr::Grouping(expr) => self.parenthesize("group", &[&expr]),
                Expr::Literal(val) => match val {
                    Some(val) => format!("{}", val),
                    None => "nil".to_string(),
                },
                Expr::Unary(operator, right) => self.parenthesize(&operator.lexeme, &[&right]),
            }
        }
    }

    #[test]
    fn print() {
        let expression = Expr::Binary(
            Expr::Unary(Token::new(TokenType::Minus, "-", None, 1), 123.0.into()).into(),
            Token::new(TokenType::Star, "*", None, 1),
            Expr::Grouping(45.67.into()).into(),
        );

        let printer = AstPrinter;

        assert_eq!("(* (- 123) (group 45.67))", printer.visit_expr(&expression))
    }
}
