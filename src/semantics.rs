use crate::{
    ast::{self, UnionNode},
    diagnostics::DiagHandler,
};
use std::{error::Error, rc::Rc};

pub struct Sema;
impl Sema {
    fn visit_expr(expr: &ast::Expr, diag: &mut DiagHandler) {
        if expr.lhs.is_none() && expr.rhs.is_none() {
            match expr.atom {
                ast::AtomKind::Ident(_) => {
                    let extracted_type = ast::Type::default();
                    expr.ty.set(Some(extracted_type));
                }
                ast::AtomKind::IntLit(_) => expr.ty.set(Some(ast::Type::Int)),
                _ => unreachable!(),
            }
        }
        if let Some(lhs) = &expr.lhs {
            Sema::visit_expr(lhs, diag);
        }
        // let rhs = &expr.rhs;
        if let Some(rhs) = &expr.rhs {
            Sema::visit_expr(rhs, diag);
        }
    }
    fn visit_decl(decl: &ast::VarDecl, diag: &mut DiagHandler) {
        Sema::visit_expr(decl.value.as_ref(), diag);
        if matches!(decl.value.ty.get().unwrap(), ast::Type::None) {
            diag.push_err(
                decl.loc,
                &format!(
                    "could not resolve type",
                    // prog.sym.get(decl.id.name).unwrap()
                ),
            );
        }
        decl.ty.set(decl.value.ty.get());
        if let (Some(anno_ty), Some(infer_ty)) = (decl.decl_type, decl.ty.get())
            && anno_ty != infer_ty
        {
            diag.push_err(
                decl.id.loc,
                &format!("expected {} and got {}", infer_ty, anno_ty),
                //TODO: Type conversion
            );
            diag.push_note(
                decl.id.loc,
                &format!("consider changing `{}` to `{}`", anno_ty, infer_ty),
            );
        }
    }
    pub fn validate_program(
        prog: &mut ast::Program,
        diag: &mut DiagHandler,
    ) -> Result<(), Box<dyn Error>> {
        dbg!("AST: {:#?}", &prog);
        print!("Diagnostics at validate_program()");
        diag.display_diagnostics();

        for stmt in &prog.stmts {
            match stmt.clone() {
                UnionNode::VarDecl(decl) => {
                    let rc = &mut Rc::clone(&decl);
                    Sema::visit_decl(rc, diag);
                }
                UnionNode::Expr(mut expr) => {
                    Sema::visit_expr(expr.as_mut(), diag);
                }
                _ => todo!(),
            }
        }
        Ok(())
    }
}
