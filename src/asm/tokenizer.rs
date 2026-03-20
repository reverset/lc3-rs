
pub enum Token {
    Origin(u16),
    End(u16),
    Fill(u16),
    Blkw(u16),
    Stringz(String),
    Label(String),

    Instruction()
}

pub struct Tokenizer<'a> {
    source: &'a str,
    pointer: usize,
    tokens: Vec<Token>,
}

