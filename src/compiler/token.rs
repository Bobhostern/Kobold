#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenType {
    EndOfFile,

    Identifier,
    StructIdentifier,
    Integer,
    Float,
    CString,

    // Symbols
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Arrow, // ->
    Minus,
    LessThan, // <
    GreaterThan, // >
    Equal,
    Plus,
    Asterisk,
    Backslash,
    Power, // **
    Comma,
    Colon,
    LParen,
    RParen,
    Period,

    // keywords
    Module,
    Struct,
    Let,
    If,
    Inner,
    Message,
}

#[derive(Debug, Clone)]
pub struct Token {
    tipe: TokenType,
    text: String,
    line: i32,
}

impl Token {
    pub fn new(tipe: TokenType, text: &str) -> Token {
        Token {
            tipe: tipe,
            text: text.to_string(),
            line: 0,
        }
    }

    pub fn with_line(&self, i: i32) -> Token {
        Token {
            tipe: self.tipe.clone(),
            text: self.text.clone(),
            line: i
        }
    }

    pub fn get_type(&self) -> TokenType {
        self.tipe
    }

    pub fn get_string(&self) -> String {
        self.text.clone()
    }

    pub fn get_line(&self) -> i32 {
        self.line
    }
}

// Stores a list of Tokens, and allows iteration.
// All Tokens in a TokenStream belong to the TokenStream
#[derive(Debug, Clone)]
pub struct TokenStream {
    toks: Vec<Token>,
    index: usize
}

impl TokenStream {
    pub fn new() -> TokenStream {
        TokenStream {
            toks: vec![],
            index: 0
        }
    }

    pub fn len(&self) -> usize {
        self.toks.len() - self.index
    }

    pub fn add(&mut self, t: Token) {
        self.toks.push(t)
    }

    pub fn read(&mut self, size: usize) -> Vec<Token> {
        let mut v = vec![];
        for _ in 0..size {
            if self.toks.len() > 0 {
                v.push(self.toks.remove(0));
            } else { break }
        }
        v
    }
}
