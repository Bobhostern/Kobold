use std::io::BufRead;
use std::iter::Iterator;
use super::token::{Token, TokenType, TokenStream};
use super::trie::{Trie, TrieError};
use std::collections::HashMap;

pub struct Lexer<T: BufRead> {
    source: T,
    source_name: String,

    keywords: HashMap<String, TokenType>,
    accept_vec: Vec<char>,

    // Mutable state
    line: i32,
    ts: TokenStream,
    ht: bool
}

#[derive(Debug)]
enum LexerState {
    NewlineRN,

    Default,
    Identifier,
    StructIdentifier,
    Integer,
    Float,
    CString,
    Operator,

    OneLineComment,
}

impl<T: BufRead> Lexer<T> {
    pub fn new(name: &str, r: T) -> Lexer<T> {
        let mut tmp = Lexer {
            source_name: name.to_string(),
            source: r,
            line: 1,
            ts: TokenStream::new(),
            ht: false,
            keywords: HashMap::new(),
            accept_vec: vec!['{', '}', '-', '>', '[', ']', '<', '=', '+', '*', '/', ',', ':', '(', ')', '.'],
        };

        tmp.keywords.insert("module".to_string(), TokenType::Module);
        tmp.keywords.insert("struct".to_string(), TokenType::Struct);
        tmp.keywords.insert("let".to_string(), TokenType::Let);
        tmp.keywords.insert("if".to_string(), TokenType::If);
        tmp.keywords.insert("inner".to_string(), TokenType::Inner);
        tmp.keywords.insert("message".to_string(), TokenType::Message);

        tmp.accept_vec.sort();

        tmp
    }

