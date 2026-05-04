use core::panic;
use std::{collections::HashMap, rc::Rc};

use crate::{
    ast,
    irgenerator::{self, ArgKind, Br, Call, Define, Expr, KlirNode, ProgScope, Ret, Store},
    lexer,
};

struct AsmMetadata {
    entry: Rc<str>,
    align: usize,
}

#[derive(Debug)]
struct AsmScope {
    id: String,
    data: String,
    stackptr: usize,
    stacksz: usize,
    ret_emitted: bool,
    expr_tmp_counter: usize,
    vars: HashMap<
        String,
        (
            ast::Type,
            usize, /* register counter */
            bool,  /* is function arg?
                    * Denotes if we load from stack address or register
                    */
        ),
    >,
}

impl Default for AsmScope {
    fn default() -> Self {
        Self {
            id: Default::default(),
            data: Default::default(),
            stackptr: 16, /* reserved for FP & LR */
            stacksz: 16,
            expr_tmp_counter: 0,
            ret_emitted: false,
            vars: Default::default(),
        }
    }
}

pub type FunctionMap = HashMap<
    String, /* function name */
    (
        ast::Type,                        /* ret type */
        Option<Vec<(String, ast::Type)>>, /* args */
    ),
>;
pub struct CodeGenerator {
    scopes: Vec<ProgScope>,
    pub asm: String,
    // fn_table: &'a HashMap<Rc<str>, CodegenFuncData>,
    fns_map: FunctionMap,
}

