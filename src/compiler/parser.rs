use super::token::{Token, TokenType, TokenStream};
use super::ast::Expression;
use std::collections::HashMap;
use super::parslets::{PrefixParslet, InfixParslet};
use super::parslets::literal::{IntegerParslet, FloatParslet};
use super::parslets::operator::{BinaryParslet, PrefixOpParslet};

pub struct Parser {
    file_name: String,
    ts: TokenStream,

    prefixs: HashMap<TokenType, Box<PrefixParslet>>,
    infixs: HashMap<TokenType, Box<InfixParslet>>,

    t: Vec<Token>,
    eof: Token
}

impl Parser {
    pub fn new(fln: &str, ts: TokenStream) -> Parser {
        let mut tmp = Parser {
            ts: ts,
            file_name: fln.to_string(),
            t: vec![],
            prefixs: HashMap::new(),
            infixs: HashMap::new(),

            eof: Token::new(TokenType::EndOfFile, "").with_line(-1),
        };

        // Setup...
        tmp.prefix(TokenType::Minus, 3);
        tmp.prefix(TokenType::Plus, 3);
        tmp.register_prefix(TokenType::Integer, Box::new(IntegerParslet::new()));
        tmp.register_prefix(TokenType::Float, Box::new(FloatParslet::new()));

        tmp.binary(TokenType::Plus, 1, true);
        tmp.binary(TokenType::Minus, 1, true);
        tmp.binary(TokenType::Asterisk, 2, true);
        tmp.binary(TokenType::Backslash, 2, true);

        tmp
    }

    fn can_parse(&self) -> bool {
        self.ts.len() > 0 || self.t.len() > 0
    }

    pub fn parse_top(&mut self) -> Vec<Box<Expression>> {
        let mut ev = vec![];
        while self.can_parse() {
            let ctok = self.consume();
            let be = match ctok.get_type() {
                TokenType::Module => self.parse_module_declaration(),
                TokenType::Struct => self.parse_struct_declaration(),
                TokenType::Message => self.parse_message_declaration(),
                TokenType::Let => self.parse_let_statement(),
                _ => panic!("{}:{}: Could not parse '{}'", self.file_name, ctok.get_line(), ctok.get_string())
            };
            // Sooner or later, allow for dynamic parsing? (to allow for extensible operators)
            ev.push(be);
        }
        ev
    }

    fn parse_module_declaration(&mut self) -> Box<Expression> {
        let mut string = "".to_string();
        let mut tok = self.consume_type(TokenType::StructIdentifier);
        string = string + &tok.get_string();
        while self.match_type(TokenType::Period).is_some() {
            tok = self.consume_type(TokenType::StructIdentifier);
            string = string + "." + &tok.get_string();
        }
        Box::new(Expression::ModuleDeclaration(string))
    }

    fn parse_struct_declaration(&mut self) -> Box<Expression> {
        let sname = self.consume_type(TokenType::StructIdentifier);
        // Read type shtuff...
        self.consume_type(TokenType::LBrace);
        // Read members...
        self.consume_type(TokenType::RBrace);
        Box::new(Expression::StructDeclaration{ name: sname.get_string() })
    }

    fn parse_message_declaration(&mut self) -> Box<Expression> {
        let tstruct = self.consume_type(TokenType::StructIdentifier);
        self.consume_type(TokenType::LBracket);
        let argname: Result<HashMap<String, String>, String>;
        let mut name = self.consume_type(TokenType::Identifier);
        match self.match_type(TokenType::Colon) {
            Some(_) => {
                let mut args_map = HashMap::new();
                args_map.insert(name.get_string(), self.consume_type(TokenType::StructIdentifier).get_string());
                while self.match_type(TokenType::Comma).is_some() {
                    name = self.consume_type(TokenType::Identifier);
                    self.consume_type(TokenType::Colon);
                    args_map.insert(name.get_string(), self.consume_type(TokenType::StructIdentifier).get_string());
                }
                argname = Ok(args_map);
            }, // It's a list
            None => argname = Err(name.get_string()) // It's just a name
        };
        self.consume_type(TokenType::RBracket);
        let mut ret_type = None;
        if let Some(_) = self.match_type(TokenType::Arrow) {
            ret_type = Some(self.consume_type(TokenType::StructIdentifier).get_string());
        }
        self.consume_type(TokenType::LBrace);
        // Read block
        self.consume_type(TokenType::RBrace);
        Box::new(Expression::MessageDeclaration {
            bound_struct: tstruct.get_string(),
            args_or_name: argname,
            ret_value: ret_type
        })
    }

