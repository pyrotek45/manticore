use colored::Colorize;

use crate::{
    string_utils::is_string_number,
    token::{Token, TokenTypes},
};

pub struct Parser {
    pub operator_stack: Vec<Token>,
    pub output_stack: Vec<Token>,
    pub debug: bool,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            operator_stack: Vec::new(),
            output_stack: Vec::new(),
            debug: false,
        }
    }

    pub fn _clear(&mut self) {
        self.operator_stack.clear();
        self.output_stack.clear();
    }

    pub fn shunt(&mut self, input: &[Token]) -> &Vec<Token> {
        for token in input {
            // (Numbers, Strings, Bool) gets passed to output stack
            if token.token_type == TokenTypes::Number {
                self.output_stack.push(token.clone());
            }

            if token.token_type == TokenTypes::String {
                self.output_stack.push(token.clone());
            }
            if token.token_type == TokenTypes::Bool {
                self.output_stack.push(token.clone());
            }

            if token.token_type == TokenTypes::Break {
                self.output_stack.push(token.clone());
            }

            if token.token_type == TokenTypes::Nothing {
                self.output_stack.push(token.clone());
            }
            // List go to output stack
            if token.token_type == TokenTypes::List {
                self.output_stack.push(token.clone());
            }

            // Blocked gets passed to output stack
            // Will bring @ function if its on the operator stack
            if token.token_type == TokenTypes::Block {
                self.output_stack.push(token.clone());
                if let Some(last) = self.operator_stack.last().cloned() {
                    if last.value == "@" {
                        self.operator_stack.pop();
                        self.output_stack.push(last)
                    }
                }
            }

            // identifiers get passed to operator stack
            if token.token_type == TokenTypes::Identifier {
                self.operator_stack.push(token.clone())
            }

            // Functions get passed to operator stack
            if token.token_type == TokenTypes::Function {
                self.operator_stack.push(token.clone())
            }

            if token.token_type == TokenTypes::Symbol {
                match token.value.as_str() {
                    "," => {
                        // pop temp off if its and check if its "("
                        if let Some(temp) = self.operator_stack.pop() {
                            if temp.value == "(" {
                                if let Some(func) = self.operator_stack.last().cloned() {
                                    // if it is function pop function
                                    if func.token_type == TokenTypes::Function {
                                        self.output_stack.push(func.clone())
                                    }

                                    // if it is identifier then pop and add @
                                    if func.token_type == TokenTypes::Identifier {
                                        // Check for complex identifier
                                        if func.value.contains('.') {
                                            let mut buffer = String::new();
                                            let mut list = Vec::new();

                                            // Split by dots
                                            // hello.world
                                            for c in func.value.chars() {
                                                if c == '.' {
                                                    list.push(buffer.clone());
                                                    buffer.clear()
                                                } else {
                                                    buffer += &c.to_string();
                                                }
                                            }
                                            if !buffer.is_empty() {
                                                list.push(buffer.clone())
                                            }
                                            buffer.clear();

                                            // [hello] [world]
                                            list.reverse();

                                            // [world] [hello]
                                            // Push first item
                                            if let Some(t) = list.pop() {
                                                if !t.is_empty() {
                                                    self.output_stack.push(Token {
                                                        token_type: TokenTypes::Identifier,
                                                        value: t.to_string(),
                                                        line_number: func.line_number,
                                                        row: func.row,
                                                        block: vec![],
                                                        proxy: None,
                                                    });
                                                }
                                            }

                                            list.reverse();
                                            // [world]
                                            for t in list {
                                                if is_string_number(&t) {
                                                    self.output_stack.push(Token {
                                                        token_type: TokenTypes::Number,
                                                        value: t.to_string(),
                                                        line_number: func.line_number,
                                                        row: func.row,
                                                        block: vec![],
                                                        proxy: None,
                                                    });
                                                } else {
                                                    self.output_stack.push(Token {
                                                        token_type: TokenTypes::Identifier,
                                                        value: t.to_string(),
                                                        line_number: func.line_number,
                                                        row: func.row,
                                                        block: vec![],
                                                        proxy: None,
                                                    });
                                                }
                                                self.output_stack.push(Token {
                                                    token_type: TokenTypes::Symbol,
                                                    value: ".".to_string(),
                                                    line_number: func.line_number,
                                                    row: func.row,
                                                    block: vec![],
                                                    proxy: None,
                                                })
                                            }
                                            self.output_stack.push(Token {
                                                token_type: TokenTypes::Symbol,
                                                value: "@".to_string(),
                                                line_number: func.line_number,
                                                row: func.row,
                                                block: vec![],
                                                proxy: None,
                                            })
                                        } else {
                                            self.output_stack.push(func);
                                            self.output_stack.push(Token {
                                                token_type: TokenTypes::Symbol,
                                                value: "@".to_string(),
                                                line_number: token.line_number,
                                                row: token.row,
                                                block: vec![],
                                                proxy: None,
                                            })
                                        }
                                    }
                                }
                            }
                            // put temp back
                            self.operator_stack.push(temp)
                        }
                    }
                    "(" => {
                        self.operator_stack.push(token.clone());
                    }
                    ")" => {
                        if let Some(last) = self.operator_stack.last() {
                            if last.value == "(" {}
                        }
                        // while the last item in the operator stack is not
                        // a "(", pop off items into output stack
                        while let Some(last) = self.operator_stack.pop() {
                            if last.value != "(" {
                                if last.token_type == TokenTypes::Identifier {
                                    // Check for complex identifier
                                    if last.value.contains('.') {
                                        let mut buffer = String::new();
                                        let mut list = Vec::new();

                                        // Split by dots
                                        // hello.world
                                        for c in last.value.chars() {
                                            if c == '.' {
                                                list.push(buffer.clone());
                                                buffer.clear()
                                            } else {
                                                buffer += &c.to_string();
                                            }
                                        }
                                        if !buffer.is_empty() {
                                            list.push(buffer.clone())
                                        }
                                        buffer.clear();

                                        // [hello] [world]
                                        list.reverse();

                                        // [world] [hello]
                                        // Push first item
                                        if let Some(t) = list.pop() {
                                            if !t.is_empty() {
                                                self.output_stack.push(Token {
                                                    token_type: TokenTypes::Identifier,
                                                    value: t.to_string(),
                                                    line_number: last.line_number,
                                                    row: last.row,
                                                    block: vec![],
                                                    proxy: None,
                                                });
                                            }
                                        }
                                        list.reverse();
                                        // [world]
                                        for t in list {
                                            if is_string_number(&t) {
                                                self.output_stack.push(Token {
                                                    token_type: TokenTypes::Number,
                                                    value: t.to_string(),
                                                    line_number: last.line_number,
                                                    row: last.row,
                                                    block: vec![],
                                                    proxy: None,
                                                });
                                            } else {
                                                self.output_stack.push(Token {
                                                    token_type: TokenTypes::Identifier,
                                                    value: t.to_string(),
                                                    line_number: last.line_number,
                                                    row: last.row,
                                                    block: vec![],
                                                    proxy: None,
                                                });
                                            }
                                            self.output_stack.push(Token {
                                                token_type: TokenTypes::Symbol,
                                                value: ".".to_string(),
                                                line_number: last.line_number,
                                                row: last.row,
                                                block: vec![],
                                                proxy: None,
                                            })
                                        }
                                    } else {
                                        self.output_stack.push(last)
                                    }
                                    continue;
                                }
                                self.output_stack.push(last)
                            } else {
                                break;
                            }
                        }

                        // if last item on operator stack is a function pop
                        // this is for leapfrog TM parsing
                        if let Some(last) = self.operator_stack.pop() {
                            match last.token_type {
                                TokenTypes::Function => {
                                    self.output_stack.push(last);
                                }
                                TokenTypes::Identifier => {
                                    // Check for complex identifier
                                    if last.value.contains('.') {
                                        let mut buffer = String::new();
                                        let mut list = Vec::new();

                                        // Split by dots
                                        // hello.world
                                        for c in last.value.chars() {
                                            if c == '.' {
                                                list.push(buffer.clone());
                                                buffer.clear()
                                            } else {
                                                buffer += &c.to_string();
                                            }
                                        }
                                        if !buffer.is_empty() {
                                            list.push(buffer.clone())
                                        }
                                        buffer.clear();

                                        // [hello] [world]
                                        list.reverse();

                                        // [world] [hello]
                                        // Push first item
                                        if let Some(t) = list.pop() {
                                            if !t.is_empty() {
                                                self.output_stack.push(Token {
                                                    token_type: TokenTypes::Identifier,
                                                    value: t.to_string(),
                                                    line_number: last.line_number,
                                                    row: last.row,
                                                    block: vec![],
                                                    proxy: None,
                                                });
                                            }
                                        }

                                        list.reverse();
                                        // [world]
                                        for t in list {
                                            if is_string_number(&t) {
                                                self.output_stack.push(Token {
                                                    token_type: TokenTypes::Number,
                                                    value: t.to_string(),
                                                    line_number: last.line_number,
                                                    row: last.row,
                                                    block: vec![],
                                                    proxy: None,
                                                });
                                            } else {
                                                self.output_stack.push(Token {
                                                    token_type: TokenTypes::Identifier,
                                                    value: t.to_string(),
                                                    line_number: last.line_number,
                                                    row: last.row,
                                                    block: vec![],
                                                    proxy: None,
                                                });
                                            }
                                            self.output_stack.push(Token {
                                                token_type: TokenTypes::Symbol,
                                                value: ".".to_string(),
                                                line_number: last.line_number,
                                                row: last.row,
                                                block: vec![],
                                                proxy: None,
                                            })
                                        }
                                        self.output_stack.push(Token {
                                            token_type: TokenTypes::Symbol,
                                            value: "@".to_string(),
                                            line_number: last.line_number,
                                            row: last.row,
                                            block: vec![],
                                            proxy: None,
                                        })
                                    } else {
                                        self.output_stack.push(last);
                                        self.output_stack.push(Token {
                                            token_type: TokenTypes::Symbol,
                                            value: "@".to_string(),
                                            line_number: token.line_number,
                                            row: token.row,
                                            block: vec![],
                                            proxy: None,
                                        })
                                    }
                                }
                                _ => self.operator_stack.push(last),
                            }
                        }
                    }
                    "+" | "-" | "*" | "/" => {
                        //Pop off higher precedence before adding

                        // if last item in operator stack is not a "("
                        // and while last item precedence is > than
                        // current token precedence pop until empty
                        if let Some(temp) = self.operator_stack.last().cloned() {
                            if temp.value != "(" {
                                while let Some(op) = self.operator_stack.last().cloned() {
                                    if op.precedence() > token.precedence() {
                                        if let Some(t) = self.operator_stack.pop() {
                                            self.output_stack.push(t)
                                        }
                                    } else {
                                        break;
                                    }
                                }

                                // if operator last on the stack is of equal precedence, then pop
                                // until empty
                                while let Some(op) = self.operator_stack.last().cloned() {
                                    if op.precedence() == token.precedence()
                                        && token.is_left_associative()
                                    {
                                        if let Some(t) = self.operator_stack.pop() {
                                            self.output_stack.push(t)
                                        }
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }

                        // push token onto operator stack
                        self.operator_stack.push(token.clone());
                        continue;
                    }
                    ";" => {
                        while let Some(tok) = self.operator_stack.pop() {
                            self.output_stack.push(tok)
                        }
                    }
                    ":" => {
                        if let Some(last) = self.operator_stack.pop() {
                            match last.token_type {
                                TokenTypes::Function => {
                                    self.output_stack.push(last);
                                }
                                TokenTypes::Identifier => {
                                    // Check for complex identifier
                                    if last.value.contains('.') {
                                        let mut buffer = String::new();
                                        let mut list = Vec::new();

                                        // Split by dots
                                        // hello.world
                                        for c in last.value.chars() {
                                            if c == '.' {
                                                list.push(buffer.clone());
                                                buffer.clear()
                                            } else {
                                                buffer += &c.to_string();
                                            }
                                        }
                                        if !buffer.is_empty() {
                                            list.push(buffer.clone())
                                        }
                                        buffer.clear();

                                        // [hello] [world]
                                        list.reverse();

                                        // [world] [hello]
                                        // Push first item
                                        if let Some(t) = list.pop() {
                                            if !t.is_empty() {
                                                self.output_stack.push(Token {
                                                    token_type: TokenTypes::Identifier,
                                                    value: t.to_string(),
                                                    line_number: last.line_number,
                                                    row: last.row,
                                                    block: vec![],
                                                    proxy: None,
                                                })
                                            }
                                        }
                                        // [world]
                                        list.reverse();
                                        for t in list {
                                            if is_string_number(&t) {
                                                self.output_stack.push(Token {
                                                    token_type: TokenTypes::Number,
                                                    value: t.to_string(),
                                                    line_number: last.line_number,
                                                    row: last.row,
                                                    block: vec![],
                                                    proxy: None,
                                                });
                                            } else {
                                                self.output_stack.push(Token {
                                                    token_type: TokenTypes::Identifier,
                                                    value: t.to_string(),
                                                    line_number: last.line_number,
                                                    row: last.row,
                                                    block: vec![],
                                                    proxy: None,
                                                });
                                            }

                                            self.output_stack.push(Token {
                                                token_type: TokenTypes::Symbol,
                                                value: ".".to_string(),
                                                line_number: last.line_number,
                                                row: last.row,
                                                block: vec![],
                                                proxy: None,
                                            })
                                        }
                                    } else {
                                        self.output_stack.push(last);
                                    }
                                }
                                _ => self.output_stack.push(last),
                            }
                        }
                    }
                    "=" | "@" | ">" | "<" => self.operator_stack.push(token.clone()),
                    "~" | "?" => self.output_stack.push(token.clone()),
                    _ => {}
                }
            }
        }

        while let Some(t) = self.operator_stack.pop() {
            self.output_stack.push(t.clone());
        }

        if self.debug {
            let mut printstack: String = "".to_string();
            for t in &self.output_stack {
                let ty = format!("{:?}", &t.token_type);
                printstack.push_str(&("[".to_owned() + &t.value + " -> " + ty.as_str() + "]"));
                //printstack.push_str(&("[".to_owned() + &t.value + "]"));
                printstack.push(' ');
            }
            println!("STACK: {}", &printstack.bright_green());
        }
        &self.output_stack
    }
}