impl CodeGenerator {
    pub fn new(scopes: Vec<ProgScope>) -> CodeGenerator {
        CodeGenerator {
            scopes,
            asm: String::new(),
            fns_map: HashMap::new(),
        }
    }
    fn resolve_integer_resolution(&mut self, val: i128) -> ast::Type {
        match val {
            _ if i8::try_from(val).is_ok() => ast::Type::I8,
            _ if i16::try_from(val).is_ok() => ast::Type::I16,
            _ if i32::try_from(val).is_ok() => ast::Type::I32,
            _ if i64::try_from(val).is_ok() => ast::Type::I64,
            _ if u8::try_from(val).is_ok() => ast::Type::U8,
            _ if u16::try_from(val).is_ok() => ast::Type::U16,
            _ if u32::try_from(val).is_ok() => ast::Type::U32,
            _ if u64::try_from(val).is_ok() => ast::Type::U64,
            _ => panic!("failed to resolve integer resolution"),
        }
    }
    fn emit_typed_load(&mut self, ty: &ast::Type, reg_idx: usize, addr: usize, scp: &mut AsmScope) {
        match ty {
            ast::Type::I8 => {
                scp.data
                    .push_str(&format!("    ldrsb   x{}, [x29, {}]\n", reg_idx, addr));
            }
            ast::Type::I16 => {
                scp.data
                    .push_str(&format!("    ldrsh   x{}, [x29, {}]\n", reg_idx, addr));
            }
            ast::Type::I32 => {
                scp.data
                    .push_str(&format!("    ldrsw   x{}, [x29, {}]\n", reg_idx, addr));
            }
            ast::Type::U8 | ast::Type::Char | ast::Type::Bool => {
                scp.data
                    .push_str(&format!("    ldrb    w{}, [x29, {}]\n", reg_idx, addr));
            }
            ast::Type::U16 => {
                scp.data
                    .push_str(&format!("    ldrh    w{}, [x29, {}]\n", reg_idx, addr));
            }
            ast::Type::U32 => {
                scp.data
                    .push_str(&format!("    ldr     w{}, [x29, {}]\n", reg_idx, addr));
            }
            ast::Type::I64 | ast::Type::U64 => {
                scp.data
                    .push_str(&format!("    ldr     x{}, [x29, {}]\n", reg_idx, addr));
            }
            _ => todo!("This type is not implemented for codegen"),
        }
    }
    fn emit_typed_store(
        &mut self,
        ty: &ast::Type,
        reg_idx: usize,
        reassign_addr: Option<usize>,
        scp: &mut AsmScope,
    ) {
        let addr = if let Some(cached_adr) = reassign_addr {
            cached_adr
        } else {
            scp.stackptr
        };
        match ty {
            ast::Type::I8 | ast::Type::U8 | ast::Type::Char | ast::Type::Bool => {
                scp.data
                    .push_str(&format!("    strb    w{}, [x29, {}]\n", reg_idx, addr));
            }
            ast::Type::I16 | ast::Type::U16 => {
                scp.data
                    .push_str(&format!("    strh    w{}, [x29, {}]\n", reg_idx, addr));
            }
            ast::Type::I32 | ast::Type::U32 => {
                scp.data
                    .push_str(&format!("    str     w{}, [x29, {}]\n", reg_idx, addr));
            }
            ast::Type::I64 | ast::Type::U64 => {
                scp.data
                    .push_str(&format!("    str     x{}, [x29, {}]\n", reg_idx, addr));
            }
            _ => todo!("This type is not implemented for codegen"),
        }
    }
    fn emit_operation(&mut self, op: &lexer::Op, ty: &ast::Type, scp: &mut AsmScope) {
        match op {
            lexer::Op::Add => scp.data.push_str("    add     x8, x9, x10\n"),
            lexer::Op::Sub => scp.data.push_str("    sub     x8, x9, x10\n"),
            lexer::Op::Mul => scp.data.push_str("    mul     x8, x9, x10\n"),
            lexer::Op::Lsl => scp.data.push_str("    lsl     x8, x9, x10\n"),
            lexer::Op::Lsr => scp.data.push_str("    lsr     x8, x9, x10\n"),
            lexer::Op::Asr => scp.data.push_str("    asr     x8, x9, x10\n"),
            lexer::Op::BwAnd => scp.data.push_str("    and     x8, x9, x10\n"),
            lexer::Op::BwOr => scp.data.push_str("    orr     x8, x9, x10\n"),
            lexer::Op::BwXor => scp.data.push_str("    eor     x8, x9, x10\n"),
            lexer::Op::BwNot => scp.data.push_str("    mvn     x8, x9\n"),
            lexer::Op::Div => {
                if ty.is_signed() {
                    scp.data.push_str("    sdiv    x8, x9, x10\n");
                } else {
                    scp.data.push_str("    udiv    x8, x9, x10\n");
                }
            }
            lexer::Op::Mod => {
                self.emit_operation(&lexer::Op::Div, ty, scp);
                scp.data.push_str("    mul     x10, x8, x10\n");
                scp.data.push_str("    sub     x8, x9, x10\n");
            }
            lexer::Op::LgAnd => {
                scp.data.push_str("    cmp     x9, 0\n");
                scp.data.push_str("    cset    x9, ne\n");
                scp.data.push_str("    cmp     x10, 0\n");
                scp.data.push_str("    cset    x10, ne\n");
                scp.data.push_str("    and     x8, x9, x10\n");
                scp.data.push_str("    cmp     x8, 0\n");
                scp.data.push_str("    cset    x8, ne\n");
            }
            lexer::Op::LgOr => {
                scp.data.push_str("    cmp     x9, 0\n");
                scp.data.push_str("    cset    x9, ne\n");
                scp.data.push_str("    cmp     x10, 0\n");
                scp.data.push_str("    cset    x10, ne\n");
                scp.data.push_str("    orr     x8, x9, x10\n");
                scp.data.push_str("    cmp     x8, 0\n");
                scp.data.push_str("    cset    x8, ne\n");
            }
            lexer::Op::Lt => {
                scp.data.push_str("    cmp     x9, x10\n");
                scp.data.push_str("    cset    x8, lt\n");
            }
            lexer::Op::Gt => {
                scp.data.push_str("    cmp     x9, x10\n");
                scp.data.push_str("    cset    x8, gt\n");
            }
            lexer::Op::Lte => {
                scp.data.push_str("    cmp     x9, x10\n");
                scp.data.push_str("    cset    x8, le\n");
            }
            lexer::Op::Gte => {
                scp.data.push_str("    cmp     x9, x10\n");
                scp.data.push_str("    cset    x8, ge\n");
            }
            lexer::Op::Eq => {
                scp.data.push_str("    cmp     x9, x10\n");
                scp.data.push_str("    cset    x8, eq\n");
            }
            lexer::Op::Neq => {
                scp.data.push_str("    cmp     x9, x10\n");
                scp.data.push_str("    cset    x8, ne\n");
            }
            lexer::Op::Pwr => {
                scp.data.push_str("    cbnz    x10, BASE_CASE_1\n"); // deg == 0
                scp.data.push_str("    mov     x8, 1\n");
                scp.data.push_str("    b       PWR_LOOP_END\n");

                scp.data.push_str("BASE_CASE_1:\n"); // deg == 1
                scp.data.push_str("    mov     x8, x9\n"); // move lhs into accum
                scp.data.push_str("    cmp     x10, 1\n");
                scp.data.push_str("    bne     PWR_LOOP_START\n");
                scp.data.push_str("    b       PWR_LOOP_END\n");

                scp.data.push_str("PWR_LOOP_START:\n");
                scp.data.push_str("    sub     x10, x10, 1\n");
                scp.data.push_str("    cbz    x10, PWR_LOOP_END\n");
                scp.data.push_str("    mul     x8, x8, x9\n"); // accum * lhs
                scp.data.push_str("    b       PWR_LOOP_START\n");
                scp.data.push_str("PWR_LOOP_END:\n");
            }
            _ => todo!("This operator is not implemented for codegen"),
        }
    }
    fn emit_move_with_reg(
        &mut self,
        ty: &ast::Type,
        dst_reg_idx: usize,
        src_reg_idx: usize,
        scp: &mut AsmScope,
    ) {
        match ty {
            ast::Type::I8
            | ast::Type::Bool
            | ast::Type::Char
            | ast::Type::I16
            | ast::Type::U8
            | ast::Type::U16
            | ast::Type::U32 => {
                scp.data
                    .push_str(&format!("    mov     w{}, x{}\n", dst_reg_idx, src_reg_idx));
            }
            ast::Type::I32 | ast::Type::I64 | ast::Type::U64 | ast::Type::Usize => {
                scp.data
                    .push_str(&format!("    mov     x{}, x{}\n", dst_reg_idx, src_reg_idx));
            }
            _ => todo!("Type not impl for `mov` yet"),
        }
    }
    fn emit_typed_move(&mut self, ty: &ast::Type, reg_idx: usize, val: i128, scp: &mut AsmScope) {
        let low = (val & 0xFFFF) as u16;
        let low_med = (val >> 16) as u16;
        let high_med = (val >> 32) as u16;
        let high = (val >> 48) as u16;
        match ty {
            ast::Type::I8
            | ast::Type::Bool
            | ast::Type::Char
            | ast::Type::I16
            | ast::Type::U8
            | ast::Type::U16
            | ast::Type::U32 => {
                scp.data
                    .push_str(&format!("    mov     w{}, {}\n", reg_idx, low));
                if low_med != 0 {
                    scp.data
                        .push_str(&format!("    movk    w{}, {}, lsl 16\n", reg_idx, low_med));
                }
            }
            ast::Type::I32 | ast::Type::I64 | ast::Type::U64 | ast::Type::Usize => {
                scp.data
                    .push_str(&format!("    mov     w{}, {}\n", reg_idx, low));
                if low_med != 0 {
                    scp.data
                        .push_str(&format!("    movk    w{}, {}, lsl 16\n", reg_idx, low_med));
                }
                if high_med != 0 {
                    scp.data
                        .push_str(&format!("    movk    w{}, {}, lsl 32\n", reg_idx, high_med));
                }
                if high != 0 {
                    scp.data
                        .push_str(&format!("    movk    w{}, {}, lsl 48\n", reg_idx, high));
                }
            }
            _ => todo!("Type not impl for `mov` yet"),
        }
    }

