use lc3::vm::instructions::{Instruction, Register};
use crate::tokenizer::Token;

pub enum ParserError {
    UnexpectedToken(Token),
    UnexpectedEOF,
    NoOrig,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(Register),
    Number(i16),
    Label(String),
}

pub struct PartialInstruction {
    opcode: String,
    operands: Vec<Operand>,
}

pub enum AstNode {
    Orig(u16, Vec<AstNode>),
    Instruction(PartialInstruction),
    Label(String),
}

pub struct Parser {
    tokens: Vec<Token>,
    pointer: usize,
    ast: Option<AstNode>,
}

// assembly is simple enough where we can parse & codegen in the same step
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Self {
            tokens,
            pointer: 0,
            ast: None,
        }
    }

    pub fn parse(mut self) -> Result<AstNode, ParserError> {
        todo!()
    }


    fn peek(&self) -> Option<Token> {
        if self.pointer < self.tokens.len() {
            Some(self.tokens[self.pointer].clone())
        } else {
            None
        }
    }

    fn next(&mut self) -> Result<Token, ParserError> {
        let token = self.peek();
        if let Some(token) = token {
            self.pointer += 1;
            Ok(token)
        } else {
            Err(ParserError::UnexpectedEOF)
        }
    }
}
