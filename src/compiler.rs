use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::process::{Command, exit};
use std::rc::Rc;

use crate::ast;
use crate::backend::CodeGenerator;
use crate::diagnostics::DiagHandler;
use crate::irgenerator::IrGenerator;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantics::Sema;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum CompilerMode {
    Run,
    Build,
}
pub type CodegenFuncData = (
    ast::Type,                         /* return type */
    Option<Vec<(Rc<str>, ast::Type)>>, /* arguments */
);
pub struct CompileOptions<'a> {
    pub src_pth: String,
    pub dst_pth: String,
    pub dst_name: Option<&'a String>,
    pub options: Vec<(String, String)>, // flag, option
    pub mode: CompilerMode,
}

pub struct Compiler {
    pub has_errors: bool,
    pub has_warns: bool,
    pub has_notes: bool,
}

impl Compiler {
    pub fn compile(opts: CompileOptions) -> Result<(), Box<dyn Error>> {
        let file = fs::read_to_string(opts.src_pth)?;
        let mut diagnostics = DiagHandler::new();
        let dst_name = opts.dst_name.unwrap_or(&String::new()).clone();

        // Tokenization
        println!("Tokenizing...");
        let mut lex = Lexer::new();
        lex.tokenize(&file)?;

        // Parsing
        println!("Parsing...");
        let mut parser = Parser::new(&mut lex, &mut diagnostics)?;
        let mut program = parser.create_program();
        let mut symbol_table = std::mem::take(&mut program.sym);
        let mut fn_table = std::mem::take(&mut program.fns);

        // Semantic analysis and type inference + checks
        println!("Semantic Analysis & Type Checking...");
        let mut sema = Sema::new(
            &mut program,
            &mut diagnostics,
            &mut symbol_table,
            &mut fn_table,
        );
        sema.validate_program()?;
        if diagnostics.has_errors() {
            diagnostics.display_diagnostics();
            exit(1);
        }

        // KLIR Generation
        let mut irgenerator = IrGenerator::new(&mut program, &mut diagnostics, &mut symbol_table);
        irgenerator.emit_klir()?;

        let irscopes = std::mem::take(&mut irgenerator.scopes);
        let generator_fn_table: HashMap<Rc<str>, CodegenFuncData> = fn_table
            .iter()
            .map(|(&sym, func)| {
                let remapped_arg_vec = if let Some(args) = &func.args {
                    Some(
                        args.iter()
                            .map(|&(argname, argty)| (symbol_table.get(argname).unwrap(), argty))
                            .collect(),
                    )
                } else {
                    None
                };
                let func_dat: (ast::Type, Option<Vec<(Rc<str>, ast::Type)>>) =
                    (func.return_ty, remapped_arg_vec);

                (symbol_table.get(sym).unwrap().clone(), func_dat)
            })
            .collect();

        // Assembly CodeGen
        let mut backend = CodeGenerator::new(irscopes, &generator_fn_table);
        backend.generate()?;

        std::fs::write("/tmp/knobc_asm_out.s", &backend.asm).expect("Error during compilation!");
        if opts.mode == CompilerMode::Build {
            // SAFETY: Startup checks
            std::fs::write(format!("./{}.s", opts.dst_name.unwrap()), &backend.asm)
                .expect("error during compilation!");
        }
        let _assembler_out = Command::new("clang")
            .args(vec![
                "-c",
                "-g",
                "-Wno-missing-sysroot",
                "-o",
                "/tmp/knobc_asm_out.o",
                "/tmp/knobc_asm_out.s",
            ])
            .output()?;

        let sdk_path_shower = Command::new("xcrun")
            .args(vec!["--sdk", "macosx", "--show-sdk-path"])
            .output()?;

        let oname = format!("./{}", dst_name);
        let _linker_out = Command::new("ld")
            .args(vec![
                "-lSystem",
                "-syslibroot",
                str::from_utf8(&sdk_path_shower.stdout)?.trim(),
                "-o",
                if matches!(opts.mode, CompilerMode::Build) {
                    &oname
                } else {
                    "/tmp/knobc_compiled_bin"
                },
                "/tmp/knobc_asm_out.o",
            ])
            .output()?;

        if matches!(opts.mode, CompilerMode::Run) {
            let _ = Command::new("/tmp/knobc_compiled_bin").output()?;
            let _ = Command::new("rm")
                .args(vec![
                    "/tmp/knobc_asm_out.s",
                    "/tmp/knobc_asm_out.o",
                    "/tmp/knobc_compiled_bin",
                ])
                .output()?;
        } else {
            let _ = Command::new("rm")
                .args(vec!["/tmp/knobc_asm_out.s", "/tmp/knobc_asm_out.o"])
                .output()?;
        }
        Ok(())
    }
}
