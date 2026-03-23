use crate::tokenizer::Token;
use lc3::vm::instructions::{Instruction, Register};

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(Token),
    UnexpectedEOF,
    NoOrig,

    ExpectedRegister(Token),
    InvalidInstruction(Token),
    ExpectedImmediate5(Token),

    CompoundError(Vec<ParserError>),
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(Register),
    Number(i16),
    Label(String),
}

#[derive(Debug)]
pub struct PartialInstruction {
    opcode: String,
    operands: Vec<Operand>,
}

impl PartialInstruction {
    pub fn new(opcode: String, operands: Vec<Operand>) -> PartialInstruction {
        Self { opcode, operands }
    }
}

#[derive(Debug)]
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

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Self {
            tokens,
            pointer: 0,
            ast: None,
        }
    }

    pub fn parse(mut self) -> Result<AstNode, ParserError> {
        let mut orig = self.parse_orig();
        loop {
            if orig.is_err() && self.ast.is_none() {
                return orig;
            }

            if orig.is_err() {
                break;
            }

            if self.ast.is_none() {
                self.ast = orig.ok();
            } else {
                todo!()
            }

            orig = self.parse_orig();
        }

        match orig {
            Ok(ast) => Ok(ast),
            Err(err) => match err {
                ParserError::UnexpectedEOF => Ok(self.ast.unwrap()),
                _ => Err(err),
            },
        }
    }

    fn parse_orig(&mut self) -> Result<AstNode, ParserError> {
        let mut result = Vec::new();
        let start = self.next()?;
        match start {
            Token::Origin(index) => {
                loop {
                    let next = self.next()?;

                    let ast = match &next {
                        Token::End => break,
                        Token::Instruction(opcode) => self.parse_instruction(&opcode, &next)?,

                        _ => return Err(ParserError::UnexpectedToken(next)),
                    };

                    result.push(ast);
                }
                Ok(AstNode::Orig(index, result))
            }

            _ => Err(ParserError::NoOrig),
        }
    }

    fn parse_instruction(&mut self, opcode: &str, token: &Token) -> Result<AstNode, ParserError> {
        match opcode {
            "add" | "and" => Ok(AstNode::Instruction(PartialInstruction::new(
                opcode.to_string(),
                vec![
                    self.expect_register()?,
                    self.expect_register()?,
                    self.expect_register().or_else(|err1| {
                        self.backtrack(); // TODO: implement custom Result type where backtracking can be made automatic by storing the 'starting' pointer.
                        self.expect_immediate_5()
                            .map_err(|err2| ParserError::CompoundError(vec![err1, err2]))
                    })?,
                ],
            ))),

            _ => Err(ParserError::UnexpectedToken(token.clone())),
        }
    }

    fn expect_immediate_5(&mut self) -> Result<Operand, ParserError> {
        let n = self.next()?;

        match n {
            Token::Number(n) if n >= -16 && n <= 15 => Ok(Operand::Number(n)),

            _ => Err(ParserError::ExpectedImmediate5(n)),
        }
    }

    fn expect_register(&mut self) -> Result<Operand, ParserError> {
        let n = self.next()?;
        match n {
            Token::Register(reg) => Ok(Operand::Register(Register::from(reg))),

            _ => Err(ParserError::ExpectedRegister(n)),
        }
    }

    fn backtrack(&mut self) {
        self.pointer = self.pointer.saturating_sub(1);
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
