use std::collections::HashMap;

use crate::asm::{codegen::partial_instruction::PartialInstruction, tokenizer::{Token, TokenKind}};
use lc3::vm::instructions::Register;

#[derive(Debug)]
pub struct ParserError {
    pub token: Token,
    pub kind: ParserErrorKind,
}

#[derive(Debug)]
pub enum ParserErrorKind {
    UnexpectedToken,
    UnexpectedEOF,
    NoOrig,

    ExpectedRegister,
    InvalidInstruction,
    ExpectedImmediate5,
    ExpectedLabel,
    ExpectedOffset9,
    ExpectedOffset11,
    ExpectedOffset6,
    ExpectedTrapVect8,

    CompoundError(Vec<ParserError>),
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(Register),
    Number(i16),
    Label(String),
}

#[derive(Debug)]
pub struct Ast {
    pub orig_sections: Vec<AstNode>,
}

impl Ast {
    pub fn scan_for_labels(&self) -> HashMap<String, usize> {
        let mut map = HashMap::new();

        for orig in &self.orig_sections {
            // keep track of how many bytes we have passed
            let mut byte_distance = 0;

            match orig {
                AstNode::Orig(pos, ast_nodes) => {
                    for node in ast_nodes {
                        if let AstNode::Label(name) = node {
                            map.insert(name.clone(), *pos as usize + byte_distance);
                        }

                        byte_distance += node.calculate_word_length();
                    }
                }

                _ => eprintln!("root ast contained non-origs"), // BUG
            }
        }

        map
    }
}

#[derive(Debug)]
pub enum AstNode {
    Orig(u16, Vec<AstNode>),
    Instruction(PartialInstruction),
    Label(String),

    Fill(i16),
    Stringz(String),
    Blkw(u16),
}

