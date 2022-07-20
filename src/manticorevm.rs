use core::fmt;
use hashbrown::HashMap;
use modulo::Mod;
use rand::Rng;
use std::borrow::BorrowMut;
use std::ops::Deref;
use std::process::Command;
use std::rc::Rc;
use std::{io::Write, vec};

use crate::{
    lexer::{self},
    parser::Parser,
    string_utils::{is_string_number, print_error, trim_newline},
    token::{Functions, Token, Value},
};

pub struct ManitcoreVm {
    vm_instruction_: Vec<Token>,
    pub execution_stack: Vec<Token>,
    last_instruction: String,
    file: String,
    pub exit_loop: bool,
    pub debug: bool,
    pub method_call: bool,
    pub core_self: Vec<Token>,
    call_stack: Vec<HashMap<String, Token>>,
}

impl ManitcoreVm {
    pub fn new(tokenlist: &[Token], file: &str) -> Self {
        Self {
            vm_instruction_: tokenlist.to_vec(),
            execution_stack: Vec::with_capacity(1024),
            file: file.to_string(),
            last_instruction: String::new(),
            debug: false,
            exit_loop: false,
            method_call: false,
            core_self: vec![],
            call_stack: vec![HashMap::new()],
        }
    }

    // Proccess each token
    pub fn execute(&mut self) {
        for i in self.vm_instruction_.clone() {
            // If token is an identifier and its value is found on the call_stack
            // push the call_stack value instead
            self.execute_token(&i);
        }

        if self.debug {
            if let Some(scope) = self.call_stack.last() {
                for (k, v) in scope {
                    if let Some(p) = &v.id {
                        println!("{} -> ({} ~ {})", k, v.get_value_as_string(), &p)
                    } else {
                        println!("{} -> ({} ~ None)", k, v.get_value_as_string())
                    }
                }
                for tok in &self.execution_stack {
                    print!("[{}] ", tok.get_value_as_string())
                }
                println!()
            }
        }
    }

    // Send each token to correct function
    pub fn execute_token(&mut self, token: &Token) {
        match &token.value {
            Value::Integer(_) => {
                self.execution_stack.push(token.clone());
            }
            Value::Float(_) => {
                self.execution_stack.push(token.clone());
            }
            Value::String(_) => {
                self.execution_stack.push(token.clone());
            }
            Value::Symbol(value) => match value {
                '*' => self.stack_mul(),
                //'/' => {self.stack_div()},
                '-' => self.stack_sub(),
                '+' => self.stack_add(),
                _ => {}
            },
            Value::Bool(_) => {
                self.execution_stack.push(token.clone());
            }
            Value::Block(_) => {
                self.execution_stack.push(token.clone());
            }
            Value::List(_) => {
                self.execution_stack.push(token.clone());
            }

            Value::Identifier(value) => {
                if let Some(scope) = self.call_stack.last_mut() {
                    if let Some(token) = scope.get(value) {
                        self.execution_stack.push(token.clone());
                        return;
                    }
                }
                self.execution_stack.push(token.clone());
            }
            Value::Function(function) => match function {
                Functions::VariableAssign => self.variable_assign(),
                Functions::Equals => self.equ_comparison(),
                Functions::UserFunctionCall => self.user_function_call(),
                Functions::Println => self.stack_println(),
                Functions::Range => self.create_range(),
                Functions::Mod => self.stack_modulo(),
                Functions::For => self.for_loop(),
                Functions::If => self.if_statement(),
                Functions::Print => self.stack_print(),
                Functions::Break => self.break_loop(),
                Functions::FunctionVariableAssign => self.function_variable_assign(),
                Functions::Flush => self.flush(),
                Functions::And => self.logical_and(),
                Functions::Or => self.logical_or(),
                Functions::Not => self.logical_not(),
                Functions::Gtr => self.gtr_comparison(),
                Functions::Lss => self.lss_comparison(),
                Functions::SelfId => self.get_self(),
                Functions::AccessCall => self.access_call(),
                Functions::UserMacroCall => self.user_macro_call(),
                Functions::Readln => self.readln(),
                Functions::Neg => self.neg(),
                Functions::Pow => self.pow(),
                Functions::Sqrt => self.sqrt(),
                Functions::Match => todo!(),
                Functions::Let => self.closure_let(),
                Functions::Ret => todo!(),
                Functions::Random => todo!(),
                Functions::Command => self.command(),
                Functions::Exit => todo!(),
                Functions::Recursive => self.recursive(),
                Functions::Dup => self.dup(),
                Functions::Capture => todo!(),
            },
            Value::Nothing => {
                todo!()
            }
            Value::Char(_) => self.execution_stack.push(token.clone()),
        }
    }

    fn closure_let(&mut self) {
        if let Some(block) = self.execution_stack.pop() {
            if let Some(scope) = self.call_stack.last_mut() {

                let mut core_self = vec![];

                for (ident, token) in scope {

                    core_self.push(Token {
                        id: None,
                        value: Value::Identifier(ident.clone()),
                    });
                    core_self.push(Token {
                        id: None,
                        value: token.value.clone(),
                    });
                    core_self.push(Token {
                        id: None,
                        value: Value::Function(Functions::VariableAssign),
                    })

                }
                if let Value::Block(Some(block)) = block.value {
                    for t in block.iter() {
                        core_self.push(t.clone())
                    }
                    self.execution_stack.push(Token {
                        id: None,
                        value: Value::Block(Some(Rc::new(core_self))),
                    })
                }

            }

        }
    }
    

