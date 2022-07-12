use crate::{
    string_utils::is_string_number,
    token::{Token, TokenTypes},
};

pub struct Lexer {
    pub source: String,
    buffer: String,
    line_number: usize,
    row: usize,
    is_parsing_stringdq: bool,
    is_parsing_stringsq: bool,
    is_parsing_comment: bool,
    pub block_stack: Vec<Vec<Token>>,
    function_keywords: Vec<String>,
    bool_keywords: Vec<String>,
    is_skip: bool,
}
fn manticore_functions() -> Vec<String> {
    vec![
        // basic output
        "print".to_string(),
        "println".to_string(),
        "flush".to_string(),
        // program flow
        "if".to_string(),
        // block control
        "call".to_string(),
        "@".to_string(),
        //".".to_string(),
        "ret".to_string(),
        "let".to_string(),
        // stack functions
        "dup".to_string(),
        "rev".to_string(),
        "shc".to_string(),
        "rm".to_string(),
        "sec".to_string(),
        // string function
        "concat".to_string(),
        // heap control
        "set".to_string(),
        "var".to_string(),
        "=".to_string(),
        // basic repl control
        "exit".to_string(),
        // math functions
        "neg".to_string(),
        "sqrt".to_string(),
        "pow".to_string(),
        // list functions
        "range".to_string(),
        // loop functions
        "for".to_string(),
        "loop".to_string(),
        // url
        "run_url".to_string(),
        "store_url".to_string(),
        "import_url".to_string(),
        // import
        "import".to_string(),
        "store_import".to_string(),
        // os control
        "command".to_string(),
        // vm function
        "break".to_string(),
        // boolean op
        "and".to_string(),
        "or".to_string(),
        "not".to_string(),
        "equ".to_string(),
        "gtr".to_string(),
        "lss".to_string(),
        // input
        "readln".to_string(),
        // random function
        "randomf".to_string(),
        "random_int".to_string(),
        // token
        "exist".to_string(),
        // list commands
        "push".to_string(),
        "pop".to_string(),
        "insert".to_string(),
        "remove".to_string(),
        "append".to_string(),
    ]
}

