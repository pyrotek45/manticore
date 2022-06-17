#[derive(Debug, Clone, Copy, std::cmp::PartialEq)]
pub enum TokenTypes {
    Block,
    Function,
    Identifier,
    Number,
    String,
    Symbol,
    NewLine,
    Bool,
}

#[derive(Clone)]
pub struct Token {
    // Used for actual work
    pub proxy: Option<String>,
    pub token_type: TokenTypes,
    pub value: String,
    pub block: Vec<Token>,

    // Used for debuging
    pub line_number: usize,
    pub row: usize,
}

impl Token {
    pub fn precedence(&self) -> usize {
        match self.value.chars().next() {
            Some('+') | Some('-') => 4,
            Some('*') | Some('/') => 5,
            _ => 0,
        }
    }

    pub fn is_left_associative(&self) -> bool {
        match self.value.chars().next() {
            Some('+') | Some('-') => true,
            Some('*') | Some('/') => true,
            _ => true,
        }
    }
}
