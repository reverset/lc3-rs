// TODO! Add line number information into the tokens for error reporting

use core::panic;
use std::ops::Add;

const INSTRUCTIONS: &[&str] = &[
    "add", "and", "brn", "brnz", "brnzp", "brz", "brzp", "brp", "brnz", "brnp", "jmp", "jsr",
    "jsrr", "ld", "ldi", "ldr", "lea", "not", "ret", "rti", "st", "sti", "str", "trap", "getc",
    "puts", "in", "out", "halt", // trap vector convienences
];

// for some reason the Try trait is still 'experimental', so in order to implement
// similiar behavior for TokenizerResult, I use this macro.
macro_rules! tryit {
    ($what:expr) => {{
        let val = $what;
        match val {
            TokenizerResult::Ok(val) => val,
            _ => return (val).coalesce_type(),
        }
    }};
}

#[derive(Debug)]
pub enum TokenizerResult<T> {
    Ok(T),
    Err(TokenizerErrorInfo),
    Fallthrough,
}

impl<T> TokenizerResult<T> {
    pub fn map<T2>(self, map: impl FnOnce(T) -> T2) -> TokenizerResult<T2> {
        match self {
            TokenizerResult::Ok(val) => TokenizerResult::Ok(map(val)),

            _ => self.coalesce_type(),
        }
    }

    pub fn has_fallen(&self) -> bool {
        match self {
            Self::Fallthrough => true,

            _ => false,
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            Self::Ok(val) => val,

            _ => panic!("TokenizerResult was not Ok"),
        }
    }

    pub fn is_ok(&self) -> bool {
        match self {
            Self::Ok(_) => true,

            _ => false,
        }
    }

    pub fn if_fell(self, map: impl FnOnce() -> Self) -> Self {
        if self.has_fallen() { map() } else { self }
    }

    pub fn coalesce_type<T2>(self) -> TokenizerResult<T2> {
        match self {
            TokenizerResult::Ok(_) => {
                panic!("TokenizerResult was Ok(_) which is invalid for this method.")
            }
            TokenizerResult::Err(err) => TokenizerResult::Err(err),
            TokenizerResult::Fallthrough => TokenizerResult::Fallthrough,
        }
    }
}

