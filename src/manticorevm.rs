use core::time;
use getch::Getch;
use hashbrown::HashMap;
use modulo::Mod;
use rand::Rng;
use std::process::Command;
use std::rc::Rc;
use std::thread;
use std::{io::Write, vec};

use crate::token::BlockType;
use crate::{
    string_utils::{is_string_number, trim_newline},
    token::{Functions, Token, Value},
};

pub struct ManitcoreVm {
    vm_instruction_: Vec<Token>,
    pub execution_stack: Vec<Token>,
    pub exit_loop: bool,
    pub continue_loop: bool,
    pub debug: bool,
    pub block_call: bool,
    pub core_self: Vec<Token>,
    call_stack: Vec<HashMap<String, Token>>,
}

impl ManitcoreVm {
    pub fn new(tokenlist: &[Token]) -> Self {
        Self {
            vm_instruction_: tokenlist.to_vec(),
            execution_stack: Vec::with_capacity(1024),
            debug: false,
            exit_loop: false,
            block_call: false,
            core_self: vec![],
            call_stack: vec![HashMap::new()],
            continue_loop: false,
        }
    }
    pub fn get_tokenifvar(&mut self) -> Option<Token> {
        if let Some(tok) = self.execution_stack.pop() {
            if let Value::Identifier(ident) = tok.value {
                if let Some(scope) = self.call_stack.last_mut() {
                    if let Some(token) = scope.get(&ident) {
                        Some(token.clone())
                    } else {
                        println!("unknown identifier {}", ident);
                        None
                    }
                } else {
                    None
                }
            } else {
                Some(tok)
            }
        } else {
            None
        }
    }

    pub fn get_tokenifident(&mut self, ident: &str) -> Option<Token> {
        if let Some(scope) = self.call_stack.last_mut() {
            if let Some(token) = scope.get(ident) {
                Some(token.clone())
            } else {
                println!("unknown identifier {}", ident);
                None
            }
        } else {
            None
        }
    }

    // Proccess each token
    pub fn execute(&mut self) {
        for i in self.vm_instruction_.clone() {
            self.execute_token(&i);
        }

        // if self.debug {
        //     if let Some(scope) = self.call_stack.last() {
        //         for (k, v) in scope {
        //             println!("{} -> ({})", k, v.get_value_as_string())
        //         }

        //         for tok in &self.execution_stack {
        //             print!("[{}] ", tok.get_value_as_string())
        //         }
        //         println!()
        //     }
        // }
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
                '/' => self.stack_div(),
                '-' => self.stack_sub(),
                '+' => self.stack_add(),
                _ => {}
            },
            Value::Bool(_) => {
                self.execution_stack.push(token.clone());
            }
            Value::Block(block) => {
                match block {
                    BlockType::Lambda(block) => {
                        // Call with new scope
                        self.call_stack.push(HashMap::new());
                        for t in block.iter() {
                            self.execute_token(t)
                        }
                        if let Some(token) = self.get_tokenifvar() {
                            self.execution_stack.push(token)
                        }
                        self.call_stack.pop();
                    }
                    _ => {
                        self.execution_stack.push(token.clone());
                    }
                }
            }
            Value::List(_) => {
                self.execution_stack.push(token.clone());
            }

