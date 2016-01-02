pub mod literal;
pub mod operator;

use super::ast::Expression;
use super::token::Token;
use super::parser::Parser;

pub trait PrefixParslet {
    fn parse(&self, parser: &mut Parser, token: Token) -> Box<Expression>;
    fn dup(&self) -> Box<PrefixParslet>;
}

pub trait InfixParslet {
    fn parse(&self, parser: &mut Parser, left: Box<Expression>, token: Token) -> Box<Expression>;
    fn dup(&self) -> Box<InfixParslet>;
    fn get_precedence(&self) -> i32;
}
