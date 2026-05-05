// TODO! Add line number information into the tokens for error reporting

use crate::{asm::Attempt, tryit};

const INSTRUCTIONS: &[&str] = &[
    "add", "and", "brn", "brnz", "brnzp", "brz", "brzp", "brp", "brnz", "brnp", "jmp", "jsr",
    "jsrr", "ld", "ldi", "ldr", "lea", "not", "ret", "rti", "st", "sti", "str", "trap", "getc",
    "puts", "in", "out", "halt", // trap vector convienences
];

pub type TokenizerResult<T> = Attempt<T, TokenizerErrorInfo>;

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
    pub index: usize,
    pub kind: TokenizerErrorKind,
}

#[derive(Debug, Eq, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub source_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
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

            // skipping comments might bring us to EOF!
            if self.at_eof() {
                break;
            }

            let word = tryit!(self.consume_word().map(|val| val.to_lowercase()));

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

    fn make_token(&self, kind: TokenKind) -> Token {
        Token { kind, source_index: self.pointer }
    }

    fn err<T>(&self, kind: TokenizerErrorKind) -> TokenizerResult<T> {
        TokenizerResult::Err(self.create_error_info(kind))
    }

    fn check_label(&mut self, word: &str) -> TokenizerResult<Token> {
        if let Some(first) = word.chars().next() {
            if first.is_ascii_digit() {
                self.err(TokenizerErrorKind::InvalidLabel)
            } else {
                TokenizerResult::Ok(self.make_token(TokenKind::Label(word.to_string())))
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
                    self.skip_whitespace();
                    self.try_skip_comment(); // skip any comments after this one on the next line
                    break;
                }
            }
        }
        self.skip_whitespace();
    }

    fn skip_leading_spaces(&mut self) {
        const SKIPPABLE: &[char] = &[' ', '\t'];

        while let Ok(c) = self.next_char() {
            if !SKIPPABLE.contains(&c) {
                self.pointer -= 1;
                break;
            }
        }
    }

    fn check_number_literal(&mut self, word: &str) -> TokenizerResult<Token> {
        if let Some(c) = word.chars().nth(0)
            && (c.is_ascii_digit() || c == '#' || c == 'x')
        {
            self.read_next_i16_num(word).map(|num| self.make_token(TokenKind::Number(num)))
        } else {
            TokenizerResult::Fallthrough
        }
    }

    fn create_error_info(&self, kind: TokenizerErrorKind) -> TokenizerErrorInfo {
        TokenizerErrorInfo { index: self.pointer-1, kind }
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
                        TokenizerResult::Ok(self.make_token(TokenKind::Register(num)))
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
            TokenizerResult::Ok(self.make_token(TokenKind::Instruction(current_word.to_string())))
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
                    TokenizerResult::Ok(self.make_token(TokenKind::Origin(index)))
                }

                ".fill" => {
                    let word = tryit!(self.consume_word()).to_string();
                    let index = tryit!(self.read_next_i16_num(&word));
                    TokenizerResult::Ok(self.make_token(TokenKind::Fill(index)))
                }

                ".end" => TokenizerResult::Ok(self.make_token(TokenKind::End)),

                ".stringz" => {
                    let s = tryit!(self.read_string());
                    TokenizerResult::Ok(self.make_token(TokenKind::Stringz(s.to_string())))
                }

                ".blkw" => {
                    let word = tryit!(self.consume_word()).to_string();
                    let count = tryit!(self.read_next_u16_bit_num(&word));

                    if count == 0 {
                        self.err(TokenizerErrorKind::BlkwParameterTooSmall)
                    } else {
                        TokenizerResult::Ok(self.make_token(TokenKind::Blkw(count)))
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
            match num_str.parse::<i16>() {
                Ok(num) => TokenizerResult::Ok(num),
                Err(_) => self.err(TokenizerErrorKind::InvalidNumber),
            }
        } else {
            // default is decimal number
            match word.parse::<i16>() {
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
            match num_str.parse::<u16>() {
                Ok(num) => TokenizerResult::Ok(num),
                Err(_) => self.err(TokenizerErrorKind::InvalidNumber),
            }
        } else {
            // default is decimal number
            match word.parse::<u16>() {
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