    fn get_self(&mut self) {
        if let Some(scope) = self.call_stack.last_mut() {
            let mut core_self = vec![];
            for (ident, token) in scope {
                core_self.push(Token {
                    id: None,
                    value: Value::Identifier(ident.clone()),
                });
                core_self.push(Token {
                    id: None,
                    value: token.value.clone(),
                });
                core_self.push(Token {
                    id: None,
                    value: Value::Function(Functions::VariableAssign),
                })
            }
            self.execution_stack.push(Token {
                id: None,
                value: Value::Block(Some(Rc::new(core_self))),
            })
        }
    }

    fn command(&mut self) {
        if let (Some(args), Some(command)) =
            (self.execution_stack.pop(), self.execution_stack.pop())
        {
            match (args.value, command.value) {
                (Value::List(Some(args)), Value::String(command)) => {
                    let mut cargs = vec![];
                    for arg in args.iter() {
                        if let Value::String(arg) = &arg.value {
                            cargs.push(arg.clone())
                        }
                    }

                    Command::new(&command)
                        .args(cargs)
                        .spawn()
                        .unwrap_or_else(|_| panic!(" {} command failed to start", &command));
                }
                _ => {
                    println!("cant start that command")
                }
            }
        }
    }

    fn dup(&mut self) {
        if let Some(token) = self.execution_stack.pop() {
            self.execution_stack.push(token.clone());
            self.execution_stack.push(token);
        }
    }

    fn function_variable_assign(&mut self) {
        let mut variable_stack: Vec<String> = Vec::with_capacity(10);

        // Pop from stack untill no more identifiers
        while let Some(token) = self.execution_stack.last() {
            if let Value::Identifier(_) = token.value {
                if let Some(token) = self.execution_stack.pop() {
                    if let Value::Identifier(ident) = &token.value {
                        variable_stack.push(ident.clone())
                    }
                }
            } else {
                break;
            }
        }

        // Tie each value into the call_stack using the tokens poped
        if let Some(scope) = self.call_stack.last_mut() {
            for values in variable_stack {
                if let Some(mut tok) = self.execution_stack.pop() {
                    tok.id = Some(values.clone());
                    scope.insert(values, tok.clone());
                }
            }
        }
    }

    fn break_loop(&mut self) {
        self.exit_loop = true;
    }

    fn logical_not(&mut self) {
        if let Some(token) = self.execution_stack.pop() {
            if let Value::Bool(bool) = token.value {
                self.execution_stack.push(Token {
                    id: None,
                    value: Value::Bool(!bool),
                })
            }
        }
    }

    fn capture(&mut self) {
        // TODO
    }


    fn recursive(&mut self) {
        if let (Some(tblock), Some(id)) = (self.execution_stack.pop(), self.execution_stack.pop()) {
            match (tblock.value.clone(), id.value) {
                (Value::Block(Some(block)), Value::Identifier(ident)) => {
                    let mut recurse = vec![
                        Token {
                            id: None,
                            value: Value::Identifier(ident.clone()),
                        },
                        Token {
                            id: None,
                            value: Value::Identifier(ident.clone()),
                        },
                        tblock,
                        Token {
                            id: None,
                            value: Value::Function(Functions::Recursive),
                        },
                        Token {
                            id: None,
                            value: Value::Function(Functions::VariableAssign),
                        },
                    ];
                    for t in block.iter() {
                        recurse.push(t.clone())
                    }
                    self.execution_stack.push(Token {
                        id: Some(ident),
                        value: Value::Block(Some(Rc::new(recurse))),
                    })
                }
                _ => {
                    println!("cant recursion these values")
                }
            }
        }
    }

    fn logical_and(&mut self) {
        if let (Some(right), Some(left)) = (self.execution_stack.pop(), self.execution_stack.pop())
        {
            match (right.value, left.value) {
                (Value::Bool(right), Value::Bool(left)) => self.execution_stack.push(Token {
                    id: None,
                    value: Value::Bool(left && right),
                }),
                _ => {
                    println!("cant && these values")
                }
            }
        }
    }

    fn logical_or(&mut self) {
        if let (Some(right), Some(left)) = (self.execution_stack.pop(), self.execution_stack.pop())
        {
            match (right.value, left.value) {
                (Value::Bool(right), Value::Bool(left)) => self.execution_stack.push(Token {
                    id: None,
                    value: Value::Bool(left || right),
                }),
                _ => {
                    println!("cant && these values")
                }
            }
        }
    }

    fn equ_comparison(&mut self) {
        if let (Some(right), Some(left)) = (self.execution_stack.pop(), self.execution_stack.pop())
        {
            self.execution_stack.push(Token {
                id: None,
                value: Value::Bool(left.value == right.value),
            })
        }
    }

    fn variable_assign(&mut self) {
        if let (Some(mut value), Some(ident)) =
            (self.execution_stack.pop(), self.execution_stack.pop())
        {
            if let Some(scope) = self.call_stack.last_mut() {
                if let Value::Identifier(identifier) = ident.value {
                    value.id = Some(identifier.clone());
                    scope.insert(identifier, value);
                } else if let Some(proxy) = ident.id {
                    value.id = Some(proxy.clone());
                    scope.insert(proxy, value);
                }
            }
        }
    }

