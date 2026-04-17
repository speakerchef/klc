use std::{
    error::Error,
    fs::{self},
};

#[derive(Debug)]
enum Op {
    Nop,
    Add,
    Sub,
    Mul,
    Div,
    Pwr,
    Mod,
    Lsl,
    Lsr,
    BwNot, // '~'
    BwOr,
    BwAnd,
    BwXor,
    LgNot, // '!'
    LgOr,
    LgAnd,
    Asgn,
    AddAsgn,
    SubAsgn,
    MulAsgn,
    DivAsgn,
    PwrAsgn,
    ModAsgn,
    AndAsgn,
    OrAsgn,
    XorAsgn,
    LslAsgn,
    LsrAsgn,
    Eq,
    Neq,
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
    Op(Op),
    DelimLparen,
    DelimRparen,
    DelimLcurly,
    DelimRcurly,
    DelimLsquare,
    DelimRsquare,
    DelimComma,
    LitInt(i128),
    VarIdent(String),
    Null,
}

#[derive(Debug, Clone, Copy)]
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
        let mut file_iter = f_str.chars().peekable();
        println!("File string: \n{}", f_str);
        let mut buf = String::new();

        while let Some(ch) = file_iter.next() {
            self.col_ct += 1;
            let loc = LocData {
                line: self.line_ct,
                col: self.col_ct,
            };
            match ch {
                ';' => {
                    if !buf.is_empty() {
                        self.tokens.push(self.classify_token(&buf, loc)?);
                        buf.clear();
                    }
                    self.tokens.push(Token {
                        kind: TokenType::DelimSemi,
                        loc,
                    });
                }
                ' ' => {
                    if !buf.is_empty() {
                        self.tokens.push(self.classify_token(&buf, loc)?);
                        buf.clear();
                    }
                }
                '\n' => {
                    if !buf.is_empty() {
                        self.tokens.push(self.classify_token(&buf, loc)?);
                        buf.clear();
                    }
                    self.line_ct += 1;
                    self.col_ct = 0;
                }
                '_' => buf.push(ch),
                _ => {
                    // Existing identifiers
                    if ch.is_ascii_alphanumeric() {
                        buf.push(ch);
                    } else {
                        if !buf.is_empty()
                            && buf.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                        {
                            self.tokens.push(self.classify_token(&buf, loc)?);
                            buf.clear();
                        }

                        // Operators
                        buf.push(ch);
                        if let Some(&doub_op) = file_iter.peek()
                            && "+-*/<>=|&^!%".contains(doub_op)
                        {
                            buf.push(doub_op);
                            file_iter.next();
                            self.col_ct += 1;
                            if let Some(&trip_op) = file_iter.peek()
                                && "+-*/<>=|&^!%".contains(trip_op)
                            {
                                buf.push(trip_op);
                                file_iter.next();
                                self.col_ct += 1;
                            }
                        }

                        self.tokens.push(Token {
                            kind: TokenType::Op(self.classify_op(&buf)),
                            loc: LocData {
                                line: self.line_ct,
                                col: self.col_ct,
                            },
                        });
                        buf.clear();
                    }
                }
            }
        }
        println!("Tokens: {:#?}", self.tokens);
        Ok(())
    }

    fn classify_op(&self, op: &str) -> Op {
        match op {
            "+" => Op::Add,
            "-" => Op::Sub,
            "*" => Op::Mul,
            "/" => Op::Div,
            "%" => Op::Mod,
            "**" => Op::Pwr,
            "&" => Op::BwAnd,
            "|" => Op::BwOr,
            "^" => Op::BwXor,
            "~" => Op::BwNot,
            "<<" => Op::Lsl,
            ">>" => Op::Lsr,
            "=" => Op::Asgn,
            "+=" => Op::AddAsgn,
            "-=" => Op::SubAsgn,
            "*=" => Op::MulAsgn,
            "/=" => Op::DivAsgn,
            "%=" => Op::ModAsgn,
            "**=" => Op::PwrAsgn,
            "&=" => Op::AndAsgn,
            "|=" => Op::OrAsgn,
            "^=" => Op::XorAsgn,
            "<<=" => Op::LslAsgn,
            ">>=" => Op::LsrAsgn,
            ">" => Op::Gt,
            "<" => Op::Lt,
            ">=" => Op::Gte,
            "<=" => Op::Lte,
            "==" => Op::Eq,
            "!=" => Op::Neq,
            "&&" => Op::LgAnd,
            "||" => Op::LgOr,
            "!" => Op::LgNot,
            _ => {
                println!("NOP Operator: {}", op);
                Op::Nop
            }
        }
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
                if !symbol.is_empty() {
                    if symbol.chars().all(|c| c.is_ascii_digit()) {
                        return Ok(Token {
                            kind: TokenType::LitInt(symbol.parse::<i128>()?),
                            loc,
                        });
                    } else {
                        return Ok(Token {
                            kind: TokenType::VarIdent(String::from(symbol)),
                            loc,
                        });
                    }
                }
                Ok(Token {
                    kind: TokenType::Null,
                    loc: LocData { line: 0, col: 0 },
                })
            }
        }
    }
}
