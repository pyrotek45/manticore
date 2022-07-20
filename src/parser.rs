use std::rc::Rc;

use colored::Colorize;

use crate::{
    string_utils::is_string_number,
    token::{Functions, Token, Value},
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

    pub fn shunt(&mut self, input: &[Token]) -> Vec<Token> {
        for token in input {
            match token.value {
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

                Value::Block(_) => {
                    // Shunt blocks first time
                    let mut np = Parser::new();
                    if self.debug {
                        np.debug = true;
                    }
                    if let Value::Block(Some(shunted)) = &token.value {
                        self.output_stack.push(Token {
                            id: None,
                            value: Value::Block(Some(Rc::new(np.shunt(shunted)))),
                        });
                    } else {
                        self.output_stack.push(Token {
                            id: None,
                            value: Value::Block(None),
                        });
                    }

                    if let Some(last) = self.operator_stack.last().cloned() {
                        if let Value::Function(function) = last.value {
                            match function {
                                Functions::UserMacroCall => {
                                    self.operator_stack.pop();
                                    self.output_stack.push(last)
                                }
                                Functions::UserFunctionCall => {
                                    self.operator_stack.pop();
                                    self.output_stack.push(last)
                                }
                                _ => {}
                            }
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
                                        match func.value {
                                            Value::Function(_) => {
                                                self.output_stack.push(func.clone())
                                            }
                                            Value::Identifier(ref value) => {
                                                if value.contains('.') {
                                                    let mut buffer = String::new();
                                                    let mut list = Vec::new();

                                                    // Split by dots
                                                    // hello.world
                                                    for c in value.chars() {
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
                                                                value: Value::Identifier(
                                                                    t.to_string(),
                                                                ),
                                                                id: None,
                                                            });
                                                        }
                                                    }

                                                    list.reverse();
                                                    // [world]
                                                    for t in list {
                                                        if is_string_number(&t) {
                                                            if let Ok(v) = t.parse() {
                                                                self.output_stack.push(Token {
                                                                    value: Value::Integer(v),
                                                                    id: None,
                                                                });
                                                            }
                                                        } else {
                                                            self.output_stack.push(Token {
                                                                value: Value::Identifier(
                                                                    t.to_string(),
                                                                ),
                                                                id: None,
                                                            });
                                                        }
                                                        self.output_stack.push(Token {
                                                            value: Value::Function(
                                                                Functions::AccessCall,
                                                            ),
                                                            id: None,
                                                        });
                                                    }
                                                    self.output_stack.push(Token {
                                                        value: Value::Function(
                                                            Functions::UserFunctionCall,
                                                        ),
                                                        id: None,
                                                    });
                                                } else {
                                                    self.output_stack.push(func);
                                                    self.output_stack.push(Token {
                                                        value: Value::Function(
                                                            Functions::UserFunctionCall,
                                                        ),
                                                        id: None,
                                                    });
                                                }
                                            }
                                            _ => {}
                                        }
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
                            if let Some(last) = self.operator_stack.last() {
                                if last.value == Value::Symbol('(') {}
                            }
                            // while the last item in the operator stack is not
                            // a "(", pop off items into output stack
                            while let Some(last) = self.operator_stack.pop() {
                                if last.value != Value::Symbol('(') {
                                    match last.value {
                                        Value::Identifier(ref value) => {
                                            if value.contains('.') {
                                                let mut buffer = String::new();
                                                let mut list = Vec::new();

                                                // Split by dots
                                                // hello.world
                                                for c in value.chars() {
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
                                                            value: Value::Identifier(t.to_string()),
                                                            id: None,
                                                        });
                                                    }
                                                }

                                                list.reverse();
                                                // [world]
                                                for t in list {
                                                    if is_string_number(&t) {
                                                        if let Ok(v) = t.parse() {
                                                            self.output_stack.push(Token {
                                                                value: Value::Integer(v),
                                                                id: None,
                                                            });
                                                        }
                                                    } else {
                                                        self.output_stack.push(Token {
                                                            value: Value::Identifier(t.to_string()),
                                                            id: None,
                                                        });
                                                    }
                                                    self.output_stack.push(Token {
                                                        value: Value::Function(
                                                            Functions::AccessCall,
                                                        ),
                                                        id: None,
                                                    });
                                                }
                                            } else {
                                                self.output_stack.push(last)
                                            }
                                            continue;
                                        }
                                        _ => self.output_stack.push(last),
                                    }
                                } else {
                                    break;
                                }
                            }

                            // if last item on operator stack is a function pop
                            // this is for leapfrog TM parsing
                            if let Some(ref last) = self.operator_stack.pop() {
                                match &last.value {
                                    Value::Function(_) => {
                                        self.output_stack.push(last.clone());
                                    }
                                    Value::Identifier(value) => {
                                        // Check for complex identifier
                                        if value.contains('.') {
                                            let mut buffer = String::new();
                                            let mut list = Vec::new();

                                            // Split by dots
                                            // hello.world
                                            for c in value.chars() {
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
                                                        value: Value::Identifier(t.to_string()),

                                                        id: None,
                                                    });
                                                }
                                            }

                                            list.reverse();
                                            // [world]
                                            for t in list {
                                                if is_string_number(&t) {
                                                    if let Ok(v) = t.parse() {
                                                        self.output_stack.push(Token {
                                                            value: Value::Integer(v),

                                                            id: None,
                                                        });
                                                    }
                                                } else {
                                                    self.output_stack.push(Token {
                                                        value: Value::Identifier(t.to_string()),

                                                        id: None,
                                                    });
                                                }
                                                self.output_stack.push(Token {
                                                    value: Value::Function(Functions::AccessCall),
                                                    id: None,
                                                });
                                            }
                                            self.output_stack.push(Token {
                                                value: Value::Function(Functions::UserFunctionCall),

                                                id: None,
                                            });
                                        } else {
                                            self.output_stack.push(last.clone());
                                            self.output_stack.push(Token {
                                                value: Value::Function(Functions::UserFunctionCall),
                                                id: None,
                                            });
                                        }
                                    }
                                    _ => self.operator_stack.push(last.clone()),
                                }
                            }
                        }
                        '+' | '-' | '*' | '/' => {
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
                        ';' => {
                            while let Some(tok) = self.operator_stack.pop() {
                                self.output_stack.push(tok)
                            }
                        }
                        ':' => {
                            if let Some(ref last) = self.operator_stack.pop() {
                                match &last.value {
                                    Value::Function(_) => {
                                        self.output_stack.push(last.clone());
                                    }
                                    Value::Identifier(value) => {
                                        // Check for complex identifier
                                        if value.contains('.') {
                                            let mut buffer = String::new();
                                            let mut list = Vec::new();

                                            // Split by dots
                                            // hello.world
                                            for c in value.chars() {
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
                                                        value: Value::Identifier(t.to_string()),

                                                        id: None,
                                                    });
                                                }
                                            }

                                            list.reverse();
                                            // [world]
                                            for t in list {
                                                if is_string_number(&t) {
                                                    if let Ok(v) = t.parse() {
                                                        self.output_stack.push(Token {
                                                            value: Value::Integer(v),

                                                            id: None,
                                                        });
                                                    }
                                                } else {
                                                    self.output_stack.push(Token {
                                                        value: Value::Identifier(t.to_string()),

                                                        id: None,
                                                    });
                                                }
                                                self.output_stack.push(Token {
                                                    value: Value::Function(Functions::AccessCall),
                                                    id: None,
                                                });
                                            }
                                        } else {
                                            self.output_stack.push(last.clone())
                                        }
                                    }
                                    _ => self.output_stack.push(last.clone()),
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Value::Identifier(_) => {
                    self.operator_stack.push(token.clone());
                    if let Some(last) = self.operator_stack.last().cloned() {
                        if let Value::Function(function) = last.value {
                            match function {
                                Functions::UserMacroCall => {
                                    self.operator_stack.pop();
                                    self.output_stack.push(last);
                                }
                                Functions::UserFunctionCall => {
                                    self.operator_stack.pop();
                                    self.output_stack.push(last);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Value::Function(function) => match function {
                    Functions::FunctionVariableAssign => self.output_stack.push(token.clone()),
                    _ => self.operator_stack.push(token.clone()),
                },
                Value::Char(_) => self.output_stack.push(token.clone()),
            }
        }

        while let Some(t) = self.operator_stack.pop() {
            self.output_stack.push(t.clone());
        }

        if self.debug {
            let mut printstack: String = "".to_string();
            for t in &self.output_stack {
                //let ty = format!("{:?}", &t.value);
                printstack.push_str(
                    &("[".to_owned()
                        + &t.get_value_as_string()
                        + " -> "
                        + &t.get_type_as_string()
                        + "]"),
                );
                //printstack.push_str(&("[".to_owned() + &t.get_value_as_string() + "]"));
                printstack.push(' ');
            }
            println!("STACK: {}", &printstack.bright_green());
        }
        self.output_stack.clone()
    }
}