    fn if_statement(&mut self) {
        if let (Some(block), Some(boolmaybe)) =
            (self.execution_stack.pop(), self.execution_stack.pop())
        {
            // use as match statement
            // if let Value::Block(Some(block)) = block.value {
            //     for (i,t) in block.iter().enumerate() {
            //         if t.value == expr.value {

            //             if let Value::Block(Some(block)) = &block[i + 1].value {
            //                 for it in &**block {
            //                     self.execute_token(it);
            //                 }
            //                 break;
            //             }
            //         } else {
            //             continue;
            //         }

            //     }
            // }

            //if true single if statement
            if let Value::Bool(bool) = boolmaybe.value {
                if bool {
                    if let Value::Block(Some(block)) = block.value {
                        for t in &*block {
                            self.execute_token(t)
                        }
                    }
                }
            } else if let Some(bool2) = self.execution_stack.pop() {
                if let Value::Bool(bool) = bool2.value {
                    if bool {
                        if let Value::Block(Some(block)) = boolmaybe.value {
                            for t in &*block {
                                self.execute_token(t)
                            }
                        }
                    } else if let Value::Block(Some(block)) = block.value {
                        for t in &*block {
                            self.execute_token(t)
                        }
                    }
                }
            }
        }
    }

    fn create_range(&mut self) {
        if let (Some(end), Some(start)) = (self.execution_stack.pop(), self.execution_stack.pop()) {
            match (start.value, end.value) {
                (Value::Integer(start), Value::Integer(end)) => {
                    let mut new_list: Vec<Token> = Vec::new();
                    for x in start..=end {
                        new_list.push(Token {
                            id: None,
                            value: Value::Integer(x),
                        });
                    }
                    self.execution_stack.push(Token {
                        id: None,
                        value: Value::List(Some(Rc::new(new_list.to_vec()))),
                    })
                }
                _ => {
                    println!("cant make a range from these types");
                }
            }
        }
    }

