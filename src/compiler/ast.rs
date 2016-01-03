use std::collections::HashMap;
use super::token::TokenType;

#[derive(Debug, Clone)]
pub enum Expression {
    // Prefix
    IntegerExpression(String),
    FloatExpression(String),
    PrefixExpression(i32, TokenType, Box<Expression>),
    Reference(i32, String), // Variable usage
    String(i32, String),

    // Infix and Postfix
    BinaryExpression(i32, TokenType, Box<Expression>, Box<Expression>),

    // Other
    ModuleDeclaration(String),
    StructDeclaration {
        name: String,
        // Storing members...
        // Storing parent...
        // Storing composers...
    },
    MessageDeclaration {
        line: i32,
        bound_struct: String,
        args_or_name: Result<HashMap<String, String>, String>,
        body: Vec<Box<Expression>>,
        ret_value: Option<String>,
    },
    CallDeclaration {
        line: i32,
        bound_struct: String,
        args_or_name: Result<HashMap<String, String>, String>,
        body: Vec<Box<Expression>>,
        ret_value: Option<String>,
    },
    MainDeclaration {
        line: i32,
        body: Vec<Box<Expression>>,
    },
    LetStatement {
        bound_name: String,
        ntype: Option<String>,
        expression: Box<Expression>
    },
    NewObject {
        struct_type: String,
        members: HashMap<String, Box<Expression>>
    },
    Message {
        target: String,
        args: HashMap<String, Box<Expression>>
    }
}