    fn emit_load_call_args(&mut self, call: &irgenerator::Call, scp: &mut AsmScope) {
        if let Some(args) = &call.args {
            for (argc, (argkind, argty)) in args.iter().enumerate() {
                match argkind {
                    ArgKind::Imm(val) => self.emit_typed_move(argty, argc, *val, scp),
                    ArgKind::Temp(name) | ArgKind::Sym(name) => {
                        let &(var_ty, addr, is_arg) = scp.vars.get(name).unwrap();
                        if is_arg {
                            self.emit_move_with_reg(&var_ty, argc, addr, scp);
                        } else {
                            self.emit_typed_load(&var_ty, argc, addr, scp);
                        }
                    }
                    ArgKind::Call(call) => {
                        self.emit_load_call_args(call, scp);
                        scp.data.push_str(&format!("    bl      {}\n", call.name));
                        self.emit_move_with_reg(&call.return_ty, argc, 0, scp);
                    }
                }
            }
        }
    }

    fn visit_store(&mut self, store: &Store, scp: &mut AsmScope) {
        match &store.src {
            ArgKind::Imm(val) => {
                if let Some(&(dst_ty, dst_addr, is_arg)) = scp.vars.get(&store.dest) {
                    if is_arg {
                        self.emit_move_with_reg(&dst_ty, 8, dst_addr, scp);
                        self.emit_typed_store(&dst_ty, 8, None, scp);
                        scp.vars
                            .insert(store.dest.clone(), (store.ty, scp.stackptr, false));
                        scp.stackptr += 8;
                    } else {
                        self.emit_typed_move(&store.ty, 8, *val, scp);
                        self.emit_typed_store(&store.ty, 8, Some(dst_addr), scp);
                    }
                } else {
                    self.emit_typed_move(&store.ty, 8, *val, scp);
                    self.emit_typed_store(&store.ty, 8, None, scp);
                    scp.vars
                        .insert(store.dest.clone(), (store.ty, scp.stackptr, false));
                    scp.stackptr += 8;
                }
            }
            ArgKind::Sym(name) | ArgKind::Temp(name) => {
                let &(src_ty, src_addr, src_is_arg) = scp.vars.get(name).unwrap();
                if let Some(&(dst_ty, dst_addr, _)) = scp.vars.get(&store.dest) {
                    if src_is_arg {
                        self.emit_move_with_reg(&src_ty, 8, src_addr, scp);
                        self.emit_typed_store(&dst_ty, 8, None, scp);
                        scp.vars
                            .insert(store.dest.clone(), (dst_ty, scp.stackptr, false));
                    } else {
                        self.emit_typed_load(&src_ty, 8, src_addr, scp);
                        self.emit_typed_store(&dst_ty, 8, Some(dst_addr), scp);
                        scp.vars
                            .insert(store.dest.clone(), (dst_ty, dst_addr, false));
                    }
                    scp.stackptr += 8;
                } else {
                    scp.vars
                        .insert(store.dest.clone(), (src_ty, src_addr, src_is_arg));
                }
            }
            ArgKind::Call(call) => {
                self.emit_load_call_args(call, scp);
                scp.data.push_str(&format!("    bl      {}\n", call.name));
                if let Some(&(ty, addr, is_arg)) = scp.vars.get(&store.dest) {
                    if is_arg {
                        self.emit_move_with_reg(&store.ty, 8, addr, scp);
                        self.emit_typed_store(&store.ty, 8, None, scp);
                        scp.vars
                            .insert(store.dest.clone(), (store.ty, scp.stackptr, false));
                        scp.stackptr += 8;
                    } else {
                        self.emit_move_with_reg(&store.ty, 8, 0, scp);
                        self.emit_typed_store(&store.ty, 8, None, scp);
                        scp.vars.insert(store.dest.clone(), (ty, addr, false));
                    }
                } else {
                    self.emit_move_with_reg(&store.ty, 8, 0, scp);
                    self.emit_typed_store(&store.ty, 8, None, scp);
                    scp.vars
                        .insert(store.dest.clone(), (store.ty, scp.stackptr, false));
                    scp.stackptr += 8;
                }
            }
        }
    }
    fn visit_expr(&mut self, expr: &Expr, scp: &mut AsmScope) {
        let mut reassign_addr = None;
        let lhtmp = match &expr.lhs {
            ArgKind::Sym(name) | ArgKind::Temp(name) => {
                let &(ty, sym_addr, is_arg) = scp
                    .vars
                    .get(name)
                    .unwrap_or_else(|| panic!("Error loading address for variable {name}"));
                if is_arg {
                    self.emit_move_with_reg(&ty, 9, sym_addr, scp);
                } else {
                    self.emit_typed_load(&ty, 9, sym_addr, scp);
                }
                self.emit_typed_store(&expr.ty, 9, None, scp);
                let call_tmp_name = format!("lhtmp{}.", scp.expr_tmp_counter);
                scp.vars
                    .insert(call_tmp_name.clone(), (ty, scp.stackptr, false));
                scp.expr_tmp_counter += 1;
                scp.stackptr += 8;
                call_tmp_name
            }
            ArgKind::Call(call) => {
                self.emit_load_call_args(call, scp);
                scp.data.push_str(&format!("    bl      {}\n", call.name));
                self.emit_typed_store(&expr.ty, 0, None, scp);

                let call_tmp_name = format!("lhtmp.{}", scp.expr_tmp_counter);
                scp.vars
                    .insert(call_tmp_name.clone(), (call.return_ty, scp.stackptr, false));
                scp.expr_tmp_counter += 1;
                scp.stackptr += 8;
                call_tmp_name
            }
            ArgKind::Imm(val) => {
                scp.data.push_str(&format!("    mov     x9, {}\n", val));
                self.emit_typed_store(&expr.ty, 9, None, scp);
                let call_tmp_name = format!("lhtmp{}.", scp.expr_tmp_counter);
                scp.vars.insert(
                    call_tmp_name.clone(),
                    (self.resolve_integer_resolution(*val), scp.stackptr, false),
                );
                scp.expr_tmp_counter += 1;
                scp.stackptr += 8;
                call_tmp_name
            }
        };
        let rhtmp = match &expr.rhs {
            ArgKind::Sym(name) | ArgKind::Temp(name) => {
                let &(ty, sym_addr, is_arg) = scp
                    .vars
                    .get(name)
                    .unwrap_or_else(|| panic!("Error loading address for variable {name}"));
                if is_arg {
                    self.emit_move_with_reg(&ty, 10, sym_addr, scp);
                } else {
                    self.emit_typed_load(&ty, 10, sym_addr, scp);
                }
                self.emit_typed_store(&expr.ty, 10, None, scp);
                let call_tmp_name = format!("lhtmp{}.", scp.expr_tmp_counter);
                scp.vars
                    .insert(call_tmp_name.clone(), (ty, scp.stackptr, false));
                scp.expr_tmp_counter += 1;
                scp.stackptr += 8;
                call_tmp_name
            }
            ArgKind::Call(call) => {
                self.emit_load_call_args(call, scp);
                scp.data.push_str(&format!("    bl      {}\n", call.name));
                self.emit_typed_store(&expr.ty, 0, None, scp);

                let call_tmp_name = format!("rhtmp{}.", scp.expr_tmp_counter);
                scp.vars
                    .insert(call_tmp_name.clone(), (call.return_ty, scp.stackptr, false));
                scp.expr_tmp_counter += 1;
                scp.stackptr += 8;
                call_tmp_name
            }
            ArgKind::Imm(val) => {
                scp.data.push_str(&format!("    mov     x10, {}\n", val));
                self.emit_typed_store(&expr.ty, 10, None, scp);
                let call_tmp_name = format!("rhtmp{}.", scp.expr_tmp_counter);
                scp.vars.insert(
                    call_tmp_name.clone(),
                    (self.resolve_integer_resolution(*val), scp.stackptr, false),
                );
                scp.expr_tmp_counter += 1;
                scp.stackptr += 8;
                call_tmp_name
            }
        };
        let mut ty_to_store = expr.ty;
        if let Some(&(ty, sym_addr, is_arg)) = scp.vars.get(&expr.dest)
            && !is_arg
        {
            reassign_addr = Some(sym_addr);
            ty_to_store = ty;
        }

        let &(lty, laddr, _) = scp.vars.get(&lhtmp).unwrap();
        let &(rty, raddr, _) = scp.vars.get(&rhtmp).unwrap();
        self.emit_typed_load(&lty, 9, laddr, scp);
        self.emit_typed_load(&rty, 10, raddr, scp);

        self.emit_operation(&expr.op, &expr.ty, scp);
        self.emit_typed_store(&ty_to_store, 8, reassign_addr, scp);
        scp.vars.insert(
            expr.dest.clone(),
            (
                ty_to_store,
                if let Some(readdr) = reassign_addr {
                    readdr
                } else {
                    let ret = scp.stackptr;
                    scp.stackptr += 8;
                    ret
                },
                false,
            ),
        );
    }
    fn visit_define(&mut self, define: &Define, scp: &mut AsmScope) {
        let mut args_vec = Vec::new();
        if let Some(args) = &define.args {
            for (argc, (arg_type, ty)) in args.iter().enumerate() {
                match arg_type {
                    ArgKind::Imm(_val) => {
                        panic!("Cannot have imm in function def args")
                    }
                    ArgKind::Call(_) => {
                        panic!("Cannot have function calls in function def args")
                    }
                    ArgKind::Temp(name) | ArgKind::Sym(name) => {
                        scp.vars.insert(name.clone(), (*ty, argc, true)); // forward decl of these vars
                        args_vec.push((name.clone(), *ty));
                    }
                }
            }
            self.fns_map.insert(
                define.name.clone(),
                (
                    define.return_ty,
                    if !args_vec.is_empty() {
                        Some(args_vec)
                    } else {
                        None
                    },
                ),
            );
        }
    }
    fn visit_call(&mut self, call: &Call, scp: &mut AsmScope) {
        if let Some(args) = &call.args {
            for (argc, (arg_type, ty)) in args.iter().enumerate() {
                match arg_type {
                    ArgKind::Imm(val) => {
                        self.emit_typed_move(ty, argc, *val, scp);
                    }
                    ArgKind::Call(call) => {
                        self.emit_load_call_args(call, scp);
                        scp.data.push_str(&format!("    bl      {}\n", call.name));
                        self.emit_move_with_reg(ty, argc, 0, scp);
                    }
                    ArgKind::Temp(name) | ArgKind::Sym(name) => {
                        let &(var_ty, addr, is_arg) = scp.vars.get(name).unwrap();
                        if is_arg {
                            self.emit_move_with_reg(&var_ty, argc, addr, scp);
                        } else {
                            self.emit_typed_load(&var_ty, argc, addr, scp);
                        }
                    }
                }
            }
        }
        scp.data.push_str(&format!("    bl      {}\n", call.name));
    }
    fn visit_br(&mut self, br: &Br, scp: &mut AsmScope) {
        if let Some(flag) = &br.flag {
            let &(ty, addr, _) = scp
                .vars
                .get(flag)
                .unwrap_or_else(|| panic!("Could not get addr for {flag}"));
            self.emit_typed_load(&ty, 8, addr, scp);
            scp.data
                .push_str(&format!("    cbnz    w8, {}\n", br.label));
        } else {
            scp.data.push_str(&format!("    b       {}\n", br.label));
        }
    }
    fn visit_ret(&mut self, ret: &Ret, scp: &mut AsmScope) {
        if let Some(retval) = &ret.value {
            match retval {
                ArgKind::Imm(val) => self.emit_typed_move(&ret.return_ty, 0, *val, scp),
                ArgKind::Sym(name) | ArgKind::Temp(name) => {
                    let &(_, addr, is_arg) = scp.vars.get(name).unwrap();
                    if is_arg {
                        self.emit_move_with_reg(&ret.return_ty, 0, addr, scp);
                    } else {
                        self.emit_typed_load(&ret.return_ty, 0, addr, scp);
                    }
                }
                ArgKind::Call(call) => {
                    self.emit_load_call_args(call, scp);
                    scp.data.push_str(&format!("    bl      {}\n", call.name));
                    self.emit_move_with_reg(&ret.return_ty, 0, 0, scp);
                }
            }
        }
        self.emit_epilogue(scp);
        scp.data.push_str("    ret\n");
        scp.ret_emitted = true;
    }