    fn for_loop(&mut self) {
        if let (Some(block), Some(list), Some(variable)) = (
            self.execution_stack.pop(),
            self.execution_stack.pop(),
            self.execution_stack.pop(),
        ) {
            match (block.value, list.value, &variable.value) {
                (
                    Value::Block(Some(ref block)),
                    Value::Block(Some(list)),
                    Value::Identifier(variable_name),
                ) => {
                    'outer: for variable in &*list {
                        if let Some(scope) = self.call_stack.last_mut() {
                            scope.insert(variable_name.to_string(), variable.clone());
                        }
                        for t in block.iter() {
                            self.execute_token(t);
                            if self.exit_loop {
                                break 'outer;
                            }
                        }
                    }
                    if let Some(scope) = self.call_stack.last_mut() {
                        scope.remove(variable_name);
                    }
                    self.exit_loop = false;
                }
                (
                    Value::Block(Some(ref block)),
                    Value::List(Some(list)),
                    Value::Identifier(variable_name),
                ) => {
                    'outer1: for variable in &*list {
                        if let Some(scope) = self.call_stack.last_mut() {
                            scope.insert(variable_name.to_string(), variable.clone());
                        }
                        for t in block.iter() {
                            self.execute_token(t);
                            if self.exit_loop {
                                break 'outer1;
                            }
                        }
                    }
                    if let Some(scope) = self.call_stack.last_mut() {
                        scope.remove(variable_name);
                    }
                    self.exit_loop = false;
                }
                (
                    Value::Block(Some(ref block)),
                    Value::String(string),
                    Value::Identifier(variable_name),
                ) => {
                    for variable in string.chars() {
                        if let Some(scope) = self.call_stack.last_mut() {
                            scope.insert(
                                variable_name.to_string(),
                                Token {
                                    id: None,
                                    value: Value::Char(variable),
                                },
                            );
                        }
                        for t in block.iter() {
                            self.execute_token(t)
                        }
                        if self.exit_loop {
                            break;
                        }
                    }
                    if let Some(scope) = self.call_stack.last_mut() {
                        scope.remove(variable_name);
                    }
                    self.exit_loop = false;
                }
                _ => {
                    println!("cant make a iterate from these types");
                }
            }
        }
    }

    fn lss_comparison(&mut self) {
        if let (Some(right), Some(left)) = (self.execution_stack.pop(), self.execution_stack.pop())
        {
            match (left.value, right.value) {
                (Value::Integer(left), Value::Integer(right)) => self.execution_stack.push(Token {
                    value: Value::Bool(left < right),
                    id: None,
                }),
                (Value::Integer(ref left), Value::Float(right)) => {
                    let left = *left as f64;
                    self.execution_stack.push(Token {
                        value: Value::Bool(left < right),
                        id: None,
                    })
                }
                (Value::Float(left), Value::Float(right)) => self.execution_stack.push(Token {
                    value: Value::Bool(left < right),
                    id: None,
                }),
                (Value::Float(left), Value::Integer(ref right)) => {
                    let right = *right as f64;
                    self.execution_stack.push(Token {
                        value: Value::Bool(left < right),
                        id: None,
                    })
                }
                _ => {
                    println!("cant lss these two types");
                }
            }
        }
    }

    fn gtr_comparison(&mut self) {
        if let (Some(right), Some(left)) = (self.execution_stack.pop(), self.execution_stack.pop())
        {
            match (left.value, right.value) {
                (Value::Integer(left), Value::Integer(right)) => self.execution_stack.push(Token {
                    value: Value::Bool(left > right),
                    id: None,
                }),
                (Value::Integer(ref left), Value::Float(right)) => {
                    let left = *left as f64;
                    self.execution_stack.push(Token {
                        value: Value::Bool(left > right),
                        id: None,
                    })
                }
                (Value::Float(left), Value::Float(right)) => self.execution_stack.push(Token {
                    value: Value::Bool(left > right),
                    id: None,
                }),
                (Value::Float(left), Value::Integer(ref right)) => {
                    let right = *right as f64;
                    self.execution_stack.push(Token {
                        value: Value::Bool(left > right),
                        id: None,
                    })
                }
                _ => {
                    println!("cant gtr these two types");
                }
            }
        }
    }

    ///vm math
    fn stack_add(&mut self) {
        if let (Some(right), Some(left)) = (self.execution_stack.pop(), self.execution_stack.pop())
        {
            match (left.value, right.value) {
                (Value::Integer(left), Value::Integer(right)) => self.execution_stack.push(Token {
                    value: Value::Integer(left + right),
                    id: None,
                }),
                (Value::Integer(ref left), Value::Float(right)) => {
                    let left = *left as f64;
                    self.execution_stack.push(Token {
                        value: Value::Float(left + right),

                        id: None,
                    })
                }
                (Value::Float(left), Value::Float(right)) => self.execution_stack.push(Token {
                    value: Value::Float(left + right),
                    id: None,
                }),
                (Value::Float(left), Value::Integer(ref right)) => {
                    let right = *right as f64;
                    self.execution_stack.push(Token {
                        value: Value::Float(left + right),

                        id: None,
                    })
                }
                (Value::String(left), Value::String(right)) => self.execution_stack.push(Token {
                    value: Value::String(left + &right),
                    id: None,
                }),
                _ => {
                    println!("cant add these two types");
                }
            }
        }
    }

    fn pow(&mut self) {
        if let (Some(right), Some(left)) = (self.execution_stack.pop(), self.execution_stack.pop())
        {
            if let (Value::Integer(left), Value::Integer(right)) = (left.value, right.value) {
                self.execution_stack.push(Token {
                    value: Value::Integer(left.pow(right as u32)),
                    id: None,
                })
            }
        }
    }

    fn neg(&mut self) {
        if let Some(left) = self.execution_stack.pop() {
            match left.value {
                Value::Integer(left) => self.execution_stack.push(Token {
                    value: Value::Integer(- left),
                    id: None,
                }),
                Value::Float(left) => self.execution_stack.push(Token {
                    value: Value::Float(- left),
                    id: None,
                }),
                _ => {
                    println!("cant sqrt this")
                }
            }
        }
    }

    fn sqrt(&mut self) {
        if let Some(left) = self.execution_stack.pop() {
            match left.value {
                Value::Integer(left) => self.execution_stack.push(Token {
                    value: Value::Float((left as f64).sqrt()),
                    id: None,
                }),
                Value::Float(left) => self.execution_stack.push(Token {
                    value: Value::Float(left.sqrt()),
                    id: None,
                }),
                _ => {
                    println!("cant sqrt this")
                }
            }
        }
    }

    fn stack_sub(&mut self) {
        if let (Some(right), Some(left)) = (self.execution_stack.pop(), self.execution_stack.pop())
        {
            if let (Value::Integer(left), Value::Integer(right)) = (left.value, right.value) {
                self.execution_stack.push(Token {
                    value: Value::Integer(left - right),
                    id: None,
                })
            }
        }
    }

    fn stack_modulo(&mut self) {
        if let (Some(right), Some(left)) = (self.execution_stack.pop(), self.execution_stack.pop())
        {
            match (left.value, right.value) {
                (Value::Integer(left), Value::Integer(right)) => {
                    self.execution_stack.push(Token {
                        value: Value::Integer(left.modulo(right)),
                        id: None,
                    });
                }
                _ => {
                    println!("Cant print this type")
                }
            }
        }
    }

    fn stack_mul(&mut self) {
        if let (Some(right), Some(left)) = (self.execution_stack.pop(), self.execution_stack.pop())
        {
            if let (Value::Integer(left), Value::Integer(right)) = (left.value, right.value) {
                self.execution_stack.push(Token {
                    value: Value::Integer(left * right),
                    id: None,
                })
            }
        }
    }

    fn readln(&mut self) {

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let line = trim_newline(&mut line);

        if is_string_number(&line) {
            // Float
            if line.contains('.') {
                if let Ok(v) = line.parse() {
                    self.execution_stack.push(Token {
                        value: Value::Float(v),
                        id: None,
                    });
                }
            } else {
                // Int
                if let Ok(v) = line.parse() {
                    self.execution_stack.push(Token {
                        value: Value::Integer(v),
                        id: None,
                    });
                }
            }
        } else {
            self.execution_stack.push(Token {
                value: Value::String(line),
                id: None,
            });
        }
    }

    fn stack_print(&mut self) {
        if let Some(token) = self.execution_stack.pop() {
            match token.value {
                Value::Identifier(value) => {
                    print!("{}", &value)
                }
                Value::Function(value) => {
                    print!("{:?}", &value)
                }
                Value::Integer(value) => {
                    print!("{}", &value)
                }
                Value::Float(value) => {
                    print!("{}", &value)
                }
                Value::String(value) => {
                    print!("{}", &value)
                }
                Value::Symbol(value) => {
                    print!("{}", &value)
                }
                Value::Bool(value) => {
                    print!("{}", &value)
                }
                Value::Char(value) => {
                    print!("{}", value)
                }
                _ => {
                    print!("Cant print this type")
                }
            }
        }
    }

    fn user_macro_call(&mut self) {
        if let Some(function) = self.execution_stack.pop() {
            match function.value {
                Value::Block(Some(ref block)) => {
                    for t in block.iter() {
                        self.execute_token(t)
                    }
                }
                _ => {
                    println!("Cant macro this type")
                }
            }
        }
    }

    fn user_function_call(&mut self) {
        if let Some(function) = self.execution_stack.pop() {
            match function.value {
                Value::Block(Some(ref block)) => {
                    self.call_stack.push(HashMap::new());

                    for t in block.iter() {
                        self.execute_token(t)
                    }

                    self.call_stack.pop();
                }
                _ => {
                    println!("Cant call this type")
                }
            }
        }
    }

    fn stack_println(&mut self) {
        if let Some(token) = self.execution_stack.pop() {
            match token.value {
                Value::Identifier(value) => {
                    println!("{}", &value)
                }
                Value::Function(value) => {
                    println!("{:?}", &value)
                }
                Value::Integer(value) => {
                    println!("{}", &value)
                }
                Value::Float(value) => {
                    println!("{}", &value)
                }
                Value::String(value) => {
                    println!("{}", &value)
                }
                Value::Symbol(value) => {
                    println!("{}", &value)
                }
                Value::Bool(value) => {
                    println!("{}", &value)
                }
                Value::Char(value) => {
                    println!("{}", value)
                }
                Value::Block(_) => {
                    println!("BLOCK")
                }
                _ => {
                    println!("Cant print this type")
                }
            }
        }
    }

    fn flush(&self) {
        std::io::stdout().flush().unwrap();
    }

    fn access_call(&mut self) {
        if let (Some(ident), Some(block)) = (self.execution_stack.pop(), self.execution_stack.pop())
        {
            match (ident.value, block.value) {
                (Value::Identifier(identifier), Value::List(Some(block))) => {
                    match identifier.as_str() {
                        "len" => self.execution_stack.push(Token {
                            id: None,
                            value: Value::Integer(block.len() as i128),
                        }),
                        _ => {
                            println!("cant len this")
                        }
                    }
                }
                (Value::Integer(index), Value::List(Some(block))) => {
                    if let Some(item) = block.get(index as usize) {
                        self.execution_stack.push(item.clone());
                    }
                }
                (Value::Identifier(identifier), Value::Block(Some(block))) => {
                    match identifier.as_str() {
                        "len" => self.execution_stack.push(Token {
                            id: None,
                            value: Value::Integer(block.len() as i128),
                        }),
                        _ => {
                            self.call_stack.push(HashMap::new());

                            for t in block.iter() {
                                self.execute_token(t)
                            }

                            if let Some(scope) = self.call_stack.last_mut() {
                                if let Some(token) = scope.get(&identifier) {
                                    self.execution_stack.push(token.clone())
                                }
                            }

                            self.call_stack.pop();
                        }
                    }
                }
                _ => {
                    println!("cant access_call these")
                }
            }
        }
    }
}

