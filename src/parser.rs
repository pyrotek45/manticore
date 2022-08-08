use std::rc::Rc;

use colored::Colorize;

use crate::token::{BlockType, Functions, Token, Value};

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

    pub fn debug_output(&mut self, depth: usize, block: Rc<Vec<Token>>) {
        for t in block.iter() {
            //let ty = format!("{:?}", &t.value);
            let mut sdep = String::new();
            sdep.push_str("|--");
            for _ in 0..depth {
                sdep.push_str("|--")
            }

            if let Value::Block(block) = &t.value {
                match block {
                    BlockType::Literal(block) => {
                        println!(
                            "{}{}{}",
                            sdep.bright_cyan(),
                            "|--".bright_cyan(),
                            "BLOCK Literal:".bright_cyan()
                        );
                        self.debug_output(depth + 1, block.clone());
                        continue;
                    }
                    BlockType::Lambda(block) => {
                        println!(
                            "{}{}{}",
                            sdep.bright_cyan(),
                            "|--".bright_cyan(),
                            "BLOCK Lambda:".bright_cyan()
                        );
                        self.debug_output(depth + 1, block.clone());
                        continue;
                    }
                    BlockType::Procedure(_) => todo!(),
                    BlockType::Struct(_) => todo!(),
                }
            }
            if let Value::List(block) = &t.value {
                println!(
                    "{}{}{}",
                    sdep.bright_cyan(),
                    "|--".bright_cyan(),
                    "LIST:".bright_cyan()
                );
                self.debug_output(depth + 1, block.clone());
                continue;
            }
            println!(
                "{}{} -> [{}]",
                sdep.bright_cyan(),
                t.get_value_as_string().bright_blue(),
                t.get_type_as_string().bright_purple()
            );

            //printstack.push_str(&("[".to_owned() + &t.get_value_as_string() + "]"));
            //printstack.push(' ');
        }
    }

    pub fn shunt(&mut self, input: &[Token]) -> Vec<Token> {
        for token in input {
            match &token.value {
                Value::Integer(_) => {
                    self.output_stack.push(token.clone());
                }
                Value::Float(_) => {
                    self.output_stack.push(token.clone());
                }
                Value::String(_) => {
                    self.output_stack.push(token.clone());
                }
                Value::Bool(_) => {
                    self.output_stack.push(token.clone());
                }
                Value::Nothing => {
                    self.output_stack.push(token.clone());
                }

                Value::Block(block) => {
                    match &block {
                        BlockType::Literal(shunted) => {
                            // Shunt blocks first time
                            let mut np = Parser::new();
                            if self.debug {
                                np.debug = true;
                            }

                            self.output_stack.push(Token {
                                value: Value::Block(BlockType::Literal(Rc::new(np.shunt(shunted)))),
                            });
                        }
                        BlockType::Lambda(shunted) => {
                            // Shunt blocks first time
                            let mut np = Parser::new();
                            if self.debug {
                                np.debug = true;
                            }

                            self.operator_stack.push(Token {
                                value: Value::Block(BlockType::Lambda(Rc::new(np.shunt(shunted)))),
                            });
                        }
                        _ => {
                            todo!()
                        }
                    }
                }
                Value::List(_) => {
                    self.output_stack.push(token.clone());
                }

                Value::Symbol(symbol) => {
                    match symbol {
                        ',' => {
                            // pop temp off if its and check if its "("
                            if let Some(temp) = self.operator_stack.pop() {
                                if temp.value == Value::Symbol('(') {
                                    if let Some(func) = self.operator_stack.last().cloned() {
                                        // if it is function pop function
                                        self.output_stack.push(func.clone())
                                    }
                                }
                                // put temp back
                                self.operator_stack.push(temp)
                            }
                        }
                        '(' => {
                            self.operator_stack.push(token.clone());
                        }
                        ')' => {
                            // while the last item in the operator stack is not
                            // a "(", pop off items into output stack
                            while let Some(last) = self.operator_stack.pop() {
                                if last.value != Value::Symbol('(') {
                                    self.output_stack.push(last)
                                } else {
                                    break;
                                }
                            }

                            // // if last item on operator stack is a function pop
                            // // this is for leapfrog TM parsing
                            if let Some(ref last) = self.operator_stack.pop() {
                                match &last.value {
                                    Value::Function(fun) => match fun {
                                        Functions::If => self.operator_stack.push(last.clone()),
                                        Functions::For => self.operator_stack.push(last.clone()),
                                        _ => {
                                            self.output_stack.push(last.clone());
                                        }
                                    },
                                    Value::UserBlockCall(_) => self.output_stack.push(last.clone()),
                                    Value::Block(BlockType::Lambda(_)) => {
                                        self.output_stack.push(last.clone())
                                    }
                                    _ => self.operator_stack.push(last.clone()),
                                }
                            }
                        }
                        ';' => {
                            while let Some(tok) = self.operator_stack.pop() {
                                if tok.value != Value::Symbol('(') {
                                    self.output_stack.push(tok)
                                } else {
                                    self.operator_stack.push(tok);
                                    break;
                                }
                            }
                        }
                        // Macros
                        // '&' => {
                        //     if let Some(token) = self.output_stack.pop() {
                        //         match token.value {
                        //             Value::Identifier(ident) => self.operator_stack.push(Token {
                        //                 value: Value::UserMacro(ident),
                        //             }),
                        //             _ => self.operator_stack.push(token),
                        //         }
                        //     }
                        // }
                        // Functions
                        // ':' => {
                        //     if let Some(token) = self.output_stack.pop() {
                        //         match token.value {
                        //             Value::Identifier(ident) => self.operator_stack.push(Token {
                        //                 value: Value::UserFunction(ident),
                        //             }),
                        //             _ => self.operator_stack.push(token),
                        //         }
                        //     }
                        // }
                        _ => self.operator_stack.push(token.clone()),
                    }
                }
                Value::Identifier(_) => {
                    self.output_stack.push(token.clone());
                    if let Some(last) = self.operator_stack.last().cloned() {
                        if let Value::Function(function) = last.value {
                            match function {
                                Functions::AccessCall => {
                                    self.operator_stack.pop();
                                    self.output_stack.push(last);
                                }
                                // Functions::UserMacroCall => {
                                //     self.operator_stack.pop();
                                //     self.output_stack.push(last);
                                // }
                                // Functions::UserFunctionCall => {
                                //     self.operator_stack.pop();
                                //     self.output_stack.push(last);
                                // }
                                _ => {
                                    continue;
                                }
                            }
                        }
                    }
                }
                Value::Function(function) => match function {
                    Functions::Add
                    | Functions::Sub
                    | Functions::Mul
                    | Functions::Div
                    | Functions::Equals
                    | Functions::VariableAssign
                    | Functions::Not
                    | Functions::Mod
                    | Functions::And
                    | Functions::Or
                    | Functions::Gtr
                    | Functions::Lss
                    | Functions::Neg
                    | Functions::UserFunctionCall => {
                        //Pop off higher precedence before adding

                        // if last item in operator stack is not a "("
                        // and while last item precedence is > than
                        // current token precedence pop until empty
                        if let Some(temp) = self.operator_stack.last().cloned() {
                            if temp.value != Value::Symbol('(') {
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
                    Functions::FunctionVariableAssign => self.output_stack.push(token.clone()),
                    Functions::StoreTemp | Functions::UserFunctionChain => self.output_stack.push(token.clone()),
                    _ => self.operator_stack.push(token.clone()),
                },
                Value::Char(_) => self.output_stack.push(token.clone()),
                Value::UserBlockCall(_) => self.operator_stack.push(token.clone()),
            }
        }

        while let Some(t) = self.operator_stack.pop() {
            self.output_stack.push(t.clone());
        }

        self.output_stack.clone()
    }
}
