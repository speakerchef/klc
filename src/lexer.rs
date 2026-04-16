use std::{
    error::Error,
    fs::{self},
};

enum Op {
    Nop,
    Add,
    Sub,
    Mul,
    Div,
    Pwr,
    Mod,
    Inc,
    Dec,
    Lsl,
    Lsr,
    BwNot, // '~'
    BwOr,
    BwAnd,
    BwXor,
    LgNot, // '!'
    LgOr,
    LgAnd,
    Eq,
    AddEq,
    SubEq,
    MulEq,
    DivEq,
    PwrEq,
    ModEq,
    AndEq,
    OrEq,
    XorEq,
    LslEq,
    LsrEq,
    Equiv,
    NEquiv,
    Lt,
    Gt,
    Lte,
    Gte,
}

#[derive(Debug)]
enum TokenType {
    DelimSemi,
    KwReturn,
    KwFn,
    KwLet,
    KwMut,
    KwIf,
    KwElif,
    KwElse,
    KwWhile,
    KwExit,
    Op(String),
    DelimLparen,
    DelimRparen,
    DelimLcurly,
    DelimRcurly,
    DelimLsquare,
    DelimRsquare,
    DelimComma,
    LitInt(i128),
    VarIdent(String),
    EOF,
}

#[derive(Debug)]
struct LocData {
    line: usize,
    col: usize,
}

#[derive(Debug)]
pub struct Token {
    kind: TokenType,
    loc: LocData,
}

#[derive(Debug)]
pub struct Lexer {
    tokens: Vec<Token>,
    pos: usize,
    line_ct: usize,
    col_ct: usize,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            tokens: vec![],
            pos: 0,
            line_ct: 1,
            col_ct: 1,
        }
    }

    pub fn tokenize(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let f_str = fs::read_to_string(path)?;
        println!("File string: {}", f_str);
        let mut buf = String::new();

        for ch in f_str.chars() {
            self.col_ct += 1;
            match ch {
                ' ' => {
                    if !buf.is_empty() {
                        self.tokens.push(self.classify_token(
                            &buf,
                            LocData {
                                line: self.line_ct,
                                col: self.col_ct,
                            },
                        )?);
                        buf.clear();
                    }
                }
                '\n' => {
                    if !buf.is_empty() {
                        self.tokens.push(self.classify_token(
                            &buf,
                            LocData {
                                line: self.line_ct,
                                col: self.col_ct,
                            },
                        )?);
                        buf.clear();
                    }
                    self.line_ct += 1;
                    self.col_ct = 0;
                }
                '_' => buf.push(ch),
                _ => {
                    if !ch.is_ascii_alphanumeric() && !buf.is_empty() {
                        // Symbol or operator
                        self.tokens.push(self.classify_token(
                            &buf,
                            LocData {
                                line: self.line_ct,
                                col: self.col_ct,
                            },
                        )?);
                        buf.clear();
                    }
                    buf.push(ch);
                }
            }
        }
        println!("Token: {:?}", self.tokens);

        // TODO: Implement operator classifier with tok vector

        Ok(())
    }

    // pub fn peek(&self, offset: usize) -> Option<&Token> {}

    fn classify_token(&self, tok: &str, loc: LocData) -> Result<Token, Box<dyn Error>> {
        match tok {
            "exit" => Ok(Token {
                kind: TokenType::KwExit,
                loc,
            }),
            "let" => Ok(Token {
                kind: TokenType::KwLet,
                loc,
            }),
            "mut" => Ok(Token {
                kind: TokenType::KwMut,
                loc,
            }),
            "if" => Ok(Token {
                kind: TokenType::KwIf,
                loc,
            }),
            "elif" => Ok(Token {
                kind: TokenType::KwElif,
                loc,
            }),
            "else" => Ok(Token {
                kind: TokenType::KwElse,
                loc,
            }),
            "while" => Ok(Token {
                kind: TokenType::KwWhile,
                loc,
            }),
            "for" => Ok(Token {
                kind: TokenType::KwWhile,
                loc,
            }),
            "fn" => Ok(Token {
                kind: TokenType::KwFn,
                loc,
            }),
            "return" => Ok(Token {
                kind: TokenType::KwReturn,
                loc,
            }),
            ";" => Ok(Token {
                kind: TokenType::DelimSemi,
                loc,
            }),
            "(" => Ok(Token {
                kind: TokenType::DelimLparen,
                loc,
            }),
            ")" => Ok(Token {
                kind: TokenType::DelimRparen,
                loc,
            }),
            "{" => Ok(Token {
                kind: TokenType::DelimLcurly,
                loc,
            }),
            "}" => Ok(Token {
                kind: TokenType::DelimRcurly,
                loc,
            }),
            "[" => Ok(Token {
                kind: TokenType::DelimLsquare,
                loc,
            }),
            "]" => Ok(Token {
                kind: TokenType::DelimRsquare,
                loc,
            }),
            "," => Ok(Token {
                kind: TokenType::DelimComma,
                loc,
            }),
            symbol => {
                if "+-*/<>=|&^!%".contains(symbol) {
                    Ok(Token {
                        kind: TokenType::Op(String::from(symbol)),
                        loc,
                    })
                } else if symbol.chars().all(|c| c.is_ascii_digit()) {
                    Ok(Token {
                        kind: TokenType::LitInt(symbol.parse::<i128>().unwrap()),
                        loc,
                    })
                } else {
                    Ok(Token {
                        kind: TokenType::VarIdent(String::from(symbol)),
                        loc,
                    })
                }
            }
        }
    }
}
