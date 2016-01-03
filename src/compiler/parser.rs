use super::token::{Token, TokenType, TokenStream};
use super::ast::Expression;
use std::collections::HashMap;
use super::parslets::prec;
use super::parslets::{PrefixParslet, InfixParslet};
use super::parslets::literal::{IntegerParslet, FloatParslet, ReferenceParslet, StringParslet};
use super::parslets::operator::{BinaryParslet, PrefixOpParslet};
use super::parslets::call::{CallParslet};

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
        Parser {
            ts: ts,
            file_name: fln.to_string(),
            t: vec![],
            prefixs: HashMap::new(),
            infixs: HashMap::new(),

            eof: Token::new(TokenType::EndOfFile, "").with_line(-1),
        }
    }

    pub fn setup_defaults(&mut self) {
        self.prefix(TokenType::Minus, prec::SIGN);
        self.prefix(TokenType::Plus, prec::SIGN);
        self.register_prefix(TokenType::Integer, Box::new(IntegerParslet::new()));
        self.register_prefix(TokenType::Float, Box::new(FloatParslet::new()));
        self.register_prefix(TokenType::Identifier, Box::new(ReferenceParslet::new()));
        self.register_prefix(TokenType::CString, Box::new(StringParslet::new()));
        self.register_prefix(TokenType::LBracket, Box::new(CallParslet::new()));

        self.binary(TokenType::Plus, prec::ADDSUB, true);
        self.binary(TokenType::Minus, prec::ADDSUB, true);
        self.binary(TokenType::Asterisk, prec::MULDIV, true);
        self.binary(TokenType::Backslash, prec::MULDIV, true);
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
                // TokenType::Let => self.parse_let_statement(), Do we allow module-wide lets?
                TokenType::Call => self.parse_call_declaration(),
                TokenType::Main => self.parse_main_declaration(),
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

    fn parse_idtype_and_add(&mut self, hshmap: &mut HashMap<String, String>) -> (Token, bool) {
        let name = self.consume_type(TokenType::Identifier);
        self.consume_type(TokenType::Colon);
        let typ = self.consume_type(TokenType::StructIdentifier);
        let exists = hshmap.contains_key(&name.get_string());
        hshmap.entry(name.get_string()).or_insert(typ.get_string());
        (name, exists)
    }

    fn parse_struct_declaration(&mut self) -> Box<Expression> {
        let sname = self.consume_type(TokenType::StructIdentifier);
        // Read type shtuff...
        self.consume_type(TokenType::LBrace);
        // Read members...
        let mut members = HashMap::new();
        while self.look_ahead(0).get_type() == TokenType::Identifier {
            let (name_tok, pe) = self.parse_idtype_and_add(&mut members);
            if pe {
                panic!("Cannot redeclare a member: {} at line {}", name_tok.get_string(), name_tok.get_line());
            }
            self.match_type(TokenType::Comma);
        }
        self.consume_type(TokenType::RBrace);
        Box::new(Expression::StructDeclaration{ name: sname.get_string() })
    }

    fn parse_message_like_declaration(&mut self) -> (i32, String, Result<HashMap<String, String>, String>, Vec<Box<Expression>>, Option<String>) {
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

        let block = self.parse_block();

        self.consume_type(TokenType::RBrace);
        (tstruct.get_line(), tstruct.get_string(), argname, block, ret_type)
    }

    fn parse_idval_and_add(&mut self, hshmap: &mut HashMap<String, Box<Expression>>) -> (Token, bool) {
        let name = self.consume_type(TokenType::Identifier);
        self.consume_type(TokenType::Colon);
        let val = match self.parse_expression(0) {
            Some(x) => x,
            None => panic!("Failed at id_val parsing at line {}", name.get_line())
        };
        let exists = hshmap.contains_key(&name.get_string());
        hshmap.entry(name.get_string()).or_insert(val);
        (name, exists)
    }

    fn parse_new_object(&mut self) -> Box<Expression> {
        let struct_name = self.consume_type(TokenType::StructIdentifier);
        let mut members = HashMap::new();
        if self.match_type(TokenType::LBrace).is_some() {
            while self.look_ahead(0).get_type() == TokenType::Identifier {
                let (name, pe) = self.parse_idval_and_add(&mut members);
                if pe {
                    panic!("Cannot send same tag twice: {} at line {}", name.get_string(), name.get_line());
                }
                self.match_type(TokenType::Comma);
            }
            self.consume_type(TokenType::RBrace);
        };
        Box::new(Expression::NewObject {
            struct_type: struct_name.get_string(),
            members: members
        })
    }

    fn parse_block(&mut self) -> Vec<Box<Expression>> {
        let mut block = vec![];
        let mut be = Some(Box::new(Expression::Reference(0, "NILL is not a value".to_string())));
        while be.clone().is_some() {
            let bt = self.look_ahead(0).clone();
            be = match bt.get_type() {
                TokenType::Module | TokenType::Struct | TokenType::Message | TokenType::Call =>
                    panic!("{}: Declaration not allowed at line {} in block", self.file_name, bt.get_line()),
                TokenType::Let => Some(self.parse_let_statement()),
                TokenType::LBracket => Some(self.parse_message_call(true)),
                TokenType::StructIdentifier => Some(self.parse_new_object()),
                _ => None
            };
            if let Some(ref x) = be {
                block.push(x.clone());
            }
        }
        block
    }

    fn parse_message_declaration(&mut self) -> Box<Expression> {
        let (ln, bn, an, bl, rv) = self.parse_message_like_declaration();
        Box::new(Expression::MessageDeclaration {
            line: ln,
            bound_struct: bn,
            args_or_name: an,
            body: bl,
            ret_value: rv
        })
    }

    fn parse_call_declaration(&mut self) -> Box<Expression> {
        let (ln, bn, an, bl, rv) = self.parse_message_like_declaration();
        Box::new(Expression::CallDeclaration {
            line: ln,
            bound_struct: bn,
            args_or_name: an,
            body: bl,
            ret_value: rv
        })
    }

    fn parse_main_declaration(&mut self) -> Box<Expression> {
        let n = self.consume_type(TokenType::LBrace);
        let bl = self.parse_block();
        self.consume_type(TokenType::RBrace);
        Box::new(Expression::MainDeclaration {
            line: n.get_line(),
            body: bl
        })
    }

    pub fn parse_message_call(&mut self, c: bool) -> Box<Expression> {
        if c {
            self.match_type(TokenType::LBracket);
        }
        // Allow for substructs (structs in other modules)
        // Do we support suboject syntax, or do we just have calls?
        // For now, just calls will do.
        let target_name = match self.match_type(TokenType::Identifier) {
            Some(x) => x,
            None => self.consume_type(TokenType::StructIdentifier)
        };
        let mut args = HashMap::new();
        while self.look_ahead(0).get_type() == TokenType::Identifier {
            let (name, pe) = self.parse_idval_and_add(&mut args);
            if pe {
                panic!("Cannot send same tag twice: {} at line {}", name.get_string(), name.get_line());
            }
            self.match_type(TokenType::Comma);
        }
        self.consume_type(TokenType::RBracket);
        Box::new(Expression::Message {
            target: target_name.get_string(),
            args: args
        })
    }

    fn parse_let_statement(&mut self) -> Box<Expression> {
        // Maybe introduce pattern matching...
        self.match_type(TokenType::Let);
        let name = self.consume_type(TokenType::Identifier);
        let mut name_type = None;
        if let Some(_) = self.match_type(TokenType::Colon) {
            name_type = Some(self.consume_type(TokenType::StructIdentifier).get_string());
        }
        self.consume_type(TokenType::Equal);
        let expr = match self.parse_expression(0) {
            Some(x) => x,
            None => panic!("Failed at let statement at line {}", name.get_line())
        };
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

    pub fn parse_expression(&mut self, precedence: i32) -> Option<Box<Expression>> {
        let tok = self.look_ahead(0).clone();
        let mut parslet: Option<Box<PrefixParslet>> = None;
        match self.prefixs.get(&tok.get_type()) {
            Some(plt) => parslet = Some(plt.dup()),
            None => {}
        };
        let mut left = match parslet {
            None => return None,
            Some(plt) => {
                let tok = self.consume();
                plt.parse(self, tok)
            }
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
        Some(left)
    }

    fn get_precedence(&mut self) -> i32 {
        let tt = self.look_ahead(0).get_type();
        match self.infixs.get(&tt) {
            Some(plt) => plt.get_precedence(),
            None => 0
        }
    }

    pub fn match_type(&mut self, expect: TokenType) -> Option<Token> {
        {
            let tok = self.look_ahead(0);
            if tok.get_type() != expect {
                return None
            }
        }
        Some(self.consume())
    }

    pub fn consume_type(&mut self, expect: TokenType) -> Token {
        {
            let tok = self.look_ahead(0);
            if tok.get_type() != expect {
                panic!("Token type mismatch: {:?} expected, {:?} received at line {}", expect, tok.get_type(), tok.get_line());
            }
        }
        self.consume()
    }

    pub fn consume(&mut self) -> Token {
        self.look_ahead(0);
        match self.t.len() > 0 {
            true => self.t.remove(0),
            false => self.eof.clone()
        }
    }

    pub fn look_ahead(&mut self, x: usize) -> &Token {
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
