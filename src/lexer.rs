use std::rc::Rc;

use colored::Colorize;

use crate::{
    string_utils::is_string_number,
    token::{BlockType, Functions, Token, Value},
};

pub struct Lexer {
    pub source: String,
    buffer: String,
    parsing_list: bool,
    line_number: usize,
    row: usize,
    is_parsing_stringdq: bool,
    is_parsing_stringsq: bool,
    is_parsing_comment: bool,
    pub block_stack: Vec<Vec<Token>>,
    //function_keywords: Vec<String>,
    is_skip: bool,
}

impl Lexer {
    // Creates a lexer using the file as input
    pub fn new_from_file(filename: &str) -> Self {
        if let Ok(content) = std::fs::read_to_string(filename) {
            Lexer {
                source: content,
                parsing_list: false,
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
            parsing_list: false,
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
            "return" => Token {
                value: Value::Function(Functions::Return),
            },
            "proc" => Token {
                value: Value::Function(Functions::Proc),
            },
            "sleep" => Token {
                value: Value::Function(Functions::Sleep),
            },
            "random" => Token {
                value: Value::Function(Functions::Random),
            },
            "exit" => Token {
                value: Value::Function(Functions::Exit),
            },
            "getch" => Token {
                value: Value::Function(Functions::Getch),
            },
            "clear" => Token {
                value: Value::Function(Functions::Clear),
            },
            "append" => Token {
                value: Value::Function(Functions::Append),
            },
            "insert" => Token {
                value: Value::Function(Functions::Insert),
            },
            "remove" => Token {
                value: Value::Function(Functions::Remove),
            },
            "pop" => Token {
                value: Value::Function(Functions::Pop),
            },
            "push" => Token {
                value: Value::Function(Functions::Push),
            },
            "continue" => Token {
                value: Value::Function(Functions::Continue),
            },
            "include" => Token {
                value: Value::Function(Functions::Include),
            },
            "let" => Token {
                value: Value::Function(Functions::Let),
            },
            "readln" => Token {
                value: Value::Function(Functions::Readln),
            },
            "pow" => Token {
                value: Value::Function(Functions::Pow),
            },
            "neg" => Token {
                value: Value::Function(Functions::Neg),
            },
            "sqrt" => Token {
                value: Value::Function(Functions::Sqrt),
            },
            "cmd" => Token {
                value: Value::Function(Functions::Command),
            },
            "dup" => Token {
                value: Value::Function(Functions::Dup),
            },
            "rec" => Token {
                value: Value::Function(Functions::Recursive),
            },
            "self" => Token {
                value: Value::Function(Functions::SelfId),
            },
            "not" => Token {
                value: Value::Function(Functions::Not),
            },
            "lss" => Token {
                value: Value::Function(Functions::Lss),
            },
            "gtr" => Token {
                value: Value::Function(Functions::Gtr),
            },
            "or" => Token {
                value: Value::Function(Functions::Or),
            },
            "and" => Token {
                value: Value::Function(Functions::And),
            },
            "flush" => Token {
                value: Value::Function(Functions::Flush),
            },
            "break" => Token {
                value: Value::Function(Functions::Break),
            },
            "print" => Token {
                value: Value::Function(Functions::Print),
            },
            "for" => Token {
                value: Value::Function(Functions::For),
            },
            "range" => Token {
                value: Value::Function(Functions::Range),
            },
            "println" => Token {
                value: Value::Function(Functions::Println),
            },
            "mod" => Token {
                value: Value::Function(Functions::Mod),
            },
            "equ" => Token {
                value: Value::Function(Functions::Equals),
            },
            "if" => Token {
                value: Value::Function(Functions::If),
            },
            "var" => Token {
                value: Value::Function(Functions::VariableAssign),
            },
            "false" => Token {
                value: Value::Bool(false),
            },
            "true" => Token {
                value: Value::Bool(true),
            },
            _ => Token {
                value: Value::Identifier(self.buffer.to_lowercase()),
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
                        });
                    }
                } else {
                    // Int
                    if let Ok(v) = self.buffer.parse() {
                        return Some(Token {
                            value: Value::Integer(v),
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
                if c == '\\' {
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
                                })
                            }
                        } else {
                            vec_last.push(Token {
                                value: Value::String(self.buffer.clone()),
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
                if c == '\\' {
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
                                })
                            }
                        } else {
                            vec_last.push(Token {
                                value: Value::String(self.buffer.clone()),
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
                    if !self.parsing_list {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            if let Some(last) = vec_last.last() {
                                if Value::Symbol(';') != last.value {
                                    vec_last.push(Token {
                                        value: Value::Symbol(';'),
                                    })
                                }
                            }
                        }
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
                    if let Some(vec_last) = self.block_stack.last_mut() {
                        if is_string_number(&self.buffer) && !(&self.buffer.contains('.')) {
                            self.buffer.push(c);
                            continue;
                        }
                    }
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t);
                            vec_last.push(Token {
                                value: Value::Function(Functions::AccessCall),
                            })
                        }
                        self.buffer.clear();
                    }
                }