impl Lexer {
    // Creates a lexer using the file as input
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
                function_keywords: manticore_functions(),
                bool_keywords: vec!["true".to_string(), "false".to_string()],
                is_parsing_comment: false,
                is_skip: false,
            }
        } else {
            println!(
                "ERROR: file: {} could not be found. Exiting with error code 1",
                filename
            );
            std::process::exit(1);
        }
    }

    // Creates a lexer using a string as input
    pub fn new_from_string(input: &str) -> Self {
        Lexer {
            source: input.to_string(),
            buffer: String::new(),
            line_number: 1,
            row: 0,
            is_parsing_stringdq: false,
            is_parsing_stringsq: false,
            block_stack: vec![vec![]],
            function_keywords: manticore_functions(),
            bool_keywords: vec!["true".to_string(), "false".to_string()],
            is_parsing_comment: false,
            is_skip: false,
        }
    }

    // Currently unused
    pub fn _add_input(&mut self, input: &str) {
        self.source.push_str(input)
    }

    // Currently unused
    pub fn _clear_lexer(&mut self) {
        self.source.clear()
    }

    // This function is used to check to see if the current
    // buffer is either a (number,function,bool,identifier)
    fn check_token(&self) -> Option<Token> {
        // Checking if buffer is numerical
        if !self.buffer.is_empty() {
            if is_string_number(&self.buffer) {
                return Some(Token {
                    token_type: TokenTypes::Number,
                    value: self.buffer.clone(),
                    line_number: self.line_number,
                    row: self.row - self.buffer.len(),
                    block: vec![],
                    proxy: None,
                });
            } else {
                // Checking if buffer is a function
                if self.function_keywords.contains(&self.buffer) {
                    return Some(Token {
                        token_type: TokenTypes::Function,
                        value: self.buffer.clone(),
                        line_number: self.line_number,
                        row: self.row - self.buffer.len(),
                        block: vec![],
                        proxy: None,
                    });
                }
                // Checking if buffer is a bool
                if self.bool_keywords.contains(&self.buffer) {
                    return Some(Token {
                        token_type: TokenTypes::Bool,
                        value: self.buffer.clone(),
                        line_number: self.line_number,
                        row: self.row - self.buffer.len(),
                        block: vec![],
                        proxy: None,
                    });
                }

                // If none of the others, return an identifier
                if self.buffer == "_" {
                    return Some(Token {
                        token_type: TokenTypes::Nothing,
                        value: "".to_string(),
                        line_number: self.line_number,
                        row: self.row - self.buffer.len(),
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    return Some(Token {
                        token_type: TokenTypes::Identifier,
                        value: self.buffer.clone(),
                        line_number: self.line_number,
                        row: self.row - self.buffer.len(),
                        block: vec![],
                        proxy: None,
                    })
                }
            }
        }
        Option::None
    }

    // Going through each char in the file or string
    pub fn parse(&mut self) {
        // Parsing strings double quote
        for c in self.source.chars() {
            if self.is_parsing_stringdq {
                if c == '/' {
                    self.is_skip = true;
                    continue;
                }
                if c != '"' || self.is_skip {
                    self.buffer.push(c);
                    if self.is_skip {
                        self.is_skip = false;
                    }
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
                            proxy: None,
                        })
                    }
                    self.row += self.buffer.len() + 1;
                    self.buffer.clear();
                    continue;
                }
            }

            // Parsing strings single quotes
            if self.is_parsing_stringsq {
                if c == '/' {
                    self.is_skip = true;
                    continue;
                }
                if c != '\'' || self.is_skip {
                    self.buffer.push(c);
                    if self.is_skip {
                        self.is_skip = false;
                    }
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
                            proxy: None,
                        })
                    }
                    self.row += self.buffer.len() + 1;
                    self.buffer.clear();
                    continue;
                }
            }

            // Parsing comments
            if self.is_parsing_comment {
                if c != '\n' {
                    continue;
                } else {
                    self.is_parsing_comment = false;
                    continue;
                }
            }

            // Main parsing function going through each char and adding them to a buffer
            // if no match is found
            match c {
                // Newline
                '\n' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }

                    self.line_number += 1;
                    self.row = 0;
                    continue;
                }

                // Comment
                '#' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }
                    self.is_parsing_comment = true;
                }

                // Letters and numbers
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                    self.buffer.push(c);
                }

                // Spaces
                ' ' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }
                }
                '.' => {
                    self.buffer.push(c);
                }

                // Symbols
                '+' | '-' | '*' | '/' | '(' | ')' | '<' | '>' | '`' | '~' | '@' | '$' | '%'
                | '^' | '&' | ',' | '?' | ';' | ':' | '=' => {
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
                            proxy: None,
                        })
                    }
                }

                // Double quotes (start parsing a string)
                '"' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }
                    self.is_parsing_stringdq = true;
                }

                // Single quotes (starts parsing a string)
                '\'' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }
                    self.is_parsing_stringsq = true;
                }

                // Parsing blocks
                '{' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }

                    self.block_stack.push(vec![]);

                    if let Some(vec_last) = self.block_stack.last_mut() {
                        vec_last.push(Token {
                            token_type: TokenTypes::Break,
                            value: "break".to_string(),
                            line_number: self.line_number,
                            row: self.row,
                            block: vec![],
                            proxy: None,
                        })
                    }
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
                                proxy: None,
                            })
                        }
                    }
                }

                // Parsing blocks
                '[' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }

                    self.block_stack.push(vec![]);
                }

                ']' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    };

                    if let Some(list) = self.block_stack.pop() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(Token {
                                token_type: TokenTypes::List,
                                value: "list".to_string(),
                                line_number: self.line_number,
                                row: self.row,
                                block: list,
                                proxy: None,
                            })
                        }
                    }
                }

                _ => println!("ERROR: {} is not defined. Line {1}", c, self.line_number),
            }
            self.row += 1;
        }

        // Add char to the buffer
        if let Some(t) = self.check_token() {
            if let Some(vec_last) = self.block_stack.last_mut() {
                vec_last.push(t)
            }
            self.buffer.clear();
        };
    }
}
