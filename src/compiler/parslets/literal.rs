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
    fn parse(&self, _: &mut Parser, token: Token) -> Box<Expression> {
        Box::new(Expression::FloatExpression(token.get_string()))
    }
    fn dup(&self) -> Box<PrefixParslet> { Box::new(FloatParslet) }
}

pub struct ReferenceParslet;
impl ReferenceParslet { pub fn new() -> ReferenceParslet { ReferenceParslet } }
impl PrefixParslet for ReferenceParslet {
    fn parse(&self, _: &mut Parser, token: Token) -> Box<Expression> {
        Box::new(Expression::Reference(token.get_line(), token.get_string()))
    }
    fn dup(&self) -> Box<PrefixParslet> { Box::new(ReferenceParslet) }
}

pub struct StringParslet;
impl StringParslet { pub fn new() -> StringParslet { StringParslet } }
impl PrefixParslet for StringParslet {
    fn parse(&self, _: &mut Parser, token: Token) -> Box<Expression> {
        Box::new(Expression::String(token.get_line(), token.get_string()))
    }
    fn dup(&self) -> Box<PrefixParslet> { Box::new(StringParslet) }
}
