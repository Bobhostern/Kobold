use super::PrefixParslet;
use super::super::ast::Expression;
use super::super::token::Token;
use super::super::parser::Parser;

pub struct IntegerParslet;
impl IntegerParslet { pub fn new() -> IntegerParslet { IntegerParslet } }
impl PrefixParslet for IntegerParslet {
    fn parse(&self, _: &mut Parser, token: Token) -> Box<Expression> {
        Box::new(Expression::IntegerExpression(token.get_string()))
    }
    fn dup(&self) -> Box<PrefixParslet> { Box::new(IntegerParslet) }
}

pub struct FloatParslet;
impl FloatParslet { pub fn new() -> FloatParslet { FloatParslet } }
impl PrefixParslet for FloatParslet {
    #[allow(unused_variables)]
    fn parse(&self, parser: &mut Parser, token: Token) -> Box<Expression> {
        Box::new(Expression::FloatExpression(token.get_string()))
    }
    fn dup(&self) -> Box<PrefixParslet> { Box::new(FloatParslet) }
}
