
#[derive(Clone, Debug)]
pub enum Token {
    Origin(u16),
    End,
    Fill(u16),
    Blkw(u16),
    Stringz(String),
    Label(String),

    Instruction(String),
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum TokenizerError {
    UnexpectedEOF,
    InvalidDirective,
    InvalidNumber,
    ExpectedString,
}

pub struct Tokenizer<'a> {
    source: &'a str,
    pointer: usize,
    tokens: Vec<Token>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source,
            pointer: 0,
            tokens: vec![],
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, TokenizerError> {
        while !self.at_eof() {
            self.skip_newlines();

            self.try_skip_comment();

            let word = self.consume_word()?.to_string();
            let token = 
                self.check_directive(&word)
                    .or_else(|_| self.check_instruction(&word))?;
            
            // println!("TOKEN: {token:?}");
            self.tokens.push(token);
        }

        Ok(self.tokens)
    }

    fn try_skip_comment(&mut self) {
        self.skip_leading_spaces();
        if self.peek() == Some(';') {
            while let Ok(c) = self.next_char() {
                if c == '\n' { break; }
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

    // TODO add full list of instructions
    fn check_instruction(&mut self, current_word: &str) -> Result<Token, TokenizerError> {
        println!("word: {current_word}");
        match current_word.to_lowercase().as_str() { // lol
            // surely there's a better way to do this
            x @ "add" | 
            x @ "and" | 
            x @ "br" | 
            x @ "brn" | 
            x @ "brnz" | 
            x @ "brnzp" | 
            x @ "brz" | 
            x @ "brzp" | 
            x @ "brnp" | 
            x @ "brp"
             => Ok(Token::Instruction(x.to_string())),

            _ => Err(TokenizerError::InvalidDirective),
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
                    let index = Self::read_next_16_bit_num(self.consume_word()?)?;
                    Ok(Token::Fill(index))
                }

                ".end" => {
                    Ok(Token::End)
                }
                
                ".stringz" => {
                    let s = self.read_string()?;
                    Ok(Token::Stringz(s.to_string()))
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
            if c == '\n' { continue; }
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
                    if WORD_DELIMETERS.contains(&c) { break; };
                },
                Err(_) => break,
            }
        }

        Ok(&self.source[start..(self.pointer - 1)]) // -1 since pointer will be after the space
    }

    fn read_next_16_bit_num(word: &str) -> Result<u16, TokenizerError> {
        let num_str = &word[1..];

        if word.starts_with('x') { // number starts with an 'x', must be hexadecimal
            match u16::from_str_radix(num_str, 16) {
                Ok(num) => Ok(num),
                Err(_) => Err(TokenizerError::InvalidNumber),
            }
        } else if word.starts_with('#') { // decimal number
            match u16::from_str_radix(num_str, 10) {
                Ok(num) => Ok(num),
                Err(_) => Err(TokenizerError::InvalidNumber),
            }
            
        }
        else {
            // default is decimal number
            match u16::from_str_radix(word, 10) {
                Ok(num) => Ok(num),
                Err(_) => Err(TokenizerError::InvalidNumber),
            }
        }
    }

    fn read_string(&mut self) -> Result<&'a str, TokenizerError> {
        let start = self.pointer+1; // skip over "
        let opened = false;

        loop {
            let c = self.next_char()?;

            if !opened && c != '"' {
                return Err(TokenizerError::ExpectedString);
            } else if c == '"' { // closing
                break;
            }

            self.pointer += 1;
        }

        Ok(&self.source[start..(self.pointer - 2)])
    }
}