    fn parse_let_statement(&mut self) -> Box<Expression> {
        // Maybe introduce pattern matching...
        let name = self.consume_type(TokenType::Identifier);
        let mut name_type = None;
        if let Some(_) = self.match_type(TokenType::Colon) {
            name_type = Some(self.consume_type(TokenType::StructIdentifier).get_string());
        }
        self.consume_type(TokenType::Equal);
        let expr = self.parse_expression(0);
        Box::new(Expression::LetStatement {
            bound_name: name.get_string(),
            ntype: name_type,
            expression: expr
        })
    }

    // Expression parsing
    fn prefix(&mut self, tt: TokenType, p: i32) {
        self.register_prefix(tt, Box::new(PrefixOpParslet::new(p)))
    }

    fn register_prefix(&mut self, tt: TokenType, p: Box<PrefixParslet>) {
        self.prefixs.insert(tt, p);
    }

    fn binary(&mut self, tt: TokenType, p: i32, lr: bool) {
        self.register_infix(tt, Box::new(BinaryParslet::new(p, lr)))
    }

    fn register_infix(&mut self, tt: TokenType, p: Box<InfixParslet>) {
        self.infixs.insert(tt, p);
    }

    pub fn parse_expression(&mut self, precedence: i32) -> Box<Expression> {
        let tok = self.consume();
        let mut parslet: Option<Box<PrefixParslet>> = None;
        match self.prefixs.get(&tok.get_type()) {
            Some(plt) => parslet = Some(plt.dup()),
            None => {}
        };
        let mut left = match parslet {
            None => panic!("{}:{}: Could not parse '{}':{:?}", self.file_name, tok.get_line(), tok.get_string(), tok.get_type()),
            Some(plt) => plt.parse(self, tok)
        };
        let mut gprec = self.get_precedence();
        while precedence < gprec {
            let mut nparslet: Option<Box<InfixParslet>> = None;
            {
                let ntok = self.look_ahead(0).clone();
                match self.infixs.get(&ntok.get_type()) {
                    Some(plt) => nparslet = Some(plt.dup()),
                    None => {}
                };
            }
            left = match nparslet {
                None => left,
                Some(plt) => {
                    let ntok = self.consume();
                    plt.parse(self, left, ntok)
                }
            };
            gprec = self.get_precedence();
        }
        left
    }

    fn get_precedence(&mut self) -> i32 {
        let tt = self.look_ahead(0).get_type();
        match self.infixs.get(&tt) {
            Some(plt) => plt.get_precedence(),
            None => 0
        }
    }

    fn match_type(&mut self, expect: TokenType) -> Option<Token> {
        {
            let tok = self.look_ahead(0);
            if tok.get_type() != expect {
                return None
            }
        }
        Some(self.consume())
    }

    fn consume_type(&mut self, expect: TokenType) -> Token {
        {
            let tok = self.look_ahead(0);
            if tok.get_type() != expect {
                panic!("Token type mismatch: {:?} expected, {:?} received at line {}", expect, tok.get_type(), tok.get_line());
            }
        }
        self.consume()
    }

    fn consume(&mut self) -> Token {
        self.look_ahead(0);
        match self.t.len() > 0 {
            true => self.t.remove(0),
            false => self.eof.clone()
        }
    }

    fn look_ahead(&mut self, x: usize) -> &Token {
        let tlen = self.t.len();
        if tlen <= x {
            self.t.append(&mut self.ts.read((x + 1) - tlen));
        }

        if x < self.t.len() {
            &self.t[x]
        } else {
            &self.eof
        }
    }
}