// //Match values for each token
// match i.value.as_str() {
//     // If left paren is found then one must be missing the other pair
//     "(" => print_error(
//         "Possibly missing ')' pair",
//         i.line_number,
//         i.row,
//         &self.file,
//         &self.last_instruction,
//     ),

//     "exit" => {
//         self.exit_loop = true;
//     }

//     // "store_import" => {
//     //     if let Some(a) = self.execution_stack.pop() {
//     //         // Get filename from argument
//     //         let mut lexer = lexer::Lexer::new_from_file(&a.value);

//     //         // Parse the file into tokens
//     //         lexer.parse();

//     //         self.execution_stack.push(Token {
//     //             id: None,
//     //             token_type: TokenTypes::Block,
//     //             value: "Block".to_string(),
//     //             block: lexer.block_stack[0].clone(),
//     //             line_number: 0,
//     //             row: 0,
//     //         })
//     //     }
//     // }
//     "import" => {
//         if let Some(a) = self.execution_stack.pop() {
//             let mut lexer = lexer::Lexer::new_from_file(&a.value);
//             // Parse the file into tokens
//             lexer.parse();

//             // Create new vm
//             let mut parser = Parser::new();
//             if self.debug {
//                 parser.debug = true;
//             }

//             // Shunt tokens in vm
//             let shunted = parser.shunt(&lexer.block_stack[0]);
//             let mut vm = ManitcoreVm::new(&shunted, &a.value);
//             if self.debug {
//                 vm.debug = true;
//             }

//             // Copy the stack and the call_stack inside the vm
//             vm.execution_stack = self.execution_stack.clone();
//             vm.call_stack = self.call_stack.clone();

//             // Run the vm
//             vm.execute();

//             self.call_stack = vm.call_stack.clone();
//             self.execution_stack = vm.execution_stack.clone();
//         } else {
//             print_error(
//                 format!("not enough arguments for {}", i.value).as_str(),
//                 i.line_number,
//                 i.row,
//                 &self.file,
//                 &self.last_instruction,
//             )
//         }
//     }
//     // "store_url" => {
//     //     if let Some(a) = self.execution_stack.pop() {
//     //         if let Ok(body) = ureq::get(&a.value).call() {
//     //             if let Ok(body) = body.into_string() {
//     //                 // Get filename from argument
//     //                 let mut lexer = lexer::Lexer::new_from_string(&body);

//     //                 // Parse the file into tokens
//     //                 lexer.parse();

