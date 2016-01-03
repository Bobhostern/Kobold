use super::PrefixParslet;
use super::super::ast::Expression;
use super::super::token::Token;
use super::super::parser::Parser;

pub struct CallParslet;
impl CallParslet { pub fn new() -> CallParslet { CallParslet } }
impl PrefixParslet for CallParslet {
    fn parse(&self, parser: &mut Parser, _: Token) -> Box<Expression> {
        parser.parse_message_call(false)
    }
    fn dup(&self) -> Box<PrefixParslet> { Box::new(CallParslet) }
}