    pub fn process(&mut self) -> TokenStream {
        let avsnp = self.accept_vec.clone();
        let space_idx = |a| match avsnp.binary_search(&a) {
            Ok(ind) => ind as i32,
            Err(_) => -1
        };
        let mut t = Trie::new(self.accept_vec.len(), &space_idx);
        t.add_string("{", TokenType::LBrace);
        t.add_string("}", TokenType::RBrace);
        t.add_string("->", TokenType::Arrow);
        t.add_string("-", TokenType::Minus);
        t.add_string("[", TokenType::LBracket);
        t.add_string("]", TokenType::RBracket);
        t.add_string("<", TokenType::LessThan);
        t.add_string(">", TokenType::GreaterThan);
        t.add_string("=", TokenType::Equal);
        t.add_string("+", TokenType::Plus);
        t.add_string("*", TokenType::Asterisk);
        t.add_string("**", TokenType::Power);
        t.add_string("/", TokenType::Backslash);
        t.add_string(",", TokenType::Comma);
        t.add_string(":", TokenType::Colon);
        t.add_string("(", TokenType::LParen);
        t.add_string(")", TokenType::RParen);
        t.add_string(".", TokenType::Period);
        let ref mut ts = self.ts;
        if !self.ht {
            let mut state = LexerState::Default;
            let mut data = "".to_string();
            let mut file = "".to_string(); self.source.read_to_string(&mut file).ok();
            // println!("{:#?}", file);
            let mut fc = file.chars();
            let mut c = ' ';
            let mut advance = true;
            while fc.clone().count() > 0 {
                match state {
                    LexerState::Default => {
                        if advance {
                            c = match fc.next(){Some(h)=>h,_=>break};
                        } else { advance = true; }
                        data = "".to_string();
                        match c {
                            'a'...'z' => {data.push(c); state = LexerState::Identifier;},
                            'A'...'Z' => {data.push(c); state = LexerState::StructIdentifier;},
                            '0'...'9' => {data.push(c); state = LexerState::Integer;},
                            '#' => state = LexerState::OneLineComment,
                            '\r' => state = LexerState::NewlineRN,
                            '\n' => self.line += 1,
                            ';' => {},
                            '"' => state = LexerState::CString,
                            _ => {
                                if !c.is_whitespace() {
                                    if self.accept_vec.binary_search(&c).is_ok() {
                                        state = LexerState::Operator;
                                    } else {
                                        panic!("{}:{}: Unknown symbol {}", self.source_name, self.line, c);
                                    }
                                }
                            }
                        }
                    },
                    LexerState::NewlineRN => {
                        c = match fc.next(){Some(h)=>h,_=>break};
                        match c {
                            '\n' => {self.line += 1; state = LexerState::Default},
                            _ => {advance = false; state = LexerState::Default},
                        }
                    },
                    LexerState::Identifier => {
                        c = match fc.next(){Some(h)=>h,_=>break};
                        match c {
                            'a'...'z' | 'A'...'Z' | '0'...'9' | '_' | '!' | '?' => data.push(c),
                            _ => {
                                let ntok = match self.keywords.get(&data) {
                                    Some(typ) => Token::new(*typ, &data),
                                    None => Token::new(TokenType::Identifier, &data)
                                }.with_line(self.line);
                                ts.add(ntok);
                                advance = false;
                                state = LexerState::Default;
                            }
                        }
                    },
                    LexerState::StructIdentifier => {
                        c = match fc.next(){Some(h)=>h,_=>break};
                        match c {
                            'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => data.push(c),
                            _ => {
                                ts.add(Token::new(TokenType::StructIdentifier, &data).with_line(self.line));
                                advance = false;
                                state = LexerState::Default;
                            }
                        }
                    },
                    LexerState::OneLineComment => {
                        c = match fc.next(){Some(h)=>h,_=>break};
                        match c {
                            '\r' => state = LexerState::NewlineRN,
                            '\n' => {self.line+=1; state = LexerState::Default},
                            _ => {},
                        }
                    },
                    LexerState::Operator => {
                        // Assume that current c is accepted.
                        data.push(c);
                        match t.search(&data) {
                            Ok(ty) => {
                                let mut pk = fc.clone().peekable();
                                if let Some(x) = pk.peek() {
                                    let mut tmp = data.clone();
                                    tmp.push(*x);
                                    match t.search(&tmp) {
                                        Err(TrieError::End)|Err(TrieError::Null)|Err(TrieError::NoHash)|Err(TrieError::NoChar) => { ts.add(Token::new(ty, &data).with_line(self.line)); state=LexerState::Default; },
                                        _ => {c = match fc.next(){Some(h)=>h,_=>break};}
                                    };
                                } else {
                                    ts.add(Token::new(ty, &data).with_line(self.line)); state=LexerState::Default;
                                }
                            },
                            Err(TrieError::End)|Err(TrieError::Null)|Err(TrieError::NoHash)|Err(TrieError::NoChar) => {
                                panic!("{}:{}: Error lexing {}", self.source_name, self.line, data);
                            },
                            _ => {c=match fc.next(){Some(h)=>h,_=>break};}
                        }
                    },
                    LexerState::CString => {
                        c=match fc.next(){Some(h)=>h,_=>break};
                        match c {
                            '"' => {ts.add(Token::new(TokenType::CString, &data).with_line(self.line)); state=LexerState::Default},
                            '/' => {
                                // Escape sequence processing
                            },
                            _ => data.push(c)
                        };
                    },
                    LexerState::Integer => {
                        c=match fc.next(){Some(h)=>h,_=>break};
                        match c {
                            '0'...'9' => data.push(c),
                            '.' | 'e' => {data.push(c); state=LexerState::Float},
                            _ => {ts.add(Token::new(TokenType::Integer, &data).with_line(self.line)); state=LexerState::Default;}
                        }
                    },
                    LexerState::Float => {
                        c=match fc.next(){Some(h)=>h,_=>break};
                        match c {
                            '0'...'9' => data.push(c),
                            _ => {ts.add(Token::new(TokenType::Float, &data).with_line(self.line)); state=LexerState::Default;}
                        }
                    },
                    // e @ _ => unreachable!("All states in a DST should be handled. {:?} {:?}", e, c)
                };
            }
        }
        ts.clone()
    }
}
