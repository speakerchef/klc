use knobc::compiler::CompileOptions;
use knobc::compiler::Compiler;
use std::process::exit;
use std::{env::args, error::Error};

const ARG_ERR_MSG: &str = "Usage: knob [ build | run ] <file.knv> [ optional<exec-name> ]";

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = args().collect();
    let mode = args.get(1);
    let path = args.get(2);
    let exec_name = args.get(3);
    if mode.is_none() || path.is_none() {
        eprintln!("{}", ARG_ERR_MSG);
        exit(1)
    }
    if let Some(mode) = mode
        && mode == "build"
        && exec_name.is_none()
    {
        eprintln!("Please supply an executable name in build mode");
        eprintln!("{}", ARG_ERR_MSG);
        exit(1);
    }

    Compiler::compile(CompileOptions {
        src_pth: path.unwrap().clone(),
        dst_pth: String::from("./"),
        dst_name: exec_name,
        options: Vec::new(),
        mode: if mode.unwrap() == "run" {
            // SAFETY: Guaranteed by checks
            knobc::compiler::CompilerMode::Run
        } else {
            knobc::compiler::CompilerMode::Build
        },
    })?;
    Ok(())
}
