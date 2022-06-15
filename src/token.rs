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
    pub token_type: TokenTypes,
    pub value: String,
    pub line_number: usize,
    pub row: usize,
    pub block: Vec<Token>,
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