//     //                 self.execution_stack.push(Token {
//     //                     id: None,
//     //                     token_type: TokenTypes::Block,
//     //                     value: "Block".to_string(),
//     //                     block: lexer.block_stack[0].clone(),
//     //                     line_number: 0,
//     //                     row: 0,
//     //                 })
//     //             }
//     //         }
//     //     }
//     // }
//     "import_url" => {
//         if let Some(a) = self.execution_stack.pop() {
//             if let Ok(body) = ureq::get(&a.value).call() {
//                 if let Ok(body) = body.into_string() {
//                     // Get filename from argument
//                     let mut lexer = lexer::Lexer::new_from_string(&body);

//                     // Parse the file into tokens
//                     lexer.parse();
//                     let mut parser = Parser::new();
//                     if self.debug {
//                         parser.debug = true;
//                     }

//                     // Store now parsed tokens into a new list
//                     let shunted = parser.shunt(&lexer.block_stack[0]);
//                     let mut vm = ManitcoreVm::new(&shunted, &body);
//                     if self.debug {
//                         vm.debug = true;
//                     }

//                     // Copy the stack and the call_stack inside the vm
//                     vm.execution_stack = self.execution_stack.clone();
//                     vm.call_stack = self.call_stack.clone();

//                     // Run the vm
//                     vm.execute();

//                     self.call_stack = vm.call_stack.clone();
//                     self.execution_stack = vm.execution_stack.clone();
//                 }
//             }
//         }
//     }
//     "run_url" => {
//         if let Some(a) = self.execution_stack.pop() {
//             if let Ok(body) = ureq::get(&a.value).call() {
//                 if let Ok(body) = body.into_string() {
//                     // Get filename from argument
//                     let mut lexer = lexer::Lexer::new_from_string(&body);

//                     // Parse the file into tokens
//                     lexer.parse();
//                     let mut parser = Parser::new();
//                     if self.debug {
//                         parser.debug = true;
//                     }

//                     // Store now parsed tokens into a new list
//                     let shunted = parser.shunt(&lexer.block_stack[0]);
//                     let mut vm = ManitcoreVm::new(&shunted, &body);
//                     if self.debug {
//                         vm.debug = true;
//                     }

//                     // Execute the vm using parsed token list
//                     vm.execute();
//                 }
//             }
//         }
//     }

//     // "randomf" => {
//     //     if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
//     //     {
//     //         let mut f: f32 = 0.0;
//     //         let mut s: f32 = 0.0;

//     //         if let Ok(v) = a.value.parse() {
//     //             f = v
//     //         } else {
//     //             print_error(
//     //                 "expected a number",
//     //                 i.line_number,
//     //                 i.row,
//     //                 &self.file,
//     //                 &self.last_instruction,
//     //             )
//     //         }

//     //         if let Ok(v) = b.value.parse() {
//     //             s = v
//     //         } else {
//     //             print_error(
//     //                 "expected a number",
//     //                 i.line_number,
//     //                 i.row,
//     //                 &self.file,
//     //                 &self.last_instruction,
//     //             )
//     //         }

//     //         if s <= f {
//     //             let mut rng = rand::thread_rng();

//     //             self.execution_stack.push(Token {
//     //                 token_type: TokenTypes::Integer,
//     //                 value: (rng.gen_range(s..=f)).to_string(),
//     //                 line_number: 0,
//     //                 row: 0,
//     //                 block: vec![],
//     //                 id: None,
//     //             })
//     //         } else {
//     //             let mut rng = rand::thread_rng();

//     //             self.execution_stack.push(Token {
//     //                 token_type: TokenTypes::Integer,
//     //                 value: (rng.gen_range(f..=s)).to_string(),
//     //                 line_number: 0,
//     //                 row: 0,
//     //                 block: vec![],
//     //                 id: None,
//     //             })
//     //         }
//     //     } else {
//     //         print_error(
//     //             format!("not enough arguments for {}", i.value).as_str(),
//     //             i.line_number,
//     //             i.row,
//     //             &self.file,
//     //             &self.last_instruction,
//     //         )
//     //     };
//     // }
//     "random_int" => {
//         if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
//         {
//             let mut f: usize = 0;
//             let mut s: usize = 0;

//             if let Ok(v) = a.value.parse() {
//                 f = v
//             } else {
//                 print_error(
//                     "expected a number",
//                     i.line_number,
//                     i.row,
//                     &self.file,
//                     &self.last_instruction,
//                 )
//             }

//             if let Ok(v) = b.value.parse() {
//                 s = v
//             } else {
//                 print_error(
//                     "expected a number",
//                     i.line_number,
//                     i.row,
//                     &self.file,
//                     &self.last_instruction,
//                 )
//             }

//             if s <= f {
//                 let mut rng = rand::thread_rng();

//                 self.execution_stack.push(Token {
//                     token_type: TokenTypes::Integer,
//                     value: (rng.gen_range(s..=f)).to_string(),
//                     line_number: 0,
//                     row: 0,
//
//                     id: None,
//                 })
//             } else {
//                 let mut rng = rand::thread_rng();

