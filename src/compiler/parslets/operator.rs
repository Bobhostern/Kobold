use super::PrefixParslet;
use super::InfixParslet;
use super::super::ast::Expression;
use super::super::token::{Token/*, TokenType*/};
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
        let expr = parser.parse_expression(self.precedence);
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
        let right = parser.parse_expression(prec);
        Box::new(Expression::BinaryExpression(token.get_line(), token.get_type(), left, right))
    }
    fn get_precedence(&self) -> i32 { self.precedence }
    fn dup(&self) -> Box<InfixParslet> { return Box::new(BinaryParslet::new(self.precedence, self.left_rec)) }
}