impl AstNode {
    pub fn calculate_word_length(&self) -> usize {
        // LC-3 words are 2 bytes
        match self {
            AstNode::Orig(_, ast_nodes) => {
                let mut acc = 0;
                for node in ast_nodes {
                    acc += node.calculate_word_length();
                }

                acc
            }
            AstNode::Instruction(_) => 1,
            AstNode::Label(_) => 0,
            AstNode::Fill(_) => 1,
            AstNode::Stringz(str) => str.len() + 1, // + null terminator, each char gets it's own word
            AstNode::Blkw(size) => *size as usize,
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pointer: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Self { tokens, pointer: 0 }
    }

    fn err<T>(&self, token: Token, err: ParserErrorKind) -> Result<T, ParserError> {
        Err(ParserError { token, kind: err })
    }

    pub fn parse(mut self) -> Result<Ast, ParserError> {
        let mut origs = Vec::new();
        loop {
            let orig = self.parse_orig();

            match orig {
                Ok(node) => {
                    origs.push(node);
                }

                Err(err) => {
                    if origs.is_empty() {
                        return Err(err);
                    } else {
                        match err.kind {
                            ParserErrorKind::UnexpectedEOF => break,
                            _ => return Err(err),
                        }
                    }
                }
            }
        }

        Ok(Ast {
            orig_sections: origs,
        })
    }

    fn parse_orig(&mut self) -> Result<AstNode, ParserError> {
        let mut result = Vec::new();

        let start = self.next()?;
        match start.kind {
            TokenKind::Origin(index) => {
                loop {
                    let next = self.next()?;
                    // println!("got token: {next:?}");

                    let ast = match &next.kind {
                        TokenKind::End => break,
                        TokenKind::Label(label) => AstNode::Label(label.clone()),
                        TokenKind::Instruction(opcode) => self.parse_instruction(opcode, &next)?,

                        TokenKind::Fill(val) => AstNode::Fill(*val),
                        TokenKind::Blkw(val) => AstNode::Blkw(*val),
                        TokenKind::Stringz(val) => AstNode::Stringz(val.clone()),

                        _ => return self.err(next, ParserErrorKind::UnexpectedToken),
                    };

                    result.push(ast);
                }

                Ok(AstNode::Orig(index, result))
            }

            _ => self.err(start, ParserErrorKind::NoOrig),
        }
    }

    fn parse_instruction(&mut self, opcode: &str, token: &Token) -> Result<AstNode, ParserError> {
        if opcode.starts_with("br") {
            return Ok(AstNode::Instruction(PartialInstruction::new(
                opcode.to_string(),
                vec![self.expect_label_or_offset_9()?],
            )));
        }

        match opcode {
            "add" | "and" => Ok(AstNode::Instruction(PartialInstruction::new(
                opcode.to_string(),
                vec![
                    self.expect_register()?,
                    self.expect_register()?,
                    self.expect_register().or_else(|err1| {
                        self.backtrack(); // TODO: implement custom Result type where backtracking can be made automatic by storing the 'starting' pointer.
                        self.expect_immediate_5()
                            .map_err(|err2| ParserError { token: token.clone(), kind: ParserErrorKind::CompoundError(vec![err1, err2]) })
                    })?,
                ],
            ))),

            "jmp" | "jsrr" => Ok(AstNode::Instruction(PartialInstruction::new(
                opcode.to_string(),
                vec![self.expect_register()?],
            ))),

            "jsr" => Ok(AstNode::Instruction(PartialInstruction::new(
                opcode.to_string(),
                vec![self.expect_label_or_offset_11()?],
            ))),

            "ld" | "ldi" | "lea" | "st" | "sti" => {
                Ok(AstNode::Instruction(PartialInstruction::new(
                    opcode.to_string(),
                    vec![self.expect_register()?, self.expect_label_or_offset_9()?],
                )))
            }

            "ldr" | "str" => Ok(AstNode::Instruction(PartialInstruction::new(
                opcode.to_string(),
                vec![
                    self.expect_register()?,
                    self.expect_register()?,
                    self.expect_offset_6()?,
                ],
            ))),

            "not" => Ok(AstNode::Instruction(PartialInstruction::new(
                opcode.to_string(),
                vec![self.expect_register()?, self.expect_register()?],
            ))),

            "ret" => Ok(AstNode::Instruction(PartialInstruction::new(
                "jmp".to_string(),
                vec![Operand::Register(Register::R7)],
            ))),

            "rti" => Ok(AstNode::Instruction(PartialInstruction::new(
                opcode.to_string(),
                vec![],
            ))),

            "trap" => Ok(AstNode::Instruction(PartialInstruction::new(
                opcode.to_string(),
                vec![self.expect_trapvect8()?],
            ))),

            "getc" => Ok(AstNode::Instruction(PartialInstruction::new(
                "trap".into(),
                vec![Operand::Number(0x20)],
            ))),

            "out" => Ok(AstNode::Instruction(PartialInstruction::new(
                "trap".into(),
                vec![Operand::Number(0x21)],
            ))),

            "puts" => Ok(AstNode::Instruction(PartialInstruction::new(
                "trap".into(),
                vec![Operand::Number(0x22)],
            ))),

            "in" => Ok(AstNode::Instruction(PartialInstruction::new(
                "trap".into(),
                vec![Operand::Number(0x23)],
            ))),

            "putsp" => Ok(AstNode::Instruction(PartialInstruction::new(
                "trap".into(),
                vec![Operand::Number(0x24)],
            ))),

            "halt" => Ok(AstNode::Instruction(PartialInstruction::new(
                "trap".into(),
                vec![Operand::Number(0x25)],
            ))),

            _ => self.err(token.clone(), ParserErrorKind::UnexpectedToken),
        }
    }

    fn expect_trapvect8(&mut self) -> Result<Operand, ParserError> {
        let n = self.next()?;

        match n.kind {
            // offset 6 is used for register offsets, so no labels in this case
            // Token::Label(label) => Ok(Operand::Label(label)),
            TokenKind::Number(num) if (-128..=127).contains(&num) => Ok(Operand::Number(num)),
            _ => self.err(n, ParserErrorKind::ExpectedTrapVect8),
        }
    }

    fn expect_offset_6(&mut self) -> Result<Operand, ParserError> {
        let n = self.next()?;

        match n.kind {
            // offset 6 is used for register offsets, so no labels in this case
            // Token::Label(label) => Ok(Operand::Label(label)),
            TokenKind::Number(num) if (-32..=31).contains(&num) => Ok(Operand::Number(num)),
            _ => self.err(n, ParserErrorKind::ExpectedOffset6),
        }
    }

    fn expect_label_or_offset_11(&mut self) -> Result<Operand, ParserError> {
        let n = self.next()?;

        match n.kind {
            TokenKind::Label(label) => Ok(Operand::Label(label)),
            TokenKind::Number(num) => {
                if (-1024..=1023).contains(&num) {
                    Ok(Operand::Number(num))
                } else {
                    self.err(n, ParserErrorKind::ExpectedOffset11)
                }
            }
            _ => self.err(n, ParserErrorKind::ExpectedLabel),
        }
    }

    fn expect_label_or_offset_9(&mut self) -> Result<Operand, ParserError> {
        let n = self.next()?;

        match n.kind {
            TokenKind::Label(label) => Ok(Operand::Label(label)),
            TokenKind::Number(num) => {
                if (-256..=255).contains(&num) {
                    Ok(Operand::Number(num))
                } else {
                    self.err(n, ParserErrorKind::ExpectedOffset9)
                }
            }
            _ => self.err(n, ParserErrorKind::ExpectedLabel),
        }
    }

    fn expect_immediate_5(&mut self) -> Result<Operand, ParserError> {
        let n = self.next()?;

        match n.kind {
            TokenKind::Number(n) if (-16..=15).contains(&n) => Ok(Operand::Number(n)),

            _ => self.err(n, ParserErrorKind::ExpectedImmediate5),
        }
    }

    fn expect_register(&mut self) -> Result<Operand, ParserError> {
        let n = self.next()?;
        match n.kind {
            TokenKind::Register(reg) => Ok(Operand::Register(Register::from(reg))),

            _ => self.err(n, ParserErrorKind::ExpectedRegister),
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
            // todo this is kinda weird
            self.err(Token { kind: TokenKind::End, source_index: self.tokens.last().map(|token| token.source_index).expect("nothing to parse") }, ParserErrorKind::UnexpectedEOF)
            // Err(ParserErrorKind::UnexpectedEOF)
        }
    }
}
