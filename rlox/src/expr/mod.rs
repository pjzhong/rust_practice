use crate::token::{Literal, Token};

pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
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
                Expr::Literal(val) => format!("{}", val),
                Expr::Unary(operator, right) => self.parenthesize(&operator.lexeme, &[&right]),
            }
        }
    }

    #[test]
    fn print() {
        let expression = Expr::Binary(
            Expr::Unary(
                Token::new(TokenType::Minus, "-", Literal::Nil, 1),
                Box::new(123.0.into()),
            )
            .into(),
            Token::new(TokenType::Star, "*", Literal::Nil, 1),
            Expr::Grouping(Box::new(45.67.into())).into(),
        );

        let printer = AstPrinter;

        assert_eq!("(* (- 123) (group 45.67))", printer.visit_expr(&expression))
    }
}
