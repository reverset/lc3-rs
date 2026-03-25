// TODO! Add line number information into the tokens for error reporting

const INSTRUCTIONS: &[&str] = &[
    "add", "and", "brn", "brnz", "brnzp", "brz", "brzp", "brp", "brnz", "brnp", "jmp", "jsr",
    "jsrr", "ld", "ldi", "ldr", "lea", "not", "ret", "rti", "st", "sti", "str", "trap", "getc",
    "puts", "in", "out", "halt", // trap vector convienences
];

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
pub enum TokenizerError {
    UnexpectedEOF,
    InvalidDirective,
    InvalidNumber,
    ExpectedString,
    InvalidRegister,
    InvalidLabel,
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

    pub fn tokenize(mut self) -> Result<Vec<Token>, TokenizerError> {
        while !self.at_eof() {
            self.skip_newlines();

            self.try_skip_comment();

            let word = self.consume_word()?.to_string();
            let token = self
                .check_directive(&word)
                .or_else(|_| self.check_instruction(&word))
                .or_else(|_| self.check_register(&word))
                .or_else(|_| self.check_number_literal(&word))
                .or_else(|_| self.check_label(&word))?;

            // println!("TOKEN: {token:?}");
            self.tokens.push(token);
        }

        Ok(self.tokens)
    }

    fn check_label(&mut self, word: &str) -> Result<Token, TokenizerError> {
        if let Some(first) = word.chars().next() {
            if first.is_digit(10) {
                Err(TokenizerError::InvalidLabel)
            } else {
                Ok(Token::Label(word.to_string()))
            }
        } else {
            Err(TokenizerError::UnexpectedEOF)
        }
    }

    fn try_skip_comment(&mut self) {
        self.skip_leading_spaces();
        if self.peek() == Some(';') {
            while let Ok(c) = self.next_char() {
                if c == '\n' {
                    break;
                }
            }
        }
        self.skip_newlines();
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

    fn check_number_literal(&mut self, word: &str) -> Result<Token, TokenizerError> {
        Self::read_next_i16_num(word).map(|num| Token::Number(num))
    }

    fn check_register(&mut self, word: &str) -> Result<Token, TokenizerError> {
        if word.to_lowercase().starts_with("r") {
            let num_str = word.chars().nth(1).ok_or(TokenizerError::InvalidRegister)?;

            let num = (num_str as u8).wrapping_sub(48);
            if num <= 7 {
                Ok(Token::Register(num))
            } else {
                Err(TokenizerError::InvalidRegister)
            }
        } else {
            Err(TokenizerError::InvalidRegister)
        }
    }

    fn check_instruction(&mut self, current_word: &str) -> Result<Token, TokenizerError> {
        if INSTRUCTIONS.contains(&current_word) {
            Ok(Token::Instruction(current_word.to_string()))
        } else {
            Err(TokenizerError::InvalidDirective)
        }
    }

    fn check_directive(&mut self, current_word: &str) -> Result<Token, TokenizerError> {
        if !current_word.starts_with('.') {
            Err(TokenizerError::InvalidDirective)
        } else {
            match current_word.to_lowercase().as_str() {
                ".orig" => {
                    let index = Self::read_next_16_bit_num(self.consume_word()?)?;
                    Ok(Token::Origin(index))
                }

                ".fill" => {
                    // let index = Self::read_next_16_bit_num(self.consume_word()?)?;
                    let index = Self::read_next_i16_num(self.consume_word()?)?;
                    Ok(Token::Fill(index))
                }

                ".end" => Ok(Token::End),

                ".stringz" => {
                    let s = self.read_string()?;
                    Ok(Token::Stringz(s.to_string()))
                }

                ".blkw" => {
                    let count = Self::read_next_16_bit_num(self.consume_word()?)?;
                    Ok(Token::Blkw(count))
                }

                _ => Err(TokenizerError::InvalidDirective),
            }
        }
    }

    fn at_eof(&self) -> bool {
        self.pointer >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.pointer)
    }

    fn next_char(&mut self) -> Result<char, TokenizerError> {
        let c = self.source.chars().nth(self.pointer);
        self.pointer += 1;
        c.ok_or(TokenizerError::UnexpectedEOF)
    }

    fn skip_newlines(&mut self) {
        while let Ok(c) = self.next_char() {
            if c == '\n' {
                continue;
            }
            break;
        }
        self.pointer -= 1; // we overstep in the loop
    }

    fn consume_word(&mut self) -> Result<&str, TokenizerError> {
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

        Ok(&self.source[start..(self.pointer - 1)]) // -1 since pointer will be after the space
    }

    fn read_next_i16_num(word: &str) -> Result<i16, TokenizerError> {
        let num_str = &word[1..];

        if word.starts_with('x') {
            // number starts with an 'x', must be hexadecimal
            match i16::from_str_radix(num_str, 16) {
                Ok(num) => Ok(num),
                Err(_) => Err(TokenizerError::InvalidNumber),
            }
        } else if word.starts_with('#') {
            // decimal number
            match i16::from_str_radix(num_str, 10) {
                Ok(num) => Ok(num),
                Err(_) => Err(TokenizerError::InvalidNumber),
            }
        } else {
            // default is decimal number
            match i16::from_str_radix(word, 10) {
                Ok(num) => Ok(num),
                Err(_) => Err(TokenizerError::InvalidNumber),
            }
        }
    }

    // duplicate code! FIXME

    fn read_next_16_bit_num(word: &str) -> Result<u16, TokenizerError> {
        let num_str = &word[1..];

        if word.starts_with('x') {
            // number starts with an 'x', must be hexadecimal
            match u16::from_str_radix(num_str, 16) {
                Ok(num) => Ok(num),
                Err(_) => Err(TokenizerError::InvalidNumber),
            }
        } else if word.starts_with('#') {
            // decimal number
            match u16::from_str_radix(num_str, 10) {
                Ok(num) => Ok(num),
                Err(_) => Err(TokenizerError::InvalidNumber),
            }
        } else {
            // default is decimal number
            match u16::from_str_radix(word, 10) {
                Ok(num) => Ok(num),
                Err(_) => Err(TokenizerError::InvalidNumber),
            }
        }
    }

    fn read_string(&mut self) -> Result<&'a str, TokenizerError> {
        self.skip_leading_spaces();
        let start = self.pointer;
        let mut opened = false;

        loop {
            let c = self.next_char()?;

            if !opened {
                if c != '"' {
                    return Err(TokenizerError::ExpectedString);
                } else {
                    opened = true;
                }
            } else {
                if c == '"' {
                    break;
                }
            }
        }

        Ok(&self.source[(start + 1)..(self.pointer - 1)])
    }
}