                // Symbols
                '+' | '-' | '*' | '/' | '(' | ')' | '<' | '>' | '`' | '~' | '@' | '%' | '^'
                | '&' | ',' | '?' | ';' | ':' | '=' | '!' | '$' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }

                    if let Some(vec_last) = self.block_stack.last_mut() {
                        match c {
                            '-' => {
                                if let Some(last) = vec_last.pop() {
                                    match last.value {
                                        Value::Identifier(_) => {
                                            vec_last.push(last);
                                            vec_last.push(Token {
                                                value: Value::Function(Functions::Sub),
                                            });
                                            continue;
                                        }
                                        Value::Integer(_) => {
                                            vec_last.push(last);
                                            vec_last.push(Token {
                                                value: Value::Function(Functions::Sub),
                                            });
                                            continue;
                                        }
                                        Value::Float(_) => {
                                            vec_last.push(last);
                                            vec_last.push(Token {
                                                value: Value::Function(Functions::Sub),
                                            });
                                            continue;
                                        }
                                        _ => {
                                            vec_last.push(last);
                                            vec_last.push(Token {
                                                value: Value::Function(Functions::Neg),
                                            })
                                        }
                                    }
                                } else {
                                    vec_last.push(Token {
                                        value: Value::Function(Functions::Neg),
                                    })
                                }
                            }
                            '(' => {
                                if let Some(ref last) = vec_last.pop() {
                                    match &last.value {
                                        Value::Identifier(ident) => {
                                            vec_last.push(Token {
                                                value: Value::UserBlockCall(ident.clone()),
                                            });
                                            vec_last.push(Token {
                                                value: Value::Symbol(c),
                                            });
                                            continue;
                                        }
                                        Value::Symbol(')') => {
                                            vec_last.push(last.clone());
                                            vec_last.push(Token {
                                                value: Value::Function(
                                                    Functions::UserFunctionChain,
                                                ),
                                            });
                                            vec_last.push(Token {
                                                value: Value::Symbol(c),
                                            });

                                            continue;
                                        }
                                        Value::Block(BlockType::Literal(block)) => {
                                            vec_last.push(Token {
                                                value: Value::Block(BlockType::Lambda(
                                                    block.clone(),
                                                )),
                                            });
                                            vec_last.push(Token {
                                                value: Value::Symbol(c),
                                            });
                                            continue;
                                        }
                                        _ => {
                                            vec_last.push(last.clone());
                                            vec_last.push(Token {
                                                value: Value::Symbol(c),
                                            })
                                        }
                                    }
                                }
                            }
                            // '&' => {
                            //     if let Some(last) = vec_last.pop() {
                            //         match last.value {
                            //             Value::Symbol('&') => {
                            //                 vec_last.push(Token {
                            //                     value: Value::Function(Functions::UserMacroChain),
                            //                 });
                            //                 continue;
                            //             }
                            //             _ => {
                            //                 vec_last.push(last);
                            //                 vec_last.push(Token {
                            //                     value: Value::Symbol(c),
                            //                 })
                            //             }
                            //         }
                            //     }
                            // }
                            '<' => {
                                if let Some(last) = vec_last.pop() {
                                    match last.value {
                                        Value::Function(Functions::Lss) => {
                                            vec_last.push(Token {
                                                value: Value::Function(Functions::PopStack),
                                            });
                                            continue;
                                        }
                                        _ => {
                                            vec_last.push(last);
                                            vec_last.push(Token {
                                                value: Value::Function(Functions::Lss),
                                            })
                                        }
                                    }
                                }
                            }
                            '>' => {
                                if let Some(last) = vec_last.pop() {
                                    match last.value {
                                        // Value::Function(Functions::Neg) => {
                                        //     vec_last.push(Token {
                                        //         value: Value::Function(Functions::UserFunctionCall),
                                        //     });
                                        //     continue;
                                        // }
                                        Value::Function(Functions::Gtr) => {
                                            vec_last.push(Token {
                                                value: Value::Function(Functions::Dup),
                                            });
                                            continue;
                                        }
                                        _ => {
                                            vec_last.push(last);
                                            vec_last.push(Token {
                                                value: Value::Function(Functions::Gtr),
                                            })
                                        }
                                    }
                                }
                            }
                            '!' => vec_last.push(Token {
                                value: Value::Function(Functions::Not),
                            }),
                            '%' => vec_last.push(Token {
                                value: Value::Function(Functions::Mod),
                            }),
                            '/' => vec_last.push(Token {
                                value: Value::Function(Functions::Div),
                            }),
                            '*' => vec_last.push(Token {
                                value: Value::Function(Functions::Mul),
                            }),
                            '+' => vec_last.push(Token {
                                value: Value::Function(Functions::Add),
                            }),
                            // '@' => vec_last.push(Token {
                            //     value: Value::Function(Functions::UserFunctionCall),
                            // }),
                            // '$' => vec_last.push(Token {
                            //     value: Value::Function(Functions::UserMacroCall),
                            // }),
                            '~' => vec_last.push(Token {
                                value: Value::Function(Functions::FunctionVariableAssign),
                            }),
                            // '?' => vec_last.push(Token {
                            //     value: Value::Function(Functions::MacroVariableAssign),
                            // }),
                            '=' => {
                                if let Some(last) = vec_last.pop() {
                                    match last.value {
                                        Value::Function(Functions::VariableAssign) => {
                                            vec_last.push(Token {
                                                value: Value::Function(Functions::Equals),
                                            });
                                            continue;
                                        }
                                        _ => {
                                            vec_last.push(last);
                                            vec_last.push(Token {
                                                value: Value::Function(Functions::VariableAssign),
                                            })
                                        }
                                    }
                                }
                            }
                            _ => vec_last.push(Token {
                                value: Value::Symbol(c),
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
                                value: Value::Block(BlockType::Literal(Rc::new(list))),
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
                    self.parsing_list = true;
                    self.block_stack.push(vec![]);
                }

                ']' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    };

                    self.parsing_list = false;
                    if let Some(list) = self.block_stack.pop() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(Token {
                                value: Value::List(Rc::new(list)),
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

    // pub fn debug_output(&mut self,depth: usize,block: Rc<Vec<Token>>) {
    //     for t in block.iter() {
    //         //let ty = format!("{:?}", &t.value);
    //         let mut sdep = String::new();
    //         sdep.push_str("|--");
    //         for _ in 0..depth {
    //             sdep.push_str("|--")
    //         }

    //         if let Value::Block(block) = &t.value {
    //             println!(
    //                 "{}{}{}",
    //                 sdep.bright_cyan(),
    //                 "|--".bright_cyan(),
    //                 "BLOCK:".bright_cyan()
    //             );
    //             self.debug_output(depth + 1, block.clone());
    //             continue;

    //         }
    //         if let Value::List(block) = &t.value {
    //             println!(
    //                 "{}{}{}",
    //                 sdep.bright_cyan(),
    //                 "|--".bright_cyan(),
    //                 "LIST:".bright_cyan()
    //             );
    //             self.debug_output(depth + 1, block.clone());
    //             continue;

    //         }
    //         println!(
    //             "{}{} -> [{}]",
    //             sdep.bright_cyan(),
    //             t.get_value_as_string().bright_blue(),
    //             t.get_type_as_string().bright_purple()
    //         );

    //         //printstack.push_str(&("[".to_owned() + &t.get_value_as_string() + "]"));
    //         //printstack.push(' ');
    //     }
    //}
}