//                 self.execution_stack.push(Token {
//                     token_type: TokenTypes::Integer,
//                     value: (rng.gen_range(f..=s)).to_string(),
//                     line_number: 0,
//                     row: 0,
//
//                     id: None,
//                 })
//             }
//         } else {
//             print_error(
//                 format!("not enough arguments for {}", i.value).as_str(),
//                 i.line_number,
//                 i.row,
//                 &self.file,
//                 &self.last_instruction,
//             )
//         };
//     }
//     // "insert" => {
//     //     if let (Some(index), Some(item), Some(mut list)) = (
//     //         self.execution_stack.pop(),
//     //         self.execution_stack.pop(),
//     //         self.execution_stack.pop(),
//     //     ) {
//     //         let mut i: usize = 0;
//     //         if let Ok(v) = index.value.parse() {
//     //             i = v
//     //         } else {
//     //             print_error(
//     //                 "expected a number",
//     //                 index.line_number,
//     //                 index.row,
//     //                 &self.file,
//     //                 &self.last_instruction,
//     //             )
//     //         }
//     //         if i > list.block.len() {
//     //             list.block.push(item);
//     //         } else {
//     //             list.block.insert(i, item);
//     //         }
//     //         self.execution_stack.push(list)
//     //     } else {
//     //         print_error(
//     //             format!("not enough arguments for {}", i.value).as_str(),
//     //             i.line_number,
//     //             i.row,
//     //             &self.file,
//     //             &self.last_instruction,
//     //         )
//     //     }
//     // }
//     // "remove" => {
//     //     if let (Some(a), Some(mut b)) =
//     //         (self.execution_stack.pop(), self.execution_stack.pop())
//     //     {
//     //         let mut i: usize = 0;
//     //         if let Ok(v) = a.value.parse() {
//     //             i = v
//     //         } else {
//     //             print_error(
//     //                 "expected a number",
//     //                 a.line_number,
//     //                 a.row,
//     //                 &self.file,
//     //                 &self.last_instruction,
//     //             )
//     //         }
//     //         if i > b.block.len() {
//     //             b.block.pop();
//     //         } else {
//     //             b.block.remove(i);
//     //         }

//     //         self.execution_stack.push(b)
//     //     } else {
//     //         print_error(
//     //             format!("not enough arguments for {}", i.value).as_str(),
//     //             i.line_number,
//     //             i.row,
//     //             &self.file,
//     //             &self.last_instruction,
//     //         )
//     //     }
//     // }
//     // "append" => {
//     //     if let (Some(mut a), Some(mut b)) =
//     //         (self.execution_stack.pop(), self.execution_stack.pop())
//     //     {
//     //         b.block.append(&mut a.block);
//     //         self.execution_stack.push(b)
//     //     } else {
//     //         print_error(
//     //             format!("not enough arguments for {}", i.value).as_str(),
//     //             i.line_number,
//     //             i.row,
//     //             &self.file,
//     //             &self.last_instruction,
//     //         )
//     //     }
//     // }
//     // "push" => {
//     //     if let (Some(a), Some(mut b)) =
//     //         (self.execution_stack.pop(), self.execution_stack.pop())
//     //     {
//     //         b.block.push(a);
//     //         self.execution_stack.push(b)
//     //     } else {
//     //         print_error(
//     //             format!("not enough arguments for {}", i.value).as_str(),
//     //             i.line_number,
//     //             i.row,
//     //             &self.file,
//     //             &self.last_instruction,
//     //         )
//     //     }
//     // }
//     // "pop" => {
//     //     if let (Some(a), Some(mut b)) =
//     //         (self.execution_stack.pop(), self.execution_stack.pop())
//     //     {
//     //         if let Some(mut item) = b.block.pop() {
//     //             if item.token_type != TokenTypes::Nothing {
//     //                 if let Some(p) = a.id {
//     //                     item.id = Some(p.clone());
//     //                     self.call_stack.insert(p, item);
//     //                 } else {
//     //                     item.id = Some(a.value.clone());
//     //                     self.call_stack.insert(a.value.clone(), item);
//     //                 }
//     //             }
//     //         } else {
//     //             print_error(
//     //                 format!("Could not pop list {}, Not enough items", b.value).as_str(),
//     //                 i.line_number,
//     //                 i.row,
//     //                 &self.file,
//     //                 &self.last_instruction,
//     //             )
//     //         }
//     //         self.execution_stack.push(b)
//     //     } else {
//     //         print_error(
//     //             format!("not enough arguments for {}", i.value).as_str(),
//     //             i.line_number,
//     //             i.row,
//     //             &self.file,
//     //             &self.last_instruction,
//     //         )
//     //     }
//     // }

//     "loop" => {
//         if let Some(block) = self.execution_stack.pop() {
//             loop {
//                 if let Some(ref bl) = block.block {
//                     for t in bl {
//                         self.execute_token(t)
//                     }
//                     if self.exit_loop {
//                         break;
//                     }
//                 }
//             }
//             self.exit_loop = false;
//         }
//     }

//     // "from" => {
//     //     if let (Some(block), Some(end), Some(start)) = (
//     //         self.execution_stack.pop(),
//     //         self.execution_stack.pop(),
//     //         self.execution_stack.pop(),
//     //     ) {
//     //         let mut s: usize = 0;
//     //         if let Ok(v) = start.value.parse() {
//     //             s = v
//     //         } else {
//     //             print_error(
//     //                 &format!("expected a number but found {}", start.value),
//     //                 start.line_number,
//     //                 start.row,
//     //                 &self.file,
//     //                 &self.last_instruction,
//     //             )
//     //         }

//     //         let mut e: usize = 0;
//     //         if let Ok(v) = end.value.parse() {
//     //             e = v
//     //         } else {
//     //             print_error(
//     //                 &format!("expected a number but found {}", end.value),
//     //                 end.line_number,
//     //                 end.row,
//     //                 &self.file,
//     //                 &self.last_instruction,
//     //             )
//     //         }

//     //         for _t in s..=e {
//     //             for t in &block.block {
//     //                 self.execute_token(t)
//     //             }
//     //             if self.exit_loop {
//     //                 break;
//     //             }
//     //         }
//     //     }
//     // }