            Value::Identifier(_) => {
                self.execution_stack.push(token.clone());
            }
            Value::Function(function) => match function {
                Functions::VariableAssign => self.variable_assign(),
                Functions::Equals => self.equ_comparison(),
                Functions::Println => self.stack_println(),
                Functions::Range => self.create_range(),
                Functions::Mod => self.stack_modulo(),
                Functions::For => self.for_loop(),
                Functions::If => self.if_statement(),
                Functions::Print => self.stack_print(),
                Functions::Break => self.break_loop(),
                Functions::Flush => self.flush(),
                Functions::And => self.logical_and(),
                Functions::Or => self.logical_or(),
                Functions::Not => self.logical_not(),
                Functions::Gtr => self.gtr_comparison(),
                Functions::Lss => self.lss_comparison(),
                Functions::SelfId => self.get_self(),
                Functions::AccessCall => self.access_call(),
                Functions::Readln => self.readln(),
                Functions::Neg => self.neg(),
                Functions::Pow => self.pow(),
                Functions::Sqrt => self.sqrt(),
                Functions::Match => todo!(),
                Functions::Let => self.closure_let(),
                Functions::Ret => todo!(),
                Functions::Random => self.random(),
                Functions::Command => self.command(),
                Functions::Exit => self.exit(),
                Functions::Recursive => todo!(),
                Functions::Dup => self.dup(),
                Functions::Include => self.include(),
                Functions::Add => self.stack_add(),
                Functions::Sub => self.stack_sub(),
                Functions::Mul => self.stack_mul(),
                Functions::Div => self.stack_div(),
                Functions::UserFunctionCall => self.block_call(),
                Functions::UserFunctionChain => self.user_block_chain_call(),
                Functions::FunctionVariableAssign => self.function_variable_assign(),
                Functions::Continue => self.continue_loop(),
                Functions::Push => self.list_push(),
                Functions::Pop => self.list_pop(),
                Functions::Insert => self.list_insert(),
                Functions::Remove => self.list_remove(),
                Functions::Append => todo!(),
                Functions::PopStack => self.pop_stack(),
                Functions::Clear => self.clearscreen(),
                Functions::Getch => self.getch(),
                Functions::Sleep => self.sleep(),
                Functions::Proc => self.create_proc(),
                Functions::Return => self.return_top(),

            },
            Value::Nothing => {
                todo!()
            }
            Value::Char(_) => self.execution_stack.push(token.clone()),
            Value::UserBlockCall(function_name) => self.user_block_call(function_name),
        }
    }

    fn return_top(&mut self) {
        if let Some(token) = self.get_tokenifvar() {
            self.execution_stack.push(token)
        }
    }

    // todo change to match statement for each block type

    fn closure_let(&mut self) {
        if let Some(block) = self.get_tokenifvar() {
            if let Some(scope) = self.call_stack.last_mut() {
                let mut core_self = vec![];

                for (ident, token) in scope {
                    core_self.push(Token {
                        value: Value::Identifier(ident.clone()),
                    });
                    core_self.push(Token {
                        value: token.value.clone(),
                    });
                    core_self.push(Token {
                        value: Value::Function(Functions::VariableAssign),
                    })
                }
                if let Value::Block(BlockType::Literal(ref block)) = block.value {
                    for t in block.iter() {
                        core_self.push(t.clone())
                    }
                    self.execution_stack.push(Token {
                        value: Value::Block(BlockType::Literal(Rc::new(core_self))),
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
                    value: Value::Identifier(ident.clone()),
                });
                core_self.push(Token {
                    value: token.value.clone(),
                });
                core_self.push(Token {
                    value: Value::Function(Functions::VariableAssign),
                })
            }
            self.execution_stack.push(Token {
                value: Value::Block(BlockType::Literal(Rc::new(core_self))),
            })
        }
    }

    fn command(&mut self) {
        if let (Some(args), Some(command)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            match (args.value, command.value) {
                (Value::List(args), Value::String(command)) => {
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

    fn pop_stack(&mut self) {
        self.execution_stack.pop();
    }

    fn macro_variable_assign(&mut self) {
        let mut variable_stack: Vec<String> = Vec::with_capacity(10);
        if let Some(token) = self.get_tokenifvar() {
            if let Value::List(identifiers) = token.value {
                for toks in identifiers.iter().rev() {
                    if let Value::Identifier(ident) = &toks.value {
                        variable_stack.push(ident.clone())
                    }
                }
            }
        }

        // Tie each value into the call_stack using the tokens poped

        if let Some(newscope) = self.call_stack.last_mut() {
            for values in variable_stack {
                if let Some(tok) = self.execution_stack.pop() {
                    if values != "_" {
                        newscope.insert(values, tok.clone());
                    }
                }
            }
        }
    }

    fn function_variable_assign(&mut self) {
        let mut variable_stack: Vec<String> = Vec::with_capacity(10);
        if let Some(token) = self.get_tokenifvar() {
            if let Value::List(identifiers) = token.value {
                for toks in identifiers.iter().rev() {
                    if let Value::Identifier(ident) = &toks.value {
                        variable_stack.push(ident.clone())
                    }
                }
            }
        }

        // Tie each value into the call_stack using the tokens poped

        if let Some(mut newscope) = self.call_stack.pop() {
            for values in variable_stack {
                if let Some(tok) = self.get_tokenifvar() {
                    newscope.insert(values, tok.clone());
                }
            }
            self.call_stack.push(newscope)
        }
    }

    fn break_loop(&mut self) {
        self.exit_loop = true;
    }

    fn continue_loop(&mut self) {
        self.continue_loop = true;
    }

    fn logical_not(&mut self) {
        if let Some(token) = self.get_tokenifvar() {
            if let Value::Bool(bool) = token.value {
                self.execution_stack.push(Token {
                    value: Value::Bool(!bool),
                })
            }
        }
    }

    // fn capture(&mut self) {
    //     // TODO
    // }

    fn include(&mut self) {
        if let (Some(tblock), Some(list)) = (self.execution_stack.pop(), self.execution_stack.pop())
        {
            match (tblock.value, &list.value) {
                (Value::Block(BlockType::Literal(ref block)), Value::List(list)) => {
                    let mut capture = vec![];
                    capture.clone_from(&*block);

                    for t in list.iter() {
                        if let Value::Identifier(ident) = &t.value {
                            capture.insert(
                                0,
                                Token {
                                    value: Value::Function(Functions::VariableAssign),
                                },
                            );
                            if let Some(scope) = self.call_stack.last_mut() {
                                if let Some(item) = scope.get(ident) {
                                    capture.insert(0, item.clone());
                                }
                            }
                            capture.insert(0, t.clone());
                        }
                    }

                    self.execution_stack.push(Token {
                        value: Value::Block(BlockType::Literal(Rc::new(capture))),
                    })
                }
                (Value::Block(BlockType::Literal(ref block)), Value::Identifier(ident)) => {
                    let mut capture = vec![];
                    capture.clone_from(&*block);

                    capture.insert(
                        0,
                        Token {
                            value: Value::Function(Functions::VariableAssign),
                        },
                    );
                    if let Some(scope) = self.call_stack.last_mut() {
                        if let Some(item) = scope.get(ident) {
                            capture.insert(0, item.clone());
                        }
                    }
                    capture.insert(0, list.clone());

                    self.execution_stack.push(Token {
                        value: Value::Block(BlockType::Literal(Rc::new(capture))),
                    })
                }
                _ => {
                    println!("cant include these values")
                }
            }
        }
    }

    fn recursive(&mut self) {
        if let (Some(tblock), Some(id)) = (self.execution_stack.pop(), self.execution_stack.pop()) {
            match (tblock.value.clone(), id.value) {
                (Value::Block(BlockType::Literal(ref block)), Value::Identifier(ident)) => {
                    let mut recurse = vec![
                        Token {
                            value: Value::Identifier(ident.clone()),
                        },
                        Token {
                            value: Value::Identifier(ident),
                        },
                        tblock,
                        Token {
                            value: Value::Function(Functions::Recursive),
                        },
                        Token {
                            value: Value::Function(Functions::VariableAssign),
                        },
                    ];
                    for t in block.iter() {
                        recurse.push(t.clone())
                    }
                    self.execution_stack.push(Token {
                        value: Value::Block(BlockType::Literal(Rc::new(recurse))),
                    })
                }
                _ => {
                    println!("cant recursion these values")
                }
            }
        }
    }

    fn logical_and(&mut self) {
        if let (Some(right), Some(left)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            match (right.value, left.value) {
                (Value::Bool(right), Value::Bool(left)) => self.execution_stack.push(Token {
                    value: Value::Bool(left && right),
                }),
                _ => {
                    println!("cant && these values")
                }
            }
        }
    }

    fn logical_or(&mut self) {
        if let (Some(right), Some(left)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            match (right.value, left.value) {
                (Value::Bool(right), Value::Bool(left)) => self.execution_stack.push(Token {
                    value: Value::Bool(left || right),
                }),
                _ => {
                    println!("cant || these values")
                }
            }
        }
    }

    fn equ_comparison(&mut self) {
        if let (Some(right), Some(left)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            self.execution_stack.push(Token {
                value: Value::Bool(left.value == right.value),
            })
        }
    }

    fn variable_assign(&mut self) {
        if let (Some(value), Some(ident)) = (self.execution_stack.pop(), self.execution_stack.pop())
        {
            if let Some(scope) = self.call_stack.last_mut() {
                if let Value::Identifier(identifier) = ident.value {
                    if identifier != "_" {
                        scope.insert(identifier, value);
                    }
                }
            }
        }
    }

    fn if_statement(&mut self) {
        if let (Some(block), Some(boolmaybe)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
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
                    if let Value::Block(BlockType::Literal(block)) = block.value {
                        for t in &*block {
                            self.execute_token(t)
                        }
                    }
                }
            } else if let Some(bool2) = self.get_tokenifvar() {
                if let Value::Bool(bool) = bool2.value {
                    if bool {
                        if let Value::Block(BlockType::Literal(block)) = boolmaybe.value {
                            for t in &*block {
                                self.execute_token(t)
                            }
                        }
                    } else if let Value::Block(BlockType::Literal(block)) = block.value {
                        for t in &*block {
                            self.execute_token(t)
                        }
                    }
                }
            }
        }
    }

    fn create_range(&mut self) {
        if let (Some(end), Some(start)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            match (start.value.clone(), end.value.clone()) {
                (Value::Integer(start), Value::Integer(end)) => {
                    let mut new_list: Vec<Token> = Vec::new();
                    for x in start..=end {
                        new_list.push(Token {
                            value: Value::Integer(x),
                        });
                    }
                    self.execution_stack.push(Token {
                        value: Value::List(Rc::new(new_list.to_vec())),
                    })
                }
                _ => {
                    println!("cant make a range from these types{} {}",start.get_type_as_string(),end.get_type_as_string());
                }
            }
        }
    }

    fn for_loop(&mut self) {
        if let (Some(block), Some(list), Some(variable)) = (
            self.get_tokenifvar(),
            self.get_tokenifvar(),
            self.execution_stack.pop(),
        ) {
            match (block.value.clone(), list.value.clone(), &variable.value) {
                (Value::Block(BlockType::Literal(ref block)), Value::Block(BlockType::Literal(ref list)), Value::Identifier(variable_name)) => {
                    'outer: for variable in list.iter() {
                        if variable_name != "_" {
                            if let Some(scope) = self.call_stack.last_mut() {
                                scope.insert(variable_name.to_string(), variable.clone());
                            }
                        }
                        for t in block.iter() {
                            self.execute_token(t);
                            if self.exit_loop {
                                break 'outer;
                            }
                            if self.continue_loop {
                                self.continue_loop = false;
                                continue 'outer;
                            }
                        }
                    }
                    if let Some(scope) = self.call_stack.last_mut() {
                        scope.remove(variable_name);
                    }
                    self.exit_loop = false;
                }
                (Value::Block(BlockType::Literal(ref block)), Value::List(ref list), Value::Identifier(variable_name)) => {
                    'outer1: for variable in list.iter() {
                        if variable_name != "_" {
                            if let Some(scope) = self.call_stack.last_mut() {
                                scope.insert(variable_name.to_string(), variable.clone());
                            }
                        }
                        for t in block.iter() {
                            self.execute_token(t);
                            if self.exit_loop {
                                break 'outer1;
                            }
                            if self.continue_loop {
                                self.continue_loop = false;
                                continue 'outer1;
                            }
                        }
                    }
                    if let Some(scope) = self.call_stack.last_mut() {
                        scope.remove(variable_name);
                    }
                    self.exit_loop = false;
                }
                (
                    Value::Block(BlockType::Literal(ref block)),
                    Value::String(string),
                    Value::Identifier(variable_name),
                ) => {
                    'outer2: for variable in string.chars() {
                        if variable_name != "_" {
                            if let Some(scope) = self.call_stack.last_mut() {
                                scope.insert(
                                    variable_name.to_string(),
                                    Token {
                                        value: Value::Char(variable),
                                    },
                                );
                            }
                        }
                        for t in block.iter() {
                            self.execute_token(t);
                            if self.exit_loop {
                                break 'outer2;
                            }
                            if self.continue_loop {
                                self.continue_loop = false;
                                continue 'outer2;
                            }
                        }
                    }
                    if let Some(scope) = self.call_stack.last_mut() {
                        scope.remove(variable_name);
                    }
                    self.exit_loop = false;
                }
                (
                    Value::Block(BlockType::Literal(ref block)),
                    Value::Bool(condition),
                    Value::Identifier(variable_name),
                ) => {
                    if condition {
                        'outerw: loop {
                            for t in block.iter() {
                                self.execute_token(t);
                                if self.exit_loop {
                                    break 'outerw;
                                }
                                if self.continue_loop {
                                    self.continue_loop = false;
                                    continue 'outerw;
                                }
                            }
                        }
                    }

                    if let Some(scope) = self.call_stack.last_mut() {
                        scope.remove(variable_name);
                    }
                    self.exit_loop = false;
                }
                _ => {
                    println!("cant make a iterate from these types {} {} {}", block.get_type_as_string(),list.get_value_as_string(),variable.get_value_as_string());
                }
            }
        }
    }

    fn lss_comparison(&mut self) {
        if let (Some(right), Some(left)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            match (left.value, right.value) {
                (Value::Integer(left), Value::Integer(right)) => self.execution_stack.push(Token {
                    value: Value::Bool(left < right),
                }),
                (Value::Integer(ref left), Value::Float(right)) => {
                    let left = *left as f64;
                    self.execution_stack.push(Token {
                        value: Value::Bool(left < right),
                    })
                }
                (Value::Float(left), Value::Float(right)) => self.execution_stack.push(Token {
                    value: Value::Bool(left < right),
                }),
                (Value::Float(left), Value::Integer(ref right)) => {
                    let right = *right as f64;
                    self.execution_stack.push(Token {
                        value: Value::Bool(left < right),
                    })
                }
                _ => {
                    println!("cant lss these two types");
                }
            }
        }
    }

    fn gtr_comparison(&mut self) {
        if let (Some(right), Some(left)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            match (left.value, right.value) {
                (Value::Integer(left), Value::Integer(right)) => self.execution_stack.push(Token {
                    value: Value::Bool(left > right),
                }),
                (Value::Integer(ref left), Value::Float(right)) => {
                    let left = *left as f64;
                    self.execution_stack.push(Token {
                        value: Value::Bool(left > right),
                    })
                }
                (Value::Float(left), Value::Float(right)) => self.execution_stack.push(Token {
                    value: Value::Bool(left > right),
                }),
                (Value::Float(left), Value::Integer(ref right)) => {
                    let right = *right as f64;
                    self.execution_stack.push(Token {
                        value: Value::Bool(left > right),
                    })
                }
                _ => {
                    println!("cant gtr these two types");
                }
            }
        }
    }

    ///vm math

    fn stack_div(&mut self) {
        if let (Some(right), Some(left)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            match (left.value, right.value) {
                (Value::Integer(left), Value::Integer(right)) => self.execution_stack.push(Token {
                    value: Value::Float(left as f64 / right as f64),
                }),
                (Value::Integer(ref left), Value::Float(right)) => {
                    let left = *left as f64;
                    self.execution_stack.push(Token {
                        value: Value::Float(left / right),
                    })
                }
                (Value::Float(left), Value::Float(right)) => self.execution_stack.push(Token {
                    value: Value::Float(left / right),
                }),
                (Value::Float(left), Value::Integer(ref right)) => {
                    let right = *right as f64;
                    self.execution_stack.push(Token {
                        value: Value::Float(left / right),
                    })
                }
                _ => {
                    println!("cant div these two types");
                }
            }
        }
    }

    fn stack_add(&mut self) {
        if let (Some(right), Some(left)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            match (left.value, right.value) {
                (Value::Integer(left), Value::Integer(right)) => self.execution_stack.push(Token {
                    value: Value::Integer(left + right),
                }),
                (Value::Integer(ref left), Value::Float(right)) => {
                    let left = *left as f64;
                    self.execution_stack.push(Token {
                        value: Value::Float(left + right),
                    })
                }
                (Value::Float(left), Value::Float(right)) => self.execution_stack.push(Token {
                    value: Value::Float(left + right),
                }),
                (Value::Float(left), Value::Integer(ref right)) => {
                    let right = *right as f64;
                    self.execution_stack.push(Token {
                        value: Value::Float(left + right),
                    })
                }
                (Value::String(left), Value::String(right)) => self.execution_stack.push(Token {
                    value: Value::String(left + &right),
                }),
                (Value::Char(left), Value::Char(right)) => self.execution_stack.push(Token {
                    value: Value::String(left.to_string() + &right.to_string()),
                }),
                (Value::Char(left), Value::String(right)) => self.execution_stack.push(Token {
                    value: Value::String(left.to_string() + &right),
                }),
                (Value::String(left), Value::Char(right)) => self.execution_stack.push(Token {
                    value: Value::String(left + &right.to_string()),
                }),
                (Value::String(left), Value::Integer(right)) => self.execution_stack.push(Token {
                    value: Value::String(left + &right.to_string()),
                }),
                (Value::Integer(left), Value::String(right)) => self.execution_stack.push(Token {
                    value: Value::String(left.to_string() + &right),
                }),
                (Value::Char(left), Value::Integer(right)) => self.execution_stack.push(Token {
                    value: Value::String(left.to_string() + &right.to_string()),
                }),
                (Value::Integer(left), Value::Char(right)) => self.execution_stack.push(Token {
                    value: Value::String(left.to_string() + &right.to_string()),
                }),
                (Value::List(left), Value::List(right)) => {
                    let mut newlist = vec![];
                    newlist.clone_from(&*left);
                    let mut secondlist = vec![];
                    secondlist.clone_from(&*right);

                    newlist.append(&mut secondlist);
                    self.execution_stack.push(Token {
                        value: Value::List(Rc::new(newlist)),
                    })
                }
                _ => {
                    println!("cant add these two types");
                }
            }
        }
    }

    fn pow(&mut self) {
        if let (Some(right), Some(left)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            if let (Value::Integer(left), Value::Integer(right)) = (left.value, right.value) {
                self.execution_stack.push(Token {
                    value: Value::Integer(left.pow(right as u32)),
                })
            }
        }
    }

    fn neg(&mut self) {
        if let Some(left) = self.get_tokenifvar() {
            match left.value {
                Value::Integer(left) => self.execution_stack.push(Token {
                    value: Value::Integer(-left),
                }),
                Value::Float(left) => self.execution_stack.push(Token {
                    value: Value::Float(-left),
                }),
                _ => {
                    println!("cant neg this")
                }
            }
        }
    }

    fn sqrt(&mut self) {
        if let Some(left) = self.get_tokenifvar() {
            match left.value {
                Value::Integer(left) => self.execution_stack.push(Token {
                    value: Value::Float((left as f64).sqrt()),
                }),
                Value::Float(left) => self.execution_stack.push(Token {
                    value: Value::Float(left.sqrt()),
                }),
                _ => {
                    println!("cant sqrt this")
                }
            }
        }
    }

    fn stack_sub(&mut self) {
        if let (Some(right), Some(left)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            if let (Value::Integer(left), Value::Integer(right)) = (left.value, right.value) {
                self.execution_stack.push(Token {
                    value: Value::Integer(left - right),
                })
            }
        }
    }

    fn stack_modulo(&mut self) {
        if let (Some(right), Some(left)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            match (left.value, right.value) {
                (Value::Integer(left), Value::Integer(right)) => {
                    self.execution_stack.push(Token {
                        value: Value::Integer(left.modulo(right)),
                    });
                }
                _ => {
                    println!("Cant mod this type")
                }
            }
        }
    }

    fn stack_mul(&mut self) {
        if let (Some(right), Some(left)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            if let (Value::Integer(left), Value::Integer(right)) = (left.value, right.value) {
                self.execution_stack.push(Token {
                    value: Value::Integer(left * right),
                })
            }
        }
    }
    fn exit(&mut self) {
        std::process::exit(0)
    }

    fn clearscreen(&mut self) {
        clearscreen::clear().unwrap();
    }

    fn getch(&mut self) {
        let getch = Getch::new();
        if let Ok(char) = getch.getch() {
            self.execution_stack.push(Token {
                value: Value::Integer(char.into()),
            });
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
                    });
                }
            } else {
                // Int
                if let Ok(v) = line.parse() {
                    self.execution_stack.push(Token {
                        value: Value::Integer(v),
                    });
                }
            }
        } else if line.chars().count() == 1 {
            if let Some(char) = line.chars().next() {
                self.execution_stack.push(Token {
                    value: Value::Char(char),
                });
            }
        } else {
            self.execution_stack.push(Token {
                value: Value::String(line),
            });
        }
    }

    fn stack_print(&mut self) {
        if let Some(token) = self.get_tokenifvar() {
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

    // Handles calling block types
    // fn block_call(&mut self) {
    //     if let Some(function) = self.get_tokenifvar() {
    //         match function.value {
    //             Value::Block(ref block) => {
    //                 match block {
    //                     crate::token::BlockType::Literal(block) => {
    //                         self.call_stack.push(HashMap::new());
    //                         for t in block.iter() {
    //                             self.execute_token(t)
    //                         }
    //                         if let Some(token) = self.get_tokenifvar() {
    //                             self.execution_stack.push(token)
    //                         }
    //                         self.call_stack.pop();

    //                     },
    //                     crate::token::BlockType::Procedure(_) => todo!(),
    //                     crate::token::BlockType::Struct(_) => todo!(),
    //                 }
    //             }
    //             _ => {
    //                 println!("Cant call this type")
    //             }
    //         }
    //     }
    // }

    fn create_proc(&mut self) {
        if let Some(token) = self.get_tokenifvar() {
            match token.value {
                Value::Block(block) => match block {
                    BlockType::Literal(block) => self.execution_stack.push(Token {
                        value: Value::Block(BlockType::Procedure(block)),
                    }),
                    BlockType::Procedure(block) => self.execution_stack.push(Token {
                        value: Value::Block(BlockType::Procedure(block)),
                    }),
                    BlockType::Struct(block) => todo!(),
                    BlockType::Lambda(_) => todo!(),
                },
                _ => {
                    println!("cant turn this into procedure")
                }
            }
        }
    }

    fn block_call(&mut self) {
        if let Some(function) = self.execution_stack.pop() {
            match function.value {
                Value::Block(block) => {
                    match block {
                        BlockType::Literal(block) => {
                            // Call with new scope
                            self.call_stack.push(HashMap::new());
                            for t in block.iter() {
                                self.execute_token(t)
                            }
                            if let Some(token) = self.get_tokenifvar() {
                                self.execution_stack.push(token)
                            }
                            self.call_stack.pop();
                        }
                        BlockType::Procedure(block) => {
                            // call in same scope                    self.call_stack.push(HashMap::new());
                            for t in block.iter() {
                                self.execute_token(t)
                            }
                            if let Some(token) = self.get_tokenifvar() {
                                self.execution_stack.push(token)
                            }
                            self.call_stack.pop();
                            if let Some(token) = self.get_tokenifvar() {
                                self.execution_stack.push(token)
                            }
                        }
                        BlockType::Struct(_) => todo!(),
                        BlockType::Lambda(_) => todo!(),
                    }
                }
                _ => {
                    println!("Cant chain call this type")
                }
            }
        }
    }

    fn user_block_call(&mut self, function_name: &str) {
        if let Some(token) = self.get_tokenifident(function_name) {
            if let Value::Block(block) = token.value {
                match block {
                    BlockType::Literal(block) => {
                        // Call with new scope
                        self.call_stack.push(HashMap::new());
                        for t in block.iter() {
                            self.execute_token(t)
                        }
                        if let Some(token) = self.get_tokenifvar() {
                            self.execution_stack.push(token)
                        }
                        self.call_stack.pop();
                    }
                    BlockType::Procedure(block) => {
                        // call in same scope
                        for t in block.iter() {
                            self.execute_token(t)
                        }
                        if let Some(token) = self.get_tokenifvar() {
                            self.execution_stack.push(token)
                        }
                    }
                    BlockType::Struct(_) => todo!(),
                    BlockType::Lambda(_) => todo!(),
                }
            } else {
                println!("Cant call this type")
            }
        }
    }

    fn user_block_chain_call(&mut self) {
        self.execution_stack.reverse();
        if let Some(function) = self.execution_stack.pop() {
            match function.value {
                Value::Block(block) => {
                    match block {
                        BlockType::Literal(block) => {
                            // Call with new scope
                            self.call_stack.push(HashMap::new());
                            for t in block.iter() {
                                self.execute_token(t)
                            }
                            if let Some(token) = self.get_tokenifvar() {
                                self.execution_stack.push(token)
                            }
                            self.call_stack.pop();
                        }
                        BlockType::Procedure(block) => {
                            // call in same scope                    self.call_stack.push(HashMap::new());
                            for t in block.iter() {
                                self.execute_token(t)
                            }
                            if let Some(token) = self.get_tokenifvar() {
                                self.execution_stack.push(token)
                            }
                            self.call_stack.pop();
                            if let Some(token) = self.get_tokenifvar() {
                                self.execution_stack.push(token)
                            }
                        }
                        BlockType::Struct(_) => todo!(),
                        BlockType::Lambda(_) => todo!(),
                    }
                }
                _ => {
                    println!("Cant chain call this type")
                }
            }
        }
        self.execution_stack.reverse();
    }

    // Macros ( no scope created )
    // fn macro_call(&mut self) {
    //     if let Some(function) = self.get_tokenifvar() {
    //         match function.value {
    //             Value::Block(ref block) => {
    //                 for t in block.iter() {
    //                     self.execute_token(t)
    //                 }
    //             }
    //             _ => self.execution_stack.push(function),
    //         }
    //     }
    // }

    // fn user_macro_call(&mut self, function_name: &str) {
    //     if let Some(token) = self.get_tokenifident(function_name) {
    //         if let Value::Block(ref block) = token.value {
    //             for t in block.iter() {
    //                 self.execute_token(t)
    //             }
    //         } else {
    //             println!("Cant macro this type")
    //         }
    //     }
    // }

    // fn user_macro_chain_call(&mut self) {
    //     self.execution_stack.reverse();
    //     if let Some(function) = self.execution_stack.pop() {
    //         match function.value {
    //             Value::Block(ref block) => {
    //                 for t in block.iter() {
    //                     self.execute_token(t)
    //                 }
    //             }
    //             _ => {
    //                 println!("Cant macro this type")
    //             }
    //         }
    //     }
    //     self.execution_stack.reverse();
    // }

    fn stack_println(&mut self) {
        if let Some(token) = self.get_tokenifvar() {
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
                Value::List(_) => {
                    println!("LIST")
                }
                Value::Nothing => todo!(),
                Value::UserBlockCall(_) => {
                    println!("Block Call")
                }
            }
        }
    }

    fn flush(&self) {
        std::io::stdout().flush().unwrap();
    }

    fn access_call(&mut self) {
        if let (Some(ident), Some(block)) = (self.execution_stack.pop(), self.get_tokenifvar()) {
            match (ident.value, block.value) {
                (Value::Identifier(identifier), Value::List(block)) => match identifier.as_str() {
                    "len" => self.execution_stack.push(Token {
                        value: Value::Integer(block.len() as i128),
                    }),
                    _ => {
                        println!("{} not reconized in access call", identifier)
                    }
                },
                (Value::Integer(index), Value::List(block)) => {
                    if let Some(item) = block.get(index as usize) {
                        self.execution_stack.push(item.clone());
                    } else {
                        println!("cant get this index, nothing here")
                    }
                }
                (Value::Identifier(identifier), Value::Block(BlockType::Literal(ref block))) => match identifier.as_str() {
                    "len" => self.execution_stack.push(Token {
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
                },
                _ => {
                    println!("cant access_call these")
                }
            }
        }
    }

    fn list_push(&mut self) {
        if let (Some(item), Some(list)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            match list.value {
                Value::List(list) => {
                    let mut newlist = Vec::new();
                    newlist.clone_from(&*list);
                    newlist.push(item);
                    self.execution_stack.push(Token {
                        value: Value::List(Rc::new(newlist)),
                    })
                }
                _ => {
                    println!("cant push to this ")
                }
            }
        }
    }

    fn list_insert(&mut self) {
        if let (Some(index), Some(item), Some(list)) = (
            self.get_tokenifvar(),
            self.get_tokenifvar(),
            self.get_tokenifvar(),
        ) {
            match (index.value, list.value) {
                (Value::Integer(i), Value::List(list)) => {
                    let mut newlist = Vec::new();
                    newlist.clone_from(&*list);
                    if i as usize > newlist.len() {
                        newlist.push(item);
                    } else {
                        newlist.insert(i.abs() as usize, item);
                    }
                    self.execution_stack.push(Token {
                        value: Value::List(Rc::new(newlist)),
                    })
                }
                _ => {
                    println!("cant insert to this ")
                }
            }
        }
    }

    fn list_remove(&mut self) {
        if let (Some(index), Some(list)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            match (index.value, list.value) {
                (Value::Integer(i), Value::List(list)) => {
                    let mut newlist = Vec::new();
                    newlist.clone_from(&*list);
                    if i as usize > newlist.len() {
                        newlist.pop();
                    } else {
                        newlist.remove(i.abs() as usize);
                    }
                    self.execution_stack.push(Token {
                        value: Value::List(Rc::new(newlist)),
                    })
                }
                _ => {
                    println!("cant remove to this ")
                }
            }
        }
    }

    fn list_pop(&mut self) {
        if let (Some(ident), Some(list)) = (self.execution_stack.pop(), self.get_tokenifvar()) {
            match (ident.value, list.value) {
                (Value::Identifier(ident), Value::List(list)) => {
                    let mut newlist = Vec::new();
                    newlist.clone_from(&*list);
                    if let Some(item) = newlist.pop() {
                        if ident != "_" {
                            if let Some(scope) = self.call_stack.last_mut() {
                                scope.insert(ident, item);
                            }
                        }
                    }

                    self.execution_stack.push(Token {
                        value: Value::List(Rc::new(newlist)),
                    })
                }
                _ => {
                    println!("cant remove to this ")
                }
            }
        }
    }

    fn sleep(&mut self) {
        if let Some(token) = self.get_tokenifvar() {
            if let Value::Integer(time) = token.value {
                let delay = time::Duration::from_millis(time as u64);
                thread::sleep(delay);
            }
        }
    }

    fn random(&mut self) {
        if let (Some(right), Some(left)) = (self.get_tokenifvar(), self.get_tokenifvar()) {
            match (left.value, right.value) {
                (Value::Integer(left), Value::Integer(right)) => {
                    if left <= right {
                        let mut rng = rand::thread_rng();
                        self.execution_stack.push(Token {
                            value: Value::Integer(rng.gen_range(left..=right)),
                        })
                    } else {
                        let mut rng = rand::thread_rng();
                        self.execution_stack.push(Token {
                            value: Value::Integer(rng.gen_range(right..=left)),
                        })
                    }
                }
                _ => {
                    println!("cant make random")
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

//     // "store_import" => {
//     //     if let Some(a) = self.execution_stack.pop() {
//     //         // Get filename from argument
//     //         let mut lexer = lexer::Lexer::new_from_file(&a.value);

//     //         // Parse the file into tokens
//     //         lexer.parse();

//     //         self.execution_stack.push(Token {
//     //
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
//     //
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
//     //
//     //             })
//     //         } else {
//     //             let mut rng = rand::thread_rng();

//     //             self.execution_stack.push(Token {
//     //                 token_type: TokenTypes::Integer,
//     //                 value: (rng.gen_range(f..=s)).to_string(),
//     //                 line_number: 0,
//     //                 row: 0,
//     //                 block: vec![],
//     //
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
//     //
//     //                 token_type: TokenTypes::Bool,
//     //                 value: "false".to_string(),
//     //                 block: vec![],
//     //                 line_number: 0,
//     //                 row: 0,
//     //             })
//     //         } else {
//     //             self.execution_stack.push(Token {
//     //
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
//     //     if self.block_call {
//     //         self.exit_loop = true;
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
