use std::{cell::Cell, error::Error, rc::Rc};

use crate::{
    ast::{self, ExprKind, Type, VarType},
    diagnostics::DiagHandler,
    lexer::{Lexer, LocData, Op, Token, TokenType},
    traits::Iter,
};

pub struct Parser<'a> {
    program: ast::Program,
    diagnostics: &'a mut DiagHandler,
    lex: &'a mut Lexer,
}

impl Parser<'_> {
    pub fn new<'a>(
        lex: &'a mut Lexer,
        diagnostics: &'a mut DiagHandler,
    ) -> Result<Parser<'a>, Box<dyn Error>> {
        Ok(Parser {
            program: ast::Program::default(),
            diagnostics,
            lex,
        })
    }

    #[must_use]
    fn validate_tok(&mut self, kind: TokenType) -> bool {
        if let Some(&tok) = self.lex.peek()
            && tok.kind == kind
        {
            return true;
        }
        false
    }

    fn parse_var_decl(&mut self, t: Token) -> Option<ast::VarDecl> {
        let mut decl = ast::VarDecl {
            loc: t.loc,
            kind: if matches!(t.kind, TokenType::KwLet) {
                VarType::Let
            } else {
                VarType::Mut
            },
            ..Default::default()
        };

        // Get identifier
        if let Some(&idtok) = self.lex.peek() {
            let TokenType::VarIdent(sym) = idtok.kind else {
                self.diagnostics.push_err(
                    idtok.loc,
                    &format!("expected identifier; got `{:?}`", idtok.kind),
                );
                return None;
            };
            decl.id = ast::Ident {
                name: sym,
                loc: idtok.loc,
            };
            self.lex.next(); // eat ident

            // Check for delcared type
            if let Some(&colon) = self.lex.peek()
                && matches!(colon.kind, TokenType::Colon)
            {
                let default = Token::default();
                let tytok = self.lex.next().unwrap_or(&default); // eat ':'
                if !matches!(
                    tytok.kind,
                    TokenType::KwInt
                        | TokenType::KwFloat
                        | TokenType::KwBool
                        | TokenType::KwChar
                        | TokenType::KwString
                ) {
                    self.diagnostics.push_err(
                        tytok.loc,
                        &format!("expected type specifier; got `{:?}`", tytok.kind),
                    );
                } else {
                    decl.decl_type = Some(Type::from(tytok.kind));
                }
            }
            self.lex.next();
            if !self.validate_tok(TokenType::Op(Op::Asgn)) {
                self.diagnostics
                    .push_err(t.loc, "expected `=` after variable declaration");
            }

            decl.value = Box::new(self.parse_expr());
            Some(decl)
        } else {
            self.diagnostics.push_err(
                t.loc,
                &format!(
                    "expected identifier after variable declaration `{}`",
                    decl.kind
                ),
            );
            None
        }
    }

    fn parse_expr(&mut self) -> ast::Expr {
        //TODO: Impl pratt parsing
        // let mut lhs = ast::Expr::default();
        // if let Some(&delim) = self.lex.peek()
        //     && matches!(
        //         delim.kind,
        //         TokenType::Semi | TokenType::Rparen | TokenType::Rcurly
        //     )
        // {
        //     return lhs;
        // }
        // if let Some(&lparen) = self.lex.peek()
        //     && matches!(lparen.kind, TokenType::Lparen)
        // {
        //     self.lex.next(); // eat '('
        //     lhs = self.parse_expr();
        //     if !self.validate_tok(TokenType::Rparen) {
        //         self.diagnostics.push_err(
        //             self.lex.peek().unwrap_or(&Token::default()).loc,
        //             "expected closing `)`",
        //         );
        //     } else {
        //         self.lex.next(); // eat ')'
        //     }
        // }
        //
        // todo!("Handle literals and identifier tokens");
        //
        // while let Some(&op) = self.lex.peek()
        //     && op.kind.is_op()
        // {
        //     let op: Op = Op::from(op);
        //     assert_ne!(op, Op::Nop);
        //     todo!("Pratt parsing for expressions");
        //
        //     // let bp = self.get_infix_bp(op);
        // }

        ast::Expr {
            kind: ExprKind::IntLit(ast::IntLit {
                val: 69,
                loc: self.lex.peek().unwrap().loc,
            }),
            lhs: None,
            rhs: None,
            ty: Cell::new(None),
            loc: LocData { line: 0, col: 0 },
        }
    }

    fn parse_stmt(&mut self, scp: &mut ast::Scope, is_prog: bool) -> Result<(), Box<dyn Error>> {
        let mut loc_scp = ast::Scope::default();
        // if is_prog {
        //     self.program.vars.iter().for_each(|(&k, v)| {
        //         loc_scp.vars.insert(k, v.clone());
        //     });
        // } else {
        scp.vars.iter().for_each(|(&k, v)| {
            loc_scp.vars.insert(k, v.clone());
        });
        // }

        while let Some(&tok) = self.lex.peek() {
            if matches!(tok.kind, TokenType::Rcurly) {
                break; // end of scope
            }
            match tok.kind {
                TokenType::KwExit => {
                    todo!("Yet to implement exit stmt parsing");
                    // self.parse_stmt_exit();
                }
                TokenType::KwLet | TokenType::KwMut => {
                    self.lex.next(); // eat 'let' | 'mut'
                    let decl: ast::VarDecl = self.parse_var_decl(tok).unwrap();
                    let sym = decl.id.name;
                    let rc = Rc::new(decl);
                    loc_scp.vars.insert(sym, Rc::clone(&rc));
                    self.program.stmts.push(ast::UnionNode::VarDecl(rc));
                }
                TokenType::VarIdent(sym) => {
                    println!("IDENT");
                    if !loc_scp.vars.contains_key(&sym) && !loc_scp.fns.contains_key(&sym) {
                        self.diagnostics.push_err(
                            tok.loc,
                            &format!("undeclared identifier `{}`", self.lex.sym.get(sym).unwrap()),
                        );
                    }

                    if loc_scp.vars.contains_key(&sym) {
                        if !matches!(loc_scp.vars.get(&sym).unwrap().kind, VarType::Let) {
                            self.diagnostics.push_err(
                                tok.loc,
                                &format!(
                                    "cannot mutate immutable variable {} declared with `let`",
                                    self.lex.sym.get(sym).unwrap()
                                ),
                            );
                            self.diagnostics.push_note(
                                tok.loc,
                                &format!(
                                    "did you mean to use `mut` when declaring {}?",
                                    self.lex.sym.get(sym).unwrap()
                                ),
                            );
                            break;
                        }
                        let decl = self.parse_var_decl(tok).unwrap();
                        let sym = decl.id.name;
                        let rc = Rc::new(decl);
                        loc_scp.vars.insert(sym, Rc::clone(&rc));
                        self.program.stmts.push(ast::UnionNode::VarDecl(rc));
                    }
                }
                TokenType::Semi => {
                    println!("SEMI");
                    self.lex.next();
                    continue;
                }
                _ => {
                    println!("Symbols: {:?}", self.lex.sym.symbols);
                    println!("Stmts: {:#?}", self.program.stmts);
                    // self.diagnostics.display_diagnostics();
                    // panic!("Unhandled Type");
                    eprintln!("Unhandled Type");
                    self.lex.next();
                }
            }
        }
        Ok(())
    }
    pub fn create_program(&mut self) -> Result<ast::Program, Box<dyn Error>> {
        self.parse_stmt(&mut ast::Scope::default(), true)?;
        Ok(ast::Program {
            sym: self.lex.sym.clone(),
            stmts: self.program.stmts.clone(),
        })
    }
}
