use std::collections::HashMap;
use super::token::TokenType;

#[derive(Debug, Clone)]
pub enum Expression {
    // Prefix
    IntegerExpression(String),
    FloatExpression(String),
    PrefixExpression(i32, TokenType, Box<Expression>),

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
        bound_struct: String,
        args_or_name: Result<HashMap<String, String>, String>,
        // Store body...
        ret_value: Option<String>,
    },
    LetStatement {
        bound_name: String,
        ntype: Option<String>,
        expression: Box<Expression>
    },
}