impl<T> From<Result<T, TokenizerErrorInfo>> for TokenizerResult<T> {
    fn from(value: Result<T, TokenizerErrorInfo>) -> Self {
        match value {
            Ok(ok) => TokenizerResult::Ok(ok),
            Err(err) => TokenizerResult::Err(err),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct TokenizerErrorInfo {
    pub line: usize,
    pub kind: TokenizerErrorKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    Origin(u16),
    End,
    Fill(i16),
    Blkw(u16),
    Stringz(String),
    Label(String),

    Instruction(String),
    Register(u8),
    Number(i16),
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum TokenizerErrorKind {
    UnexpectedEOF,
    InvalidDirective,
    InvalidNumber,
    ExpectedString,
    InvalidRegister,
    InvalidLabel,

    BlkwParameterTooSmall,
}

pub struct Tokenizer<'a> {
    source: &'a str,
    pointer: usize,
    tokens: Vec<Token>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            pointer: 0,
            tokens: vec![],
        }
    }

    pub fn tokenize(mut self) -> TokenizerResult<Vec<Token>> {
        while !self.at_eof() {
            self.skip_whitespace();

            self.try_skip_comment();

            let word = tryit!(self.consume_word().map(|val| val.to_string()));

            // println!("got word: {word}");

            let token = self
                .check_directive(&word)
                .if_fell(|| self.check_instruction(&word))
                .if_fell(|| self.check_register(&word))
                .if_fell(|| self.check_number_literal(&word))
                .if_fell(|| self.check_label(&word));
            // println!("got token: {token:?}");

            self.tokens.push(tryit!(token));
        }

        TokenizerResult::Ok(self.tokens)
    }

    fn err<T>(&self, kind: TokenizerErrorKind) -> TokenizerResult<T> {
        TokenizerResult::Err(self.create_error_info(kind))
    }

    fn check_label(&mut self, word: &str) -> TokenizerResult<Token> {
        if let Some(first) = word.chars().next() {
            if first.is_digit(10) {
                self.err(TokenizerErrorKind::InvalidLabel)
            } else {
                TokenizerResult::Ok(Token::Label(word.to_string()))
            }
        } else {
            // println!("FAILED");
            self.err(TokenizerErrorKind::UnexpectedEOF)
        }
    }

    fn try_skip_comment(&mut self) {
        self.skip_leading_spaces();
        if self.peek() == Some(';') {
            while let Ok(c) = self.next_char() {
                if c == '\n' {
                    self.try_skip_comment(); // skip any comments after this one on the next line
                    break;
                }
            }
        }
        self.skip_whitespace();
    }

    fn skip_leading_spaces(&mut self) {
        const SKIPPABLE: &[char] = &[' ', '\t'];

        loop {
            match self.next_char() {
                Ok(c) => {
                    if !SKIPPABLE.contains(&c) {
                        self.pointer -= 1;
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }

    fn check_number_literal(&mut self, word: &str) -> TokenizerResult<Token> {
        if let Some(c) = word.chars().nth(0)
            && (c.is_digit(10) || c == '#' || c == 'x')
        {
            self.read_next_i16_num(word)
                .map(|num| Token::Number(num))
                .into()
        } else {
            TokenizerResult::Fallthrough
        }
    }

    fn create_error_info(&self, kind: TokenizerErrorKind) -> TokenizerErrorInfo {
        let line = self.source[..self.pointer]
            .chars()
            .filter(|c| *c == '\n')
            .count()
            + 1;
        TokenizerErrorInfo { line, kind }
    }

    fn check_register(&mut self, word: &str) -> TokenizerResult<Token> {
        // TODO FIXME!! (improve parsing for this)
        if word.to_lowercase().starts_with("r")
            && (word.len() == 2 || word.ends_with(",") || word.ends_with(", "))
        {
            let num_str = word.chars().nth(1);

            match num_str {
                Some(num_str) => {
                    let num = (num_str as u8).wrapping_sub(48);
                    if num <= 7 {
                        TokenizerResult::Ok(Token::Register(num))
                    } else {
                        self.err(TokenizerErrorKind::InvalidRegister)
                    }
                }

                None => self.err(TokenizerErrorKind::InvalidRegister),
            }
        } else {
            TokenizerResult::Fallthrough
        }
    }

    fn check_instruction(&mut self, current_word: &str) -> TokenizerResult<Token> {
        if INSTRUCTIONS.contains(&current_word) {
            TokenizerResult::Ok(Token::Instruction(current_word.to_string()))
        } else {
            TokenizerResult::Fallthrough
        }
    }

    fn check_directive(&mut self, current_word: &str) -> TokenizerResult<Token> {
        if !current_word.starts_with('.') {
            TokenizerResult::Fallthrough
        } else {
            match current_word.to_lowercase().as_str() {
                ".orig" => {
                    let word = tryit!(self.consume_word()).to_string();
                    let index = tryit!(self.read_next_u16_bit_num(&word));
                    TokenizerResult::Ok(Token::Origin(index))
                }

                ".fill" => {
                    let word = tryit!(self.consume_word()).to_string();
                    let index = tryit!(self.read_next_i16_num(&word));
                    TokenizerResult::Ok(Token::Fill(index))
                }

                ".end" => TokenizerResult::Ok(Token::End),

                ".stringz" => {
                    let s = tryit!(self.read_string());
                    TokenizerResult::Ok(Token::Stringz(s.to_string()))
                }

                ".blkw" => {
                    let word = tryit!(self.consume_word()).to_string();
                    let count = tryit!(self.read_next_u16_bit_num(&word));

                    if count == 0 {
                        self.err(TokenizerErrorKind::BlkwParameterTooSmall)
                    } else {
                        TokenizerResult::Ok(Token::Blkw(count))
                    }
                }

                _ => self.err(TokenizerErrorKind::InvalidDirective),
            }
        }
    }

    fn at_eof(&self) -> bool {
        self.pointer >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.pointer)
    }

    fn next_char(&mut self) -> Result<char, TokenizerErrorKind> {
        let c = self.source.chars().nth(self.pointer);
        self.pointer += 1;
        c.ok_or(TokenizerErrorKind::UnexpectedEOF)
    }

    fn skip_whitespace(&mut self) {
        while let Ok(c) = self.next_char() {
            if c == '\n' || c == ' ' || c == '\t' {
                continue;
            }
            break;
        }
        self.pointer -= 1; // we overstep in the loop
    }

    fn consume_word(&mut self) -> TokenizerResult<&str> {
        self.skip_leading_spaces();
        let start = self.pointer;

        const WORD_DELIMETERS: &[char] = &[' ', '\n'];

        loop {
            let c = self.next_char();
            match c {
                Ok(c) => {
                    if WORD_DELIMETERS.contains(&c) {
                        break;
                    };
                }
                Err(_) => break,
            }
        }

        TokenizerResult::Ok(&self.source[start..(self.pointer - 1)]) // -1 since pointer will be after the space
    }

    fn read_next_i16_num(&self, word: &str) -> TokenizerResult<i16> {
        let num_str = &word[1..];

        if word.starts_with('x') {
            // number starts with an 'x', must be hexadecimal
            match i16::from_str_radix(num_str, 16) {
                Ok(num) => TokenizerResult::Ok(num),
                Err(_) => self.err(TokenizerErrorKind::InvalidNumber),
            }
        } else if word.starts_with('#') {
            // decimal number
            match i16::from_str_radix(num_str, 10) {
                Ok(num) => TokenizerResult::Ok(num),
                Err(_) => self.err(TokenizerErrorKind::InvalidNumber),
            }
        } else {
            // default is decimal number
            match i16::from_str_radix(word, 10) {
                Ok(num) => TokenizerResult::Ok(num),
                Err(_) => self.err(TokenizerErrorKind::InvalidNumber),
            }
        }
    }

    // duplicate code! FIXME

    fn read_next_u16_bit_num(&self, word: &str) -> TokenizerResult<u16> {
        let num_str = &word[1..];

        if word.starts_with('x') {
            // number starts with an 'x', must be hexadecimal
            match u16::from_str_radix(num_str, 16) {
                Ok(num) => TokenizerResult::Ok(num),
                Err(_) => self.err(TokenizerErrorKind::InvalidNumber),
            }
        } else if word.starts_with('#') {
            // decimal number
            match u16::from_str_radix(num_str, 10) {
                Ok(num) => TokenizerResult::Ok(num),
                Err(_) => self.err(TokenizerErrorKind::InvalidNumber),
            }
        } else {
            // default is decimal number
            match u16::from_str_radix(word, 10) {
                Ok(num) => TokenizerResult::Ok(num),
                Err(_) => self.err(TokenizerErrorKind::InvalidNumber),
            }
        }
    }

    fn read_string(&mut self) -> TokenizerResult<String> {
        self.skip_leading_spaces();
        // let start = self.pointer;
        let mut opened = false;

        let mut result = String::new();

        loop {
            match self.next_char() {
                Ok(c) => {
                    if !opened {
                        if c != '"' {
                            return self.err(TokenizerErrorKind::ExpectedString);
                        } else {
                            opened = true;
                        }
                    } else {
                        if c == '"' {
                            break;
                        }

                        if c == '\\' {
                            // handle escape sequences
                            match self.next_char() {
                                Ok(esc) => {
                                    let esc_char = match esc {
                                        'n' => '\n',
                                        't' => '\t',
                                        '\\' => '\\',
                                        '"' => '"',
                                        _ => return self.err(TokenizerErrorKind::ExpectedString),
                                    };
                                    result.push(esc_char);
                                }
                                Err(_) => return self.err(TokenizerErrorKind::UnexpectedEOF),
                            }
                        } else {
                            result.push(c);
                        }
                    }
                }
                Err(err) => return self.err(err),
            }
        }

        TokenizerResult::Ok(result)
        // TokenizerResult::Ok(&self.source[(start + 1)..(self.pointer - 1)])
    }
}
