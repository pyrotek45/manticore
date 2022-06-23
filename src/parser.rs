use colored::Colorize;

use crate::token::{Token, TokenTypes};

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

            // Functions get passed to operator stack
            if token.token_type == TokenTypes::Function {
                self.operator_stack.push(token.clone())
            }

            if token.token_type == TokenTypes::Identifier {
                self.operator_stack.push(token.clone());
            }

            // Blocked gets passed to output stack
            // Will bring @ function if its on the operator stack
            if token.token_type == TokenTypes::Block {
                self.output_stack.push(token.clone());
                if let Some(last) = self.operator_stack.last().cloned() {
                    if last.value == "@" {
                        self.output_stack.push(last)
                    }
                }
            }

            // List go to output stack
            if token.token_type == TokenTypes::List {
                self.output_stack.push(token.clone());
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
                                        self.output_stack.push(func.clone());
                                        self.output_stack.push(Token {
                                            token_type: TokenTypes::Symbol,
                                            value: "@".to_string(),
                                            line_number: 0,
                                            row: 0,
                                            block: vec![],
                                            proxy: None,
                                        })
                                    }
                                }
                            }
                            // put temp back
                            self.operator_stack.push(temp)
                        }
                    }
                    "(" => {
                        // if last function is set, set break point
                        if let Some(tok) = self.operator_stack.last() {
                            if tok.value == "set" {
                                self.output_stack.push(Token {
                                    token_type: TokenTypes::Break,
                                    value: "break".to_string(),
                                    line_number: 0,
                                    row: 0,
                                    block: vec![],
                                    proxy: None,
                                })
                            }
                        }
                        self.operator_stack.push(token.clone());
                    }
                    ")" => {
                        // while the last item in the operator stack is not
                        // a "(", pop off items into output stack
                        while let Some(last) = self.operator_stack.pop() {
                            if last.value != "(" {
                                self.output_stack.push(last)
                            } else {
                                //self.operator_stack.push(last);
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
                                    self.output_stack.push(last);
                                    self.output_stack.push(Token {
                                        token_type: TokenTypes::Symbol,
                                        value: "@".to_string(),
                                        line_number: 0,
                                        row: 0,
                                        block: vec![],
                                        proxy: None,
                                    })
                                }
                                _ => {
                                    self.operator_stack.push(last)
                                }
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
                    "." => {}
                    ":" => {
                        if let Some(t) = self.operator_stack.pop() {
                            self.output_stack.push(t.clone())
                        }
                    }
                    "=" | "@" => self.operator_stack.push(token.clone()),
                    _ => {}
                }
            }
        }

        while let Some(t) = self.operator_stack.pop() {
            self.output_stack.push(t.clone())
        }

        if self.debug {
            let mut printstack: String = "".to_string();
            for t in &self.output_stack {
                //let ty = format!("{:?}", &t.token_type);
                //printstack.push_str(&("[".to_owned() + &t.value + " : " + ty.as_str() + "]"));
                printstack.push_str(&("[".to_owned() + &t.value + "]"));
                printstack.push(' ');
            }
            println!("STACK: {}", &printstack.bright_green());
        }
        &self.output_stack
    }
}