    fn emit_prologue(&mut self, scp: &mut AsmScope) {
        let _amt = scp.stackptr.next_multiple_of(16) + 16; /* +16 to account for FP & Link register `stp` */
        scp.data.insert_str(0, "    mov     x29, sp \n");
        scp.data
            // .insert_str(0, &format!("    stp     x29, x30, [sp, -{}]!\n", amt));
            .insert_str(
                0,
                &format!("    stp     x29, x30, [sp, -{}]!\n", scp.stacksz),
            );
    }
    fn emit_epilogue(&mut self, scp: &mut AsmScope) {
        let _amt = scp.stackptr.next_multiple_of(16) + 16;
        scp.data
            // .push_str(&format!("    ldp     x29, x30, [sp], {}\n", amt));
            .push_str(&format!("    ldp     x29, x30, [sp], {}\n", scp.stacksz));
    }
    fn emit_metadata(&mut self, md: AsmMetadata) {
        self.asm
            .insert_str(0, &format!(".global {}\n.align {}\n", md.entry, md.align));
    }

    pub fn generate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut asm_scopes = Vec::<AsmScope>::new();
        let ir_scopes = std::mem::take(&mut self.scopes);
        for scope in &ir_scopes {
            let mut asm_scp = AsmScope {
                id: scope.id.clone(),
                stacksz: (scope
                    .ir
                    .nodes
                    .iter()
                    .filter(|&node| {
                        matches!(
                            node,
                            KlirNode::Expr(_) | KlirNode::Store(_) | KlirNode::Alloca(_)
                        )
                    })
                    .count()
                    * 16
                    + 16)
                    .next_multiple_of(16),
                ..AsmScope::default()
            };

            for node in &scope.ir.nodes {
                match node {
                    KlirNode::Alloca(_alloca) => {}
                    KlirNode::Store(store) => {
                        self.visit_store(store, &mut asm_scp);
                    }
                    KlirNode::Expr(expr) => {
                        self.visit_expr(expr, &mut asm_scp);
                    }
                    KlirNode::Define(define) => {
                        self.visit_define(define, &mut asm_scp);
                    }
                    KlirNode::Call(call) => {
                        self.visit_call(call, &mut asm_scp);
                    }
                    KlirNode::Br(br) => {
                        self.visit_br(br, &mut asm_scp);
                    }
                    KlirNode::Label(label) => {
                        asm_scp.data.push_str(&format!("{}:\n", label.name));
                    }
                    KlirNode::Ret(ret) => {
                        self.visit_ret(ret, &mut asm_scp);
                    }
                }
            }
            self.emit_prologue(&mut asm_scp);
            if asm_scp.id == "main" {
                asm_scp.data.insert_str(0, "_main:\n"); // entry point
            } else {
                asm_scp.data.insert_str(0, &format!("{}:\n", asm_scp.id)); // label
            }
            if !asm_scp.ret_emitted {
                self.emit_epilogue(&mut asm_scp);
                asm_scp.data.push_str("    ret\n");
            }
            asm_scopes.push(asm_scp);
        }

        let md = AsmMetadata {
            entry: "_main".into(),
            align: 4,
        };
        for scp in &asm_scopes {
            self.asm.push_str(&scp.data);
        }
        self.emit_metadata(md);
        println!("ASSEMBLY: \n{}", self.asm);
        Ok(())
    }
}
