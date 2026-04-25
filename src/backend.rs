use std::collections::HashMap;

use crate::{ast, irgenerator::KlirBlob, lexer};
pub struct CodeGenerator {
    ir: KlirBlob,
    vars: HashMap<lexer::Symbol, (ast::Type, usize /* register counter */)>,
}

impl CodeGenerator {
    pub fn new(ir: KlirBlob) -> Self {
        CodeGenerator {
            ir,
            vars: HashMap::new(),
        }
    }

    pub fn generate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // delimiters are ' ' and ', '
        // build token
        // TODO: Add IR Nodes
        Ok(())
    }
}
