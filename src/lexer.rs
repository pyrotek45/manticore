use crate::{
    string_utils::is_string_number,
    token::{Token, TokenTypes},
};

pub struct Lexer {
    source: String,
    buffer: String,
    line_number: usize,
    row: usize,
    is_parsing_stringdq: bool,
    is_parsing_stringsq: bool,
    is_parsing_comment: bool,
    pub block_stack: Vec<Vec<Token>>,
    function_keywords: Vec<String>,
    bool_keywords: Vec<String>,
}

impl Lexer {
    pub fn new_from_file(filename: &str) -> Self {
        if let Ok(content) = std::fs::read_to_string(filename) {
            Lexer {
                source: content,
                buffer: String::new(),
                line_number: 1,
                row: 0,
                is_parsing_stringdq: false,
                is_parsing_stringsq: false,
                block_stack: vec![vec![]],
                function_keywords: vec![
                    "print".to_string(),
                    "println".to_string(),
                    "if".to_string(),
                    "call".to_string(),
                    "set".to_string(),
                    "@".to_string(),
                    "dup".to_string(),
                    "concat".to_string(),
                    "var".to_string(),
                    "exit".to_string(),
                ],
                bool_keywords: vec!["true".to_string(), "false".to_string()],
                is_parsing_comment: false,
            }
        } else {
            println!(
                "ERROR: file: {} could not be found. Exiting with error code 1",
                filename
            );
            std::process::exit(1);
        }
    }

    pub fn new_from_string(input: &str) -> Self {
        Lexer {
            source: input.to_string(),
            buffer: String::new(),
            line_number: 1,
            row: 0,
            is_parsing_stringdq: false,
            is_parsing_stringsq: false,
            block_stack: vec![vec![]],
            function_keywords: vec![
                "print".to_string(),
                "println".to_string(),
                "if".to_string(),
                "println".to_string(),
                "call".to_string(),
                "set".to_string(),
                "@".to_string(),
                "dup".to_string(),
                "concat".to_string(),
                "var".to_string(),
                "exit".to_string(),
            ],
            bool_keywords: vec!["true".to_string(), "false".to_string()],
            is_parsing_comment: false,
        }
    }

    pub fn _add_input(&mut self, input: &str) {
        self.source.push_str(input)
    }

    pub fn _clear_lexer(&mut self) {
        self.source.clear()
    }

    fn check_token(&self) -> Option<Token> {
        if !self.buffer.is_empty() {
            if is_string_number(&self.buffer) {
                return Some(Token {
                    token_type: TokenTypes::Number,
                    value: self.buffer.clone(),
                    line_number: self.line_number,
                    row: self.row - self.buffer.len(),
                    block: vec![],
                });
            } else {
                //special identifiers
                if self.function_keywords.contains(&self.buffer) {
                    return Some(Token {
                        token_type: TokenTypes::Function,
                        value: self.buffer.clone(),
                        line_number: self.line_number,
                        row: self.row - self.buffer.len(),
                        block: vec![],
                    });
                }
                if self.bool_keywords.contains(&self.buffer) {
                    return Some(Token {
                        token_type: TokenTypes::Bool,
                        value: self.buffer.clone(),
                        line_number: self.line_number,
                        row: self.row - self.buffer.len(),
                        block: vec![],
                    });
                }
                return Some(Token {
                    token_type: TokenTypes::Identifier,
                    value: self.buffer.clone(),
                    line_number: self.line_number,
                    row: self.row - self.buffer.len(),
                    block: vec![],
                });
            }
        }
        Option::None
    }

    pub fn parse(&mut self) {
        // todo add escape key
        for c in self.source.chars() {
            if self.is_parsing_stringdq {
                if c != '"' {
                    self.buffer.push(c);
                    continue;
                } else {
                    self.is_parsing_stringdq = false;
                    if let Some(vec_last) = self.block_stack.last_mut() {
                        vec_last.push(Token {
                            token_type: TokenTypes::String,
                            value: self.buffer.clone(),
                            line_number: self.line_number,
                            row: self.row,
                            block: vec![],
                        })
                    }
                    self.row += self.buffer.len() + 1;
                    self.buffer.clear();
                    continue;
                }
            }

            if self.is_parsing_stringsq {
                if c != '\'' {
                    self.buffer.push(c);
                    continue;
                } else {
                    self.is_parsing_stringsq = false;
                    if let Some(vec_last) = self.block_stack.last_mut() {
                        vec_last.push(Token {
                            token_type: TokenTypes::String,
                            value: self.buffer.clone(),
                            line_number: self.line_number,
                            row: self.row,
                            block: vec![],
                        })
                    }
                    self.row += self.buffer.len() + 1;
                    self.buffer.clear();
                    continue;
                }
            }

            if self.is_parsing_comment {
                if c != '\n' {
                    continue;
                } else {
                    self.is_parsing_comment = false;
                    continue;
                }
            }

            match c {
                '\n' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }

                    if let Some(vec_last) = self.block_stack.last_mut() {
                        vec_last.push(Token {
                            token_type: TokenTypes::NewLine,
                            value: "newline".to_string(),
                            line_number: self.line_number,
                            row: self.row,
                            block: vec![],
                        })
                    }

                    self.line_number += 1;
                    self.row = 0;
                    continue;
                }
                '#' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }
                    self.is_parsing_comment = true;
                }
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                    self.buffer.push(c);
                }

                ' ' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }
                }

                '+' | '-' | '*' | '/' | '(' | ')' | '[' | ']' | '<' | '>' | '`' | '~' | '!'
                | '@' | '$' | '%' | '^' | '&' | ',' | '?' | ';' | ':' | '=' | '.' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }

                    if let Some(vec_last) = self.block_stack.last_mut() {
                        vec_last.push(Token {
                            token_type: TokenTypes::Symbol,
                            value: c.to_string(),
                            line_number: self.line_number,
                            row: self.row,
                            block: vec![],
                        })
                    }
                }

                '"' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }
                    self.is_parsing_stringdq = true;
                }
                '\'' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }
                    self.is_parsing_stringsq = true;
                }
                '{' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }

                    self.block_stack.push(vec![]);
                }
                '}' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    };

                    if let Some(list) = self.block_stack.pop() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(Token {
                                token_type: TokenTypes::Block,
                                value: "block".to_string(),
                                line_number: self.line_number,
                                row: self.row,
                                block: list,
                            })
                        }
                    }
                }
                _ => println!("ERROR: {} is not defined. Line {1}", c, self.line_number),
            }
            self.row += 1;
        }

        if let Some(t) = self.check_token() {
            if let Some(vec_last) = self.block_stack.last_mut() {
                vec_last.push(t)
            }
            self.buffer.clear();
        };

        // for t in &self.block_stack[0] {
        //     println!(
        //         "token type: {:?} token value: {1} token line number: {2} ",
        //         t.token_type, t.value, t.line_number
        //     )
        // }
    }
}