//     // "shc" => self.call_stack.clear(),
//     // "exist" => {
//     //     if let Some(t) = self.execution_stack.pop() {
//     //         if t.token_type == TokenTypes::Nothing {
//     //             self.execution_stack.push(Token {
//     //                 id: None,
//     //                 token_type: TokenTypes::Bool,
//     //                 value: "false".to_string(),
//     //                 block: vec![],
//     //                 line_number: 0,
//     //                 row: 0,
//     //             })
//     //         } else {
//     //             self.execution_stack.push(Token {
//     //                 id: None,
//     //                 token_type: TokenTypes::Bool,
//     //                 value: "true".to_string(),
//     //                 block: vec![],
//     //                 line_number: 0,
//     //                 row: 0,
//     //             })
//     //         }
//     //     }
//     // }
//     // "rm" => if let Some(_t) = self.execution_stack.pop() {},
//     // // Used to tie more than 1 token at a time from the stack

//     // "?" => {
//     //     if self.method_call {
//     //         self.exit_loop = true;
//     //     }
//     // }
//     // // This function will pop off a block and execute it using the outer scope call_stack and stack
//     // "call" => {
//     //     if let Some(a) = self.execution_stack.pop() {
//     //         // Create new vm
//     //         let mut parser = Parser::new();
//     //         if self.debug {
//     //             parser.debug = true;
//     //         }

//     //         // Shunt tokens in vm
//     //         let shunted = parser.shunt(&a.block).clone();
//     //         let mut vm = ManitcoreVm::new(&shunted, &a.value);
//     //         if self.debug {
//     //             vm.debug = true;
//     //         }

//     //         // Copy the stack and the call_stack inside the vm
//     //         vm.execution_stack = self.execution_stack.clone();
//     //         vm.call_stack = self.call_stack.clone();

//     //         // Run the vm
//     //         vm.execute();

//     //         self.call_stack = vm.call_stack.clone();
//     //         self.execution_stack = vm.execution_stack.clone();
//     //     } else {
//     //         print_error(
//     //             format!("not enough arguments for {}", i.value).as_str(),
//     //             i.line_number,
//     //             i.row,
//     //             &self.file,
//     //             &self.last_instruction,
//     //         )
//     //     }
//     // }

//     "ret" => {
//         if let (Some(a), Some(b), Some(c)) = (
//             self.execution_stack.pop(),
//             self.execution_stack.pop(),
//             self.execution_stack.pop(),
//         ) {
//             if c.value == b.value {
//                 self.execution_stack.push(a)
//             }
//         } else {
//             print_error(
//                 format!("not enough arguments for {}", i.value).as_str(),
//                 i.line_number,
//                 i.row,
//                 &self.file,
//                 &self.last_instruction,
//             )
//         };
//     }

//     "sec" => {
//         self.execution_stack.clear();
//     }
//     "rev" => {
//         self.execution_stack.reverse();
//     }

//     // "/" => {
//     //     if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
//     //     {
//     //         let mut f: f32 = 0.0;
//     //         let mut s: f32 = 0.0;

//     //         if let Ok(v) = a.value.parse() {
//     //             f = v
//     //         } else {
//     //             print_error(
//     //                 "expected a number",
//     //                 i.line_number,
//     //                 i.row,
//     //                 &self.file,
//     //                 &self.last_instruction,
//     //             )
//     //         }

//     //         if let Ok(v) = b.value.parse() {
//     //             s = v
//     //         } else {
//     //             print_error(
//     //                 "expected a number",
//     //                 i.line_number,
//     //                 i.row,
//     //                 &self.file,
//     //                 &self.last_instruction,
//     //             )
//     //         }

//     //         self.execution_stack.push(Token {
//     //             token_type: TokenTypes::Number,
//     //             value: (s / f).to_string(),
//     //             line_number: 0,
//     //             row: 0,
//     //             block: vec![],
//     //             id: None,
//     //         })
//     //     } else {
//     //         print_error(
//     //             format!("not enough arguments for {}", i.value).as_str(),
//     //             i.line_number,
//     //             i.row,
//     //             &self.file,
//     //             &self.last_instruction,
//     //         )
//     //     };
//     // }


//     "println" => {
//         let mut escape_char = false;
//         if let Some(a) = self.execution_stack.pop() {
//             for c in a.value.chars() {
//                 if !escape_char && c == '\\' {
//                     escape_char = true;
//                     continue;
//                 }
//                 if escape_char {
//                     if c == 'n' {
//                         println!();
//                         escape_char = false;
//                         continue;
//                     }
//                     if c == 't' {
//                         print!("\t");
//                         escape_char = false;
//                         continue;
//                     }
//                 }
//                 print!("{}", c)
//             }
//             println!();
//         } else {
//             println!();
//         };
//     }
//     "print" => {
//         let mut escape_char = false;
//         if let Some(a) = self.execution_stack.pop() {
//             for c in a.value.chars() {
//                 if !escape_char && c == '\\' {
//                     escape_char = true;
//                     continue;
//                 }
//                 if escape_char {
//                     if c == 'n' {
//                         println!();
//                         escape_char = false;
//                         continue;
//                     }
//                     if c == 't' {
//                         print!("\t");
//                         escape_char = false;
//                         continue;
//                     }
//                 }
//                 print!("{}", c)
//             }
//         } else {
//             println!()
//         };
//     }

//     "newline" => {}
//     _ => {}
// }

// if i.value != "@" {
//     self.last_instruction = i.value.to_owned();

