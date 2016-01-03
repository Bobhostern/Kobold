use super::PrefixParslet;
use super::InfixParslet;
use super::super::ast::Expression;
use super::super::token::{Token};
use super::super::parser::Parser;

pub struct PrefixOpParslet {
    precedence: i32,
}
impl PrefixOpParslet {
    pub fn new(p: i32) -> PrefixOpParslet {
        PrefixOpParslet {
            precedence: p
        }
    }
}
impl PrefixParslet for PrefixOpParslet {
    fn parse(&self, parser: &mut Parser, token: Token) -> Box<Expression> {
        let expr = match parser.parse_expression(self.precedence) {
            Some(x) => x,
            None => panic!("Failed at prefix operator parsing at line {}", token.get_line())
        };
        Box::new(Expression::PrefixExpression(token.get_line(), token.get_type(), expr))
    }
    fn dup(&self) -> Box<PrefixParslet> { Box::new(PrefixOpParslet::new(self.precedence)) }
}

pub struct BinaryParslet {
    precedence: i32,
    left_rec: bool
}
impl BinaryParslet {
    pub fn new(p: i32, lr: bool) -> BinaryParslet {
        BinaryParslet {
            precedence: p,
            left_rec: lr
        }
    }
}
impl InfixParslet for BinaryParslet {
    fn parse(&self, parser: &mut Parser, left: Box<Expression>, token: Token) -> Box<Expression> {
        let prec = match self.left_rec {
            true => self.precedence,
            false => self.precedence - 1,
        };
        let right = match parser.parse_expression(prec) {
            Some(x) => x,
            None => panic!("Failed at binary operator parsing at line {}", token.get_line())
        };
        Box::new(Expression::BinaryExpression(token.get_line(), token.get_type(), left, right))
    }
    fn get_precedence(&self) -> i32 { self.precedence }
    fn dup(&self) -> Box<InfixParslet> { Box::new(BinaryParslet::new(self.precedence, self.left_rec)) }
}
