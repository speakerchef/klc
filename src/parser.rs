use std::{cell::Cell, collections::HashMap, error::Error, process::exit, rc::Rc};

use crate::{
    ast::{self, AtomKind, StmtReturn, Type, VarType},
    diagnostics::DiagHandler,
    lexer::{self, LocData, Op, Symbol, Token, TokenType, TokenVec},
    traits::Iter,
};

enum ArgVec {
    FnDefArgs(Vec<(Symbol, ast::Type)>),
    FnCallArgs(Vec<ast::Expr>),
}

pub struct Parser<'a> {
    diag: &'a mut DiagHandler,
    toks: &'a mut TokenVec,
    fns: HashMap<Symbol, Rc<ast::StmtFn>>,
}

impl Parser<'_> {
    pub fn new<'a>(toks: &'a mut TokenVec, diag: &'a mut DiagHandler) -> Parser<'a> {
        Parser {
            diag,
            toks,
            fns: HashMap::new(),
        }
    }

    #[must_use]
    fn validate_tok(&mut self, kind: TokenType) -> bool {
        if let Some(&tok) = self.toks.peek()
            && tok.kind == kind
        {
            return true;
        }
        false
    }

    fn get_infix_bp(&self, op: Op) -> (f64, f64) {
        match op {
            // right associative
            Op::Asgn => (1.1, 1.0),

            Op::LgOr => (2.0, 2.1),
            Op::LgAnd => (3.0, 3.1),

            Op::BwOr => (4.0, 4.1),
            Op::BwXor => (5.0, 5.1),
            Op::BwAnd => (6.0, 6.1),

            Op::Eq | Op::Neq => (7.0, 7.1),

            Op::Lt | Op::Gt | Op::Lte | Op::Gte => (8.0, 8.1),

            Op::Lsl | Op::Lsr => (9.0, 9.1),

            Op::Sub | Op::Add => (10.0, 10.1),
            Op::Mod | Op::Div | Op::Mul => (11.0, 11.1),

            // right associative
            Op::Pwr => (12.1, 12.0),
            _ => todo!("Invalid Op or received unary op"),
        }
    }

    fn parse_var_decl(&mut self, t: Token, outer_scp: &mut ast::Scope) -> Option<ast::VarDecl> {
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
        if let Some(&idtok) = self.toks.peek() {
            let TokenType::VarIdent(sym) = idtok.kind else {
                self.diag.push_err(
                    idtok.loc,
                    &format!("expected identifier; got `{:?}`", idtok.kind),
                );
                return None;
            };
            decl.name = sym;
            self.toks.next(); // eat ident

            // Check for declared type
            if let Some(&colon) = self.toks.peek()
                && matches!(colon.kind, TokenType::Colon)
            {
                let default = Token::default();
                let tytok = self.toks.next().unwrap_or(&default); // eat ':'
                if !matches!(
                    tytok.kind,
                    TokenType::Ti8
                        | TokenType::Ti16
                        | TokenType::Ti32
                        | TokenType::Ti64
                        | TokenType::Tu8
                        | TokenType::Tu16
                        | TokenType::Tu32
                        | TokenType::Tu64
                        | TokenType::Tf32
                        | TokenType::Tf64
                        | TokenType::Tusize
                        | TokenType::Tchar
                        | TokenType::Tstring
                        | TokenType::Tbool
                ) {
                    self.diag.push_err(
                        tytok.loc,
                        &format!("expected valid type specifier; got {} instead", tytok.kind),
                    );
                } else {
                    decl.decl_type = Some(Type::from(tytok.kind));
                }
                self.toks.next(); // eat typename
            }
            if !self.validate_tok(TokenType::Op(Op::Asgn)) {
                self.diag
                    .push_err(t.loc, "expected `=` after variable declaration");
            }

            self.toks.next(); // eat '='
            decl.value = Box::new(if let Some(expr) = self.parse_expr(0., outer_scp) {
                self.check_semi(self.toks.peek_behind().unwrap().loc);
                expr
            } else {
                self.diag
                    .push_err(idtok.loc, "expected variable declaration");
                return None;
            });
            Some(decl)
        } else {
            self.diag.push_err(
                t.loc,
                &format!(
                    "expected identifier after variable declaration `{}`",
                    decl.kind
                ),
            );
            None
        }
    }

    fn parse_expr(&mut self, min_rbp: f64, outer_scp: &mut ast::Scope) -> Option<ast::Expr> {
        let mut lhs: Option<ast::Expr> = None;

        // parenthesized expressions
        if let Some(&lparen) = self.toks.peek()
            && matches!(lparen.kind, TokenType::Lparen)
        {
            self.toks.next(); // eat '('
            lhs = self.parse_expr(0., outer_scp);
            if !self.validate_tok(TokenType::Rparen) {
                self.diag.push_err(
                    self.toks.peek().unwrap_or(&Token::default()).loc,
                    "expected closing `)`",
                );
            } else {
                self.toks.next(); // eat ')'
            }
        }

        // early return on delimiters like ; ) } ,
        if let Some(&tok) = self.toks.peek() {
            match tok.kind {
                TokenType::Rparen | TokenType::Semi | TokenType::Rcurly | TokenType::Comma => {
                    return lhs;
                }
                _ => { /* fall-through */ }
            }
        } else {
            let prev_token = self.toks.peek_behind().unwrap();
            self.diag.push_err(
                prev_token.loc,
                &format!("expected expression after `{}`", prev_token.kind),
            );
            return lhs;
        }

        // check operand types
        if let Some(operand) = &self.toks.peek() {
            match operand.kind {
                TokenType::IntLit(val) => {
                    lhs = Some(ast::Expr::default());
                    lhs.as_mut().unwrap().loc = operand.loc;
                    lhs.as_mut().unwrap().atom = AtomKind::IntLit(ast::IntLit {
                        val,
                        loc: operand.loc,
                    });
                    self.toks.next(); // eat literal
                }
                TokenType::VarIdent(sym) => {
                    lhs = Some(ast::Expr::default());
                    lhs.as_mut().unwrap().loc = operand.loc;
                    if let Some(tok) = self.toks.peek_ahead()
                        && matches!(tok.kind, TokenType::Lparen)
                    {
                        let call = self.parse_fn_call(sym, outer_scp);
                        let call_loc = call.loc;
                        if !self.validate_tok(TokenType::Rparen) {
                            self.diag.push_err(call_loc, "expected `)`");
                        } else {
                            self.toks.next(); // eat ')'
                        }
                        lhs.as_mut().unwrap().ty.set(Some(
                            call.return_ty.get().unwrap_or_else(|| ast::Type::Void),
                        ));
                        lhs.as_mut().unwrap().atom = AtomKind::Call(call);
                    } else {
                        lhs.as_mut().unwrap().atom = AtomKind::Ident(ast::Ident {
                            name: sym,
                            loc: operand.loc,
                        });
                        self.toks.next(); // eat ident
                    }
                }
                TokenType::Op(op) => match op {
                    Op::Add => {
                        self.toks.next();
                    }
                    Op::Sub => {
                        todo!("Unary negation")
                    }
                    Op::Nop => {
                        return None;
                    }
                    _ => {
                        // see what op failed
                        dbg!(&op);
                    }
                },
                _ => {
                    //TODO: Make this a sema analysis error
                    self.diag.push_err(
                        operand.loc,
                        &format!("invalid operand of type `{}` in expression", operand.kind),
                    );
                    return lhs;
                }
            }
        } else {
            self.diag.push_err(
                self.toks.peek_behind().unwrap().loc,
                "expected operands to expression",
            );
            return lhs;
        }

        // Binary expressions
        while let Some(&tok) = self.toks.peek()
            && tok.kind.is_op()
        {
            let op: Op = Op::from(tok);
            assert_ne!(op, Op::Nop);
            let (lbp, rbp) = self.get_infix_bp(op);
            if lbp < min_rbp {
                break;
            }
            self.toks.next(); // eat operator
            if let Some(rhs) = self.parse_expr(rbp, outer_scp) {
                let aggregate_node = ast::Expr {
                    atom: AtomKind::None,
                    loc: lhs.as_ref().unwrap().loc,
                    op,
                    lhs: Some(Box::new(lhs.unwrap())),
                    rhs: Some(Box::new(rhs)),
                    ty: Cell::new(None),
                };
                lhs = Some(aggregate_node);
            } else {
                self.diag.push_err(
                    tok.loc,
                    &format!("expected rhs operand in binary expression after `{}`", op),
                );
                return lhs;
            }
        }
        lhs
    }

    fn check_semi(&mut self, loc: LocData) {
        match self
            .toks
            .peek()
            .unwrap_or(&Token {
                kind: TokenType::Null,
                loc,
            })
            .kind
        {
            TokenType::Semi => (),
            TokenType::Rparen => self.diag.push_err(loc, "extraneous closing `)`"),
            TokenType::Rcurly => self.diag.push_err(loc, "extraneous closing `}`"),
            _ => self.diag.push_err(loc, "expected `;`"),
        }
    }

    fn parse_stmt_exit(&mut self, outer_scp: &mut ast::Scope) -> ast::StmtExit {
        let expr = self.parse_expr(0., outer_scp);
        let loc = self.toks.peek_behind().unwrap().loc;
        self.check_semi(loc);
        ast::StmtExit {
            exit_code: Box::new(expr.unwrap_or_else(|| {
                self.diag
                    .push_err(loc, "expected valid expression after `exit`");
                ast::Expr::default()
            })),
            loc,
        }
    }

    fn parse_stmt_if(&mut self, outer_scp: &mut ast::Scope) -> ast::StmtIf {
        let mut stmt_if = ast::StmtIf {
            cond: self.parse_expr(0., outer_scp).unwrap_or_else(|| {
                self.diag.push_err(
                    self.toks.peek_behind().unwrap().loc,
                    "expected valid `if` condition",
                );
                ast::Expr::default()
            }),
            ..ast::StmtIf::default()
        };
        outer_scp.vars.iter().for_each(|(&k, v)| {
            stmt_if.scope.vars.insert(k, Rc::clone(v));
        });
        outer_scp.fns.iter().for_each(|(&k, v)| {
            stmt_if.scope.fns.insert(k, Rc::clone(v));
        });

        self.toks.next(); // eat '{'
        self.parse_scope(&mut stmt_if.scope);

        while let Some(&maybe_elif) = self.toks.peek()
            && matches!(maybe_elif.kind, TokenType::KwElif)
        {
            self.toks.next(); // eat 'elif'
            let mut _elif = ast::StmtElif {
                cond: self.parse_expr(0., outer_scp).unwrap_or_else(|| {
                    self.diag
                        .push_err(maybe_elif.loc, "expected valid `elif` condition");
                    ast::Expr::default()
                }),
                loc: maybe_elif.loc,
                ..Default::default()
            };
            outer_scp.vars.iter().for_each(|(&k, v)| {
                _elif.scope.vars.insert(k, Rc::clone(v));
            });
            outer_scp.fns.iter().for_each(|(&k, v)| {
                _elif.scope.fns.insert(k, Rc::clone(v));
            });
            self.toks.next(); // eat '{'
            self.parse_scope(&mut _elif.scope);
            stmt_if._elif.push(Some(_elif));
        }

        if let Some(&maybe_else) = self.toks.peek()
            && matches!(maybe_else.kind, TokenType::KwElse)
        {
            self.toks.next(); // eat 'else'
            let mut _else = ast::StmtElse {
                loc: maybe_else.loc,
                ..Default::default()
            };
            outer_scp.vars.iter().for_each(|(&k, v)| {
                _else.scope.vars.insert(k, Rc::clone(v));
            });
            outer_scp.fns.iter().for_each(|(&k, v)| {
                _else.scope.fns.insert(k, Rc::clone(v));
            });
            self.toks.next(); // eat '{'
            self.parse_scope(&mut _else.scope);
            stmt_if._else = Some(_else);
        }
        stmt_if
    }

    fn parse_stmt_while(&mut self, outer_scp: &mut ast::Scope) -> ast::StmtWhile {
        let mut stmt_while = ast::StmtWhile {
            cond: self.parse_expr(0., outer_scp).unwrap_or_else(|| {
                self.diag.push_err(
                    self.toks.peek_behind().unwrap().loc,
                    "expected valid `while` condition",
                );
                ast::Expr::default()
            }),
            ..ast::StmtWhile::default()
        };
        outer_scp.vars.iter().for_each(|(&k, v)| {
            stmt_while.scope.vars.insert(k, Rc::clone(v));
        });
        outer_scp.fns.iter().for_each(|(&k, v)| {
            stmt_while.scope.fns.insert(k, Rc::clone(v));
        });

        self.toks.next(); // eat '{'
        self.parse_scope(&mut stmt_while.scope);

        stmt_while
    }

    #[must_use]
    fn parse_argument_pair(&mut self) -> Option<(Symbol, Type)> {
        let check = self.toks.peek();
        if let Some(tok) = check
            && !matches!(tok.kind, TokenType::VarIdent(_))
        {
            return None;
        }
        self.toks.next(); // also argname and places cursor at `:`

        if !self.validate_tok(TokenType::Colon) {
            self.diag
                .push_err(self.toks.peek_behind().unwrap().loc, "expected `:`");
            return None;
        }
        if let (Some(&argname), Some(&ty)) = (self.toks.peek_behind(), self.toks.peek_ahead()) {
            if ty.kind.is_type_token() && matches!(argname.kind, TokenType::VarIdent(_)) {
                if let TokenType::VarIdent(sym) = argname.kind {
                    self.toks.next(); // eat `:`
                    self.toks.next(); // eat type
                    Some((sym, Type::from(ty.kind)))
                } else {
                    // self.lex.next(); // eat `:`
                    self.diag.push_err(
                        self.toks.peek_behind().unwrap().loc,
                        &format!("expected `VarIdent`; got `{}`", argname.kind),
                    );
                    None
                }
            } else {
                self.diag.push_err(
                    self.toks.peek_behind().unwrap().loc,
                    "expected valid `Type, Identifier` pair for function arguments",
                );
                None
            }
        } else {
            None
        }
    }

    fn parse_args(&mut self, is_call: bool, outer_scp: &mut ast::Scope) -> Option<ArgVec> {
        if !self.validate_tok(TokenType::Lparen) {
            self.diag
                .push_err(self.toks.peek_behind().unwrap().loc, "expected `(`");
        } else {
            self.toks.next(); // eat '('
        }
        let mut def_args = Vec::<(Symbol, Type)>::new();
        let mut call_args = Vec::<ast::Expr>::new();
        let def = lexer::Token::default();
        if is_call {
            loop {
                let loc = self
                    .toks
                    .peek()
                    .unwrap_or_else(|| {
                        self.diag
                            .push_err(self.toks.peek_behind().unwrap().loc, "expected token");
                        &def
                    })
                    .loc;
                if let Some(expr) = self.parse_expr(0., outer_scp) {
                    call_args.push(expr);
                }

                if self.validate_tok(TokenType::Rparen) {
                    break;
                } else if !self.validate_tok(TokenType::Comma) {
                    self.diag
                        .push_err(loc, "expected `,` between function arguments");
                } else {
                    self.toks.next(); //eat ',' & advance to next arg
                }
            }
        } else {
            loop {
                let loc = self
                    .toks
                    .peek()
                    .unwrap_or_else(|| {
                        self.diag
                            .push_err(self.toks.peek_behind().unwrap().loc, "expected token");
                        &def
                    })
                    .loc;
                if let Some(packed_arg) = self.parse_argument_pair() {
                    def_args.push(packed_arg);
                }
                // self.lex.next(); // eat type
                if self.validate_tok(TokenType::Rparen) {
                    break;
                } else if !self.validate_tok(TokenType::Comma) {
                    self.diag
                        .push_err(loc, "expected `,` between function arguments");
                    // break;
                } else {
                    self.toks.next(); //eat ',' & advance to next type
                }
            }
        }
        if is_call && !call_args.is_empty() {
            Some(ArgVec::FnCallArgs(call_args))
        } else if !def_args.is_empty() {
            Some(ArgVec::FnDefArgs(def_args))
        } else {
            None
        }
    }

    fn parse_stmt_fn(&mut self, outer_scp: &mut ast::Scope) -> ast::StmtFn {
        let mut stmt_fn = ast::StmtFn::default();
        if let Some(tok) = self.toks.peek()
            && let TokenType::VarIdent(name) = tok.kind
        {
            stmt_fn.name = name;
            self.toks.next(); // eat name
        } else {
            self.diag.push_err(
                self.toks
                    .peek_behind()
                    .unwrap_or_else(|| {
                        self.diag.display_diagnostics();
                        exit(1);
                    })
                    .loc,
                "expected function identifier after `fn`",
            );
        }

        // Function arguments
        stmt_fn.args = if let Some(args) = self.parse_args(false, outer_scp)
            && let ArgVec::FnDefArgs(def_args) = args
        {
            Some(def_args)
        } else {
            None
        };

        if !self.validate_tok(TokenType::Rparen) {
            self.diag.push_err(
                self.toks
                    .peek_behind()
                    .unwrap_or_else(|| {
                        self.diag.display_diagnostics();
                        exit(1);
                    })
                    .loc,
                "expected `)`",
            );
        }
        self.toks.next(); // eat ')'

        self.toks.next(); // move ahead to read behind
        if let Some(tok) = self.toks.peek_behind()
            && let TokenType::Op(op) = tok.kind
            && matches!(op, Op::ThinArrow)
        // check for '->'
        {
            // self.lex.next(); // eat '->'
            if let Some(&tok) = self.toks.peek() {
                if tok.kind.is_type_token() {
                    stmt_fn.return_ty = Type::from(tok.kind);
                } else {
                    self.diag.push_err(
                        tok.loc,
                        &format!("expected function return type; received `{}`", tok.kind),
                    );
                }
                self.toks.next(); // eat return type
            } else {
                self.diag.push_err(tok.loc, "expected function return type");
            }
        } else {
            stmt_fn.return_ty = Type::Void;
        }
        println!("Function return type: {}", stmt_fn.return_ty);

        if !self.validate_tok(TokenType::Lcurly) {
            self.diag.push_err(
                self.toks
                    .peek_behind()
                    .unwrap_or_else(|| {
                        self.diag.display_diagnostics();
                        exit(1);
                    })
                    .loc,
                "expected `{`",
            );
        }
        self.toks.next(); // eat '{'
        outer_scp.vars.iter().for_each(|(&k, v)| {
            stmt_fn.body.vars.insert(k, Rc::clone(v));
        });
        outer_scp.fns.iter().for_each(|(&k, v)| {
            stmt_fn.body.fns.insert(k, Rc::clone(v));
        });
        self.parse_scope(&mut stmt_fn.body);
        stmt_fn
    }

    fn parse_stmt_return(&mut self, outer_scp: &mut ast::Scope) -> ast::StmtReturn {
        StmtReturn {
            value: self.parse_expr(0., outer_scp),
        }
    }

    fn parse_fn_call(&mut self, name: Symbol, outer_scp: &mut ast::Scope) -> ast::Call {
        let mut call = ast::Call {
            name,
            loc: self.toks.peek().unwrap().loc, // SAFETY: Guaranteed by entry into function
            ..ast::Call::default()
        };
        self.toks.next(); // eat ident

        call.return_ty
            .set(self.fns.get(&name).map(|existing_fn| existing_fn.return_ty));
        call.args = if let Some(args) = self.parse_args(true, outer_scp)
            && let ArgVec::FnCallArgs(call_args) = args
        {
            Some(call_args)
        } else {
            None
        };
        call
    }

    fn parse_scope(&mut self, outer_scp: &mut ast::Scope) {
        let mut loc_scp = ast::Scope::default();
        outer_scp.vars.iter().for_each(|(&k, v)| {
            loc_scp.vars.insert(k, Rc::clone(v));
        });
        outer_scp.fns.iter().for_each(|(&k, v)| {
            loc_scp.fns.insert(k, Rc::clone(v));
        });

        while let Some(&tok) = self.toks.peek() {
            if matches!(tok.kind, TokenType::Rcurly) {
                self.toks.next(); // eat '}'
                break; // end of scope
            }
            match tok.kind {
                TokenType::KwExit => {
                    self.toks.next(); // eat 'exit'
                    let enode = self.parse_stmt_exit(&mut loc_scp);
                    outer_scp.stmts.push(ast::UnionNode::StmtExit(enode));
                }
                TokenType::KwLet | TokenType::KwMut => {
                    self.toks.next(); // eat 'let' | 'mut'
                    let decl: ast::VarDecl =
                        if let Some(decl_res) = self.parse_var_decl(tok, &mut loc_scp) {
                            decl_res
                        } else {
                            self.diag.push_err(tok.loc, "expected variable declaration");
                            ast::VarDecl::default()
                        };
                    let sym = decl.name;
                    println!("Var Decl LocData: {}", decl.loc);
                    let rc = Rc::new(decl);

                    loc_scp.vars.insert(sym, Rc::clone(&rc));
                    outer_scp.stmts.push(ast::UnionNode::VarDecl(rc));
                }
                TokenType::VarIdent(sym) => {
                    // check if fn call
                    if let Some(tok) = self.toks.peek_ahead()
                        && matches!(tok.kind, TokenType::Lparen)
                    {
                        let call = self.parse_fn_call(sym, &mut loc_scp);
                        if !self.validate_tok(TokenType::Rparen) {
                            self.diag.push_err(
                                self.toks.peek_behind().unwrap().loc /* SAFETY: Guaranteed to exist */,
                                "expected `)`");
                        } else {
                            self.toks.next(); // eat ')'
                        }
                        outer_scp.stmts.push(ast::UnionNode::Call(call));
                    } else {
                        let mut decl = self.parse_var_decl(tok, &mut loc_scp).unwrap();
                        decl.is_reassign = true;
                        let sym = decl.name;
                        let rc = Rc::new(decl);
                        loc_scp.vars.insert(sym, Rc::clone(&rc));
                        outer_scp.stmts.push(ast::UnionNode::VarDecl(rc));
                    }
                }
                TokenType::KwIf => {
                    self.toks.next(); // eat 'if'
                    let stmt_if = self.parse_stmt_if(&mut loc_scp);

                    outer_scp.stmts.push(ast::UnionNode::StmtIf(stmt_if));
                }
                TokenType::KwElif => {
                    self.toks.next();
                    self.diag
                        .push_err(tok.loc, "expected accompanying `if` statement for `elif`");
                }
                TokenType::KwElse => {
                    self.toks.next();
                    self.diag
                        .push_err(tok.loc, "expected accompanying `if` statement for `else`");
                }
                TokenType::KwWhile => {
                    self.toks.next(); // eat 'while'
                    let stmt_while = self.parse_stmt_while(&mut loc_scp);
                    outer_scp.stmts.push(ast::UnionNode::StmtWhile(stmt_while));
                }
                TokenType::KwFn => {
                    self.toks.next(); // eat 'fn'
                    let stmt_fn = self.parse_stmt_fn(&mut loc_scp);
                    let sym = stmt_fn.name;
                    let rc = Rc::new(stmt_fn);
                    self.fns.insert(sym, Rc::clone(&rc));
                    outer_scp.stmts.push(ast::UnionNode::StmtFn(rc));
                }
                TokenType::KwReturn => {
                    self.toks.next(); // eat 'return'
                    let ret = self.parse_stmt_return(&mut loc_scp);
                    outer_scp.stmts.push(ast::UnionNode::StmtReturn(ret));
                }
                TokenType::Semi => {
                    self.toks.next();
                    continue;
                }
                // parses raw scope
                TokenType::Lcurly => {
                    self.toks.next();
                    let mut scp = ast::Scope::default();
                    self.parse_scope(&mut loc_scp);
                    scp.stmts.append(&mut loc_scp.stmts);
                    outer_scp.stmts.push(ast::UnionNode::Scope(scp));
                }
                _ => {
                    eprintln!("Unhandled Type: {:?}", tok);
                    self.toks.next();
                }
            }
        }
    }

    pub fn create_program(&mut self) -> ast::Program {
        let mut global_scope = ast::Scope::default();
        self.parse_scope(&mut global_scope);
        ast::Program {
            stmts: global_scope.stmts,
            fns: std::mem::take(&mut self.fns),
        }
    }
}
