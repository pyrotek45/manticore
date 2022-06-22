//use std::collections::HashMap;

use colored::Colorize;

use crate::token::{Token, TokenTypes};

pub struct Parser {
    //input_stack: Vec<Token>,
    pub operator_stack: Vec<Token>,
    pub output_stack: Vec<Token>,
    pub debug: bool,
    //heap: HashMap<String, Token>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            operator_stack: Vec::new(),
            output_stack: Vec::new(),
            debug: false,
            //heap: HashMap::new(),
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

            // Functions get passed to function stack
            if token.token_type == TokenTypes::Function {
                self.operator_stack.push(token.clone())
            }

            if token.token_type == TokenTypes::Identifier {
                self.operator_stack.push(token.clone());
            }
            // Blocked gets passed to output
            // Will bring @ and . function if its on the operator stack
            if token.token_type == TokenTypes::Block {
                self.output_stack.push(token.clone());
                if let Some(t) = self.operator_stack.last() {
                    if t.value == "@" {
                        if let Some(tok) = self.operator_stack.pop() {
                            self.output_stack.push(tok)
                        }
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
                        if let Some(t) = self.operator_stack.last() {
                            if t.value == "(" {
                                if let Some(temp) = self.operator_stack.pop() {
                                    if let Some(f) = self.operator_stack.last() {
                                        if f.token_type == TokenTypes::Function {
                                            self.output_stack
                                                .push(self.operator_stack.last().unwrap().clone())
                                        }
                                    }
                                    self.operator_stack.push(temp)
                                }
                            }
                        }
                    }
                    "(" => {
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
                        if !self.operator_stack.is_empty() {
                            while self.operator_stack.last().unwrap().value != "(" {
                                if let Some(t) = self.operator_stack.pop() {
                                    self.output_stack.push(t.to_owned());
                                }
                                if self.operator_stack.is_empty() {
                                    break;
                                }
                            }

                            if !self.operator_stack.is_empty() {
                                self.operator_stack.pop().unwrap();
                            }

                            if !self.operator_stack.is_empty()
                                && self.operator_stack.last().unwrap().token_type
                                    == TokenTypes::Function
                            {
                                if let Some(t) = self.operator_stack.pop() {
                                    self.output_stack.push(t.clone());
                                }
                            }

                            if !self.operator_stack.is_empty()
                                && self.operator_stack.last().unwrap().token_type
                                    == TokenTypes::Identifier
                            {
                                if let Some(t) = self.operator_stack.pop() {
                                    self.output_stack.push(t.clone());
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
                    }
                    "+" | "-" | "*" | "/" => {
                        //Pop off higher precedence before adding
                        if !self.operator_stack.is_empty()
                            && self.operator_stack.last().unwrap().value != "("
                        {
                            while self.operator_stack.last().unwrap().precedence()
                                > token.precedence()
                            {
                                if let Some(t) = self.operator_stack.pop() {
                                    self.output_stack.push(t.clone())
                                }
                                if self.operator_stack.is_empty() {
                                    break;
                                }
                            }

                            if !self.operator_stack.is_empty() {
                                while self.operator_stack.last().unwrap().precedence()
                                    == token.precedence()
                                    && token.is_left_associative()
                                {
                                    if let Some(t) = self.operator_stack.pop() {
                                        self.output_stack.push(t.clone())
                                    }
                                    if self.operator_stack.is_empty() {
                                        break;
                                    }
                                }
                            }
                        }
                        self.operator_stack.push(token.clone());
                        continue;
                    }
                    ";" => {
                        while !self.operator_stack.is_empty() {
                            if let Some(t) = self.operator_stack.pop() {
                                self.output_stack.push(t.clone())
                            }
                        }
                    }
                    "." => {

                    }
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
                printstack.push_str(&("[".to_owned() + &t.value + "]"));
                printstack.push(' ');
            }
            println!("STACK: {}", &printstack.bright_green());
        }
        &self.output_stack
    }
}
