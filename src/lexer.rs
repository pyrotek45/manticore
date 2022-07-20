use std::rc::Rc;

use crate::{
    string_utils::is_string_number,
    token::{Functions, Token, Value},
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
    //function_keywords: Vec<String>,
    is_skip: bool,
}
fn manticore_functions() -> Vec<String> {
    vec![
        // program flow
        "if".to_string(),
        // block control
        "call".to_string(),
        //".".to_string(),
        "ret".to_string(),
        "let".to_string(),
        // stack functions
        "dup".to_string(),
        "rev".to_string(),
        "shc".to_string(),
        "rm".to_string(),
        "sec".to_string(),
        // math functions
        "neg".to_string(),
        "sqrt".to_string(),
        "pow".to_string(),
        "mod".to_string(),
        "loop".to_string(),
        "from".to_string(),
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
        "exit".to_string(),
        // boolean op
        "and".to_string(),
        "or".to_string(),
        "not".to_string(),
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
                //function_keywords: manticore_functions(),
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
            //function_keywords: manticore_functions(),
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

    pub fn match_value(&self, value: &str) -> Token {
        match value.to_lowercase().as_ref() {
            "capture" => Token {
                value: Value::Function(Functions::Capture),
                id: None,
            },
            "let" => Token {
                value: Value::Function(Functions::Let),
                id: None,
            },
            "readln" => Token {
                value: Value::Function(Functions::Readln),
                id: None,
            },
            "pow" => Token {
                value: Value::Function(Functions::Pow),
                id: None,
            },
            "neg" => Token {
                value: Value::Function(Functions::Neg),
                id: None,
            },
            "sqrt" => Token {
                value: Value::Function(Functions::Sqrt),
                id: None,
            },
            "cmd" => Token {
                value: Value::Function(Functions::Command),
                id: None,
            },
            "dup" => Token {
                value: Value::Function(Functions::Dup),
                id: None,
            },
            "rec" => Token {
                value: Value::Function(Functions::Recursive),
                id: None,
            },
            "self" => Token {
                value: Value::Function(Functions::SelfId),
                id: None,
            },
            "not" => Token {
                value: Value::Function(Functions::Not),
                id: None,
            },
            "lss" => Token {
                value: Value::Function(Functions::Lss),
                id: None,
            },
            "gtr" => Token {
                value: Value::Function(Functions::Gtr),
                id: None,
            },
            "or" => Token {
                value: Value::Function(Functions::Or),
                id: None,
            },
            "and" => Token {
                value: Value::Function(Functions::And),
                id: None,
            },
            "flush" => Token {
                value: Value::Function(Functions::Flush),
                id: None,
            },
            "break" => Token {
                value: Value::Function(Functions::Break),
                id: None,
            },
            "print" => Token {
                value: Value::Function(Functions::Print),
                id: None,
            },
            "for" => Token {
                value: Value::Function(Functions::For),
                id: None,
            },
            "range" => Token {
                value: Value::Function(Functions::Range),
                id: None,
            },
            "println" => Token {
                value: Value::Function(Functions::Println),
                id: None,
            },
            "mod" => Token {
                value: Value::Function(Functions::Mod),
                id: None,
            },
            "equ" => Token {
                value: Value::Function(Functions::Equals),
                id: None,
            },
            "if" => Token {
                value: Value::Function(Functions::If),
                id: None,
            },
            "=" | "var" => Token {
                value: Value::Function(Functions::VariableAssign),
                id: None,
            },
            "false" => Token {
                value: Value::Bool(false),
                id: None,
            },
            "true" => Token {
                value: Value::Bool(true),
                id: None,
            },
            _ => Token {
                value: Value::Identifier(self.buffer.to_lowercase()),
                id: None,
            },
        }
    }

    // This function is used to check to see if the current
    // buffer is either a (number,function,bool,identifier)
    fn check_token(&self) -> Option<Token> {
        // Checking if buffer is numerical
        if !self.buffer.is_empty() {
            if is_string_number(&self.buffer) {
                // Float
                if self.buffer.contains('.') {
                    if let Ok(v) = self.buffer.parse() {
                        return Some(Token {
                            value: Value::Float(v),

                            id: None,
                        });
                    }
                } else {
                    // Int
                    if let Ok(v) = self.buffer.parse() {
                        return Some(Token {
                            value: Value::Integer(v),

                            id: None,
                        });
                    }
                }
            } else {
                return Some(self.match_value(&self.buffer));
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
                        if self.buffer.chars().count() == 1 {
                            if let Some(mychar) = self.buffer.chars().next() {
                                vec_last.push(Token {
                                    value: Value::Char(mychar),
                                    id: None,
                                })
                            }
                        } else {
                            vec_last.push(Token {
                                value: Value::String(self.buffer.clone()),
                                id: None,
                            })
                        }
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
                        if self.buffer.chars().count() == 1 {
                            if let Some(mychar) = self.buffer.chars().next() {
                                vec_last.push(Token {
                                    value: Value::Char(mychar),
                                    id: None,
                                })
                            }
                        } else {
                            vec_last.push(Token {
                                value: Value::String(self.buffer.clone()),
                                id: None,
                            })
                        }
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
                        match c {
                            '&' => vec_last.push(Token {
                                value: Value::Function(Functions::UserMacroCall),
                                id: None,
                            }),
                            '~' => vec_last.push(Token {
                                value: Value::Function(Functions::FunctionVariableAssign),
                                id: None,
                            }),
                            '@' => vec_last.push(Token {
                                value: Value::Function(Functions::UserFunctionCall),
                                id: None,
                            }),
                            '=' => vec_last.push(Token {
                                value: Value::Function(Functions::VariableAssign),
                                id: None,
                            }),
                            _ => vec_last.push(Token {
                                value: Value::Symbol(c),
                                id: None,
                            }),
                        }
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
                                value: Value::Block(Some(Rc::new(list))),
                                id: None,
                            });
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
                                value: Value::List(Some(Rc::new(list))),
                                id: None,
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
