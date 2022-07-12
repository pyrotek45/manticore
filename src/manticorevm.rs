use rand::Rng;
use std::process::Command;
use std::{collections::HashMap, io::Write};

use crate::{
    lexer::{self},
    parser::Parser,
    string_utils::{is_string_number, print_error, trim_newline},
    token::{Token, TokenTypes},
};

pub struct ManitcoreVm {
    instruction_tokens: Vec<Token>,
    execution_stack: Vec<Token>,
    last_instruction: String,
    stack_set: usize,
    file: String,
    pub exit_loop: bool,
    pub debug: bool,
    pub method_call: bool,
    pub core_self: Vec<Token>,
    heap: HashMap<String, Token>,
}

impl ManitcoreVm {
    pub fn new(tokenlist: &[Token], file: &str) -> Self {
        Self {
            instruction_tokens: tokenlist.to_vec(),
            execution_stack: Vec::new(),
            file: file.to_string(),
            last_instruction: String::new(),
            debug: false,
            heap: HashMap::new(),
            stack_set: 0,
            exit_loop: false,
            method_call: false,
            core_self: vec![],
        }
    }

    // Proccess each token
    pub fn execute(&mut self) {
        for i in self.instruction_tokens.clone() {
            // If token is an identifier and its value is found on the heap
            // push the heap value instead

            self.execute_token(&i);
            if self.exit_loop {
                break;
            }

            // for t in core_self {
            //     println!("{} in core ", t.value)
            // }
        }

        if self.debug {
            for (k, v) in &self.heap {
                if let Some(p) = &v.proxy {
                    println!("{} -> ({} ~ {})", k, v.value, &p)
                } else {
                    println!("{} -> ({} ~ None)", k, v.value)
                }
            }
            println!()
        }
    }

    pub fn execute_token(&mut self, i: &Token) {
        if i.token_type == TokenTypes::Identifier {
            if i.value == "self" {
                for (key, value) in &self.heap {
                    self.core_self.push(Token {
                        proxy: None,
                        token_type: TokenTypes::Function,
                        value: "var".to_string(),
                        block: vec![],
                        line_number: 0,
                        row: 0,
                    });
                    self.core_self.push(Token {
                        proxy: None,
                        token_type: TokenTypes::Identifier,
                        value: key.to_string(),
                        block: vec![],
                        line_number: 0,
                        row: 0,
                    });
                    self.core_self.push(Token {
                        proxy: None,
                        token_type: TokenTypes::Symbol,
                        value: ":".to_string(),
                        block: vec![],
                        line_number: 0,
                        row: 0,
                    });
                    self.core_self.push(value.clone());
                    self.core_self.push(Token {
                        proxy: None,
                        token_type: TokenTypes::Symbol,
                        value: ";".to_string(),
                        block: vec![],
                        line_number: 0,
                        row: 0,
                    });
                }

                self.execution_stack.push(Token {
                    proxy: None,
                    token_type: TokenTypes::Block,
                    value: "self".to_string(),
                    block: self.core_self.clone(),
                    line_number: 0,
                    row: 0,
                });
                return;
            }

            if let Some(tok) = self.heap.get(&i.value) {
                self.execution_stack.push(Token {
                    token_type: tok.token_type,
                    value: tok.value.clone(),
                    line_number: i.line_number,
                    row: i.row,
                    block: tok.block.clone(),
                    proxy: tok.proxy.clone(),
                });
                return;
            }
            self.execution_stack.push(i.clone());
            //continue;
        }

        // Strings , blocks, list, numbers and bools get pushed
        // onto the execution stack automatically
        if i.token_type == TokenTypes::String {
            self.execution_stack.push(i.clone());
            return;
        }

        if i.token_type == TokenTypes::Block {
            self.execution_stack.push(i.clone());
            return;
        }

        if i.token_type == TokenTypes::Number {
            self.execution_stack.push(i.clone());
            return;
        }
        if i.token_type == TokenTypes::Bool {
            self.execution_stack.push(i.clone());
            return;
        }
        if i.token_type == TokenTypes::Break {
            self.execution_stack.push(i.clone());
            return;
        }
        if i.token_type == TokenTypes::List {
            self.execution_stack.push(i.clone());
            return;
        }
        //Match values for each token
        match i.value.to_lowercase().as_str() {
            // If left paren is found then one must be missing the other pair
            "(" => print_error(
                "Possibly missing ')' pair",
                i.line_number,
                i.row,
                &self.file,
                &self.last_instruction,
            ),
            "readln" => {
                let mut line = String::new();
                std::io::stdin().read_line(&mut line).unwrap();
                let line = trim_newline(&mut line);

                if is_string_number(&line) {
                    self.execution_stack.push(Token {
                        proxy: None,
                        token_type: TokenTypes::Number,
                        value: line,
                        block: vec![],
                        line_number: 0,
                        row: 0,
                    })
                } else {
                    self.execution_stack.push(Token {
                        proxy: None,
                        token_type: TokenTypes::String,
                        value: line,
                        block: vec![],
                        line_number: 0,
                        row: 0,
                    })
                }
            }
            "break" => {
                self.exit_loop = true;
            }
            "command" => {
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut cargs = vec![];
                    for arg in a.block {
                        cargs.push(arg.value.clone())
                    }

                    Command::new(&b.value)
                        .args(cargs)
                        .spawn()
                        .unwrap_or_else(|_| panic!(" {} command failed to start", &a.value));
                }
            }
            "store_import" => {
                if let Some(a) = self.execution_stack.pop() {
                    // Get filename from argument
                    let mut lexer = lexer::Lexer::new_from_file(&a.value);

                    // Parse the file into tokens
                    lexer.parse();

                    self.execution_stack.push(Token {
                        proxy: None,
                        token_type: TokenTypes::Block,
                        value: "Block".to_string(),
                        block: lexer.block_stack[0].clone(),
                        line_number: 0,
                        row: 0,
                    })
                }
            }
            "import" => {
                if let Some(a) = self.execution_stack.pop() {
                    let mut lexer = lexer::Lexer::new_from_file(&a.value);
                    // Parse the file into tokens
                    lexer.parse();

                    // Create new vm
                    let mut parser = Parser::new();
                    if self.debug {
                        parser.debug = true;
                    }

                    // Shunt tokens in vm
                    let shunted = parser.shunt(&lexer.block_stack[0]).clone();
                    let mut vm = ManitcoreVm::new(&shunted, &a.value);
                    if self.debug {
                        vm.debug = true;
                    }

                    // Copy the stack and the heap inside the vm
                    vm.execution_stack = self.execution_stack.clone();
                    vm.heap = self.heap.clone();

                    // Run the vm
                    vm.execute();

                    self.heap = vm.heap.clone();
                    self.execution_stack = vm.execution_stack.clone();
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                }
            }
            "store_url" => {
                if let Some(a) = self.execution_stack.pop() {
                    if let Ok(body) = ureq::get(&a.value).call() {
                        if let Ok(body) = body.into_string() {
                            // Get filename from argument
                            let mut lexer = lexer::Lexer::new_from_string(&body);

                            // Parse the file into tokens
                            lexer.parse();

                            self.execution_stack.push(Token {
                                proxy: None,
                                token_type: TokenTypes::Block,
                                value: "Block".to_string(),
                                block: lexer.block_stack[0].clone(),
                                line_number: 0,
                                row: 0,
                            })
                        }
                    }
                }
            }
            "import_url" => {
                if let Some(a) = self.execution_stack.pop() {
                    if let Ok(body) = ureq::get(&a.value).call() {
                        if let Ok(body) = body.into_string() {
                            // Get filename from argument
                            let mut lexer = lexer::Lexer::new_from_string(&body);

                            // Parse the file into tokens
                            lexer.parse();
                            let mut parser = Parser::new();
                            if self.debug {
                                parser.debug = true;
                            }

                            // Store now parsed tokens into a new list
                            let shunted = parser.shunt(&lexer.block_stack[0]).clone();
                            let mut vm = ManitcoreVm::new(&shunted, &body);
                            if self.debug {
                                vm.debug = true;
                            }

                            // Copy the stack and the heap inside the vm
                            vm.execution_stack = self.execution_stack.clone();
                            vm.heap = self.heap.clone();

                            // Run the vm
                            vm.execute();

                            self.heap = vm.heap.clone();
                            self.execution_stack = vm.execution_stack.clone();
                        }
                    }
                }
            }
            "run_url" => {
                if let Some(a) = self.execution_stack.pop() {
                    if let Ok(body) = ureq::get(&a.value).call() {
                        if let Ok(body) = body.into_string() {
                            // Get filename from argument
                            let mut lexer = lexer::Lexer::new_from_string(&body);

                            // Parse the file into tokens
                            lexer.parse();
                            let mut parser = Parser::new();
                            if self.debug {
                                parser.debug = true;
                            }

                            // Store now parsed tokens into a new list
                            let shunted = parser.shunt(&lexer.block_stack[0]).clone();
                            let mut vm = ManitcoreVm::new(&shunted, &body);
                            if self.debug {
                                vm.debug = true;
                            }

                            // Execute the vm using parsed token list
                            vm.execute();
                        }
                    }
                }
            }
            "sqrt" => {
                if let Some(a) = self.execution_stack.pop() {
                    let mut f: f32 = 0.0;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Number,
                        value: (f.sqrt()).to_string(),
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "randomf" => {
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut f: f32 = 0.0;
                    let mut s: f32 = 0.0;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if let Ok(v) = b.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if s <= f {
                        let mut rng = rand::thread_rng();

                        self.execution_stack.push(Token {
                            token_type: TokenTypes::Number,
                            value: (rng.gen_range(s..=f)).to_string(),
                            line_number: 0,
                            row: 0,
                            block: vec![],
                            proxy: None,
                        })
                    } else {
                        let mut rng = rand::thread_rng();

                        self.execution_stack.push(Token {
                            token_type: TokenTypes::Number,
                            value: (rng.gen_range(f..=s)).to_string(),
                            line_number: 0,
                            row: 0,
                            block: vec![],
                            proxy: None,
                        })
                    }
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "random_int" => {
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut f: i32 = 0;
                    let mut s: i32 = 0;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if let Ok(v) = b.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if s <= f {
                        let mut rng = rand::thread_rng();

                        self.execution_stack.push(Token {
                            token_type: TokenTypes::Number,
                            value: (rng.gen_range(s..=f)).to_string(),
                            line_number: 0,
                            row: 0,
                            block: vec![],
                            proxy: None,
                        })
                    } else {
                        let mut rng = rand::thread_rng();

                        self.execution_stack.push(Token {
                            token_type: TokenTypes::Number,
                            value: (rng.gen_range(f..=s)).to_string(),
                            line_number: 0,
                            row: 0,
                            block: vec![],
                            proxy: None,
                        })
                    }
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "pow" => {
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut f: f32 = 0.0;
                    let mut s: f32 = 0.0;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if let Ok(v) = b.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Number,
                        value: (s.powf(f)).to_string(),
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "neg" => {
                if let Some(a) = self.execution_stack.pop() {
                    let mut f: f32 = 0.0;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Number,
                        value: (-f).to_string(),
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "insert" => {
                if let (Some(index), Some(item), Some(mut list)) = (
                    self.execution_stack.pop(),
                    self.execution_stack.pop(),
                    self.execution_stack.pop(),
                ) {

                    let mut i: usize = 0;
                    if let Ok(v) = index.value.parse() {
                        i = v
                    } else {
                        print_error(
                            "expected a number",
                            index.line_number,
                            index.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }
                    if i > list.block.len() {
                        list.block.push(item);
                    } else {
                        list.block.insert(i, item);
                    }
                    self.execution_stack.push(list)
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                }
            }
            "remove" => {
                if let (Some(a), Some(mut b)) =
                    (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut i: usize = 0;
                    if let Ok(v) = a.value.parse() {
                        i = v
                    } else {
                        print_error(
                            "expected a number",
                            a.line_number,
                            a.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }
                    if i > b.block.len() {
                        b.block.pop();
                    } else {
                        b.block.remove(i);
                    }

                    self.execution_stack.push(b)
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                }
            }
            "push" => {
                if let (Some(a), Some(mut b)) =
                    (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    b.block.push(a);
                    self.execution_stack.push(b)
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                }
            }
            "pop" => {
                if let (Some(a), Some(mut b)) =
                    (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    if let Some(mut item) = b.block.pop() {
                        if let Some(p) = a.proxy {
                            item.proxy = Some(p.clone());
                            self.heap.insert(p, item);
                        } else {
                            item.proxy = Some(a.value.clone());
                            self.heap.insert(a.value.clone(), item);
                        }
                    } else {
                        print_error(
                            format!("Could not pop list {}, Not enough items", b.value).as_str(),
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }
                    self.execution_stack.push(b)
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                }
            }
            "." => {
                if let (Some(id), Some(block)) =
                    (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    match id.token_type {
                        TokenTypes::Number => {
                            let mut i: usize = 0;
                            if let Ok(v) = id.value.parse() {
                                i = v
                            } else {
                                print_error(
                                    "expected a number",
                                    id.line_number,
                                    id.row,
                                    &self.file,
                                    &self.last_instruction,
                                )
                            }
                            if let Some(item) = block.block.get(i) {
                                self.execution_stack.push(item.clone())
                            } else if let Some(name) = block.proxy {
                                print_error(
                                    format!("Could not get index from {0} at index: {1}, {0} only has length of {2}  NOTE: list start at index 0. Expected value: 0 to {3}", name,id.value,block.block.len(),block.block.len()-1).as_str(),
                                    id.line_number,
                                    id.row,
                                    &self.file,
                                    &self.last_instruction,
                                )
                            } else {
                                print_error(
                                    format!("Could not get index from BLOCK at index: {0}, BLOCK only has length of {1}  NOTE: list start at index 0. Expected value: 0 to {2}", id.value,block.block.len(),block.block.len()-1).as_str(),
                                    id.line_number,
                                    id.row,
                                    &self.file,
                                    &self.last_instruction,
                                )
                            }
                        }
                        TokenTypes::Identifier => {
                            match id.value.as_str() {
                                "run" => {
                                    let mut parser = Parser::new();
                                    if self.debug {
                                        parser.debug = true
                                    }
                                    let shunted = parser.shunt(&block.block).clone();
                                    let mut vm = ManitcoreVm::new(&shunted, &block.value);
                                    if self.debug {
                                        vm.debug = true
                                    }
                                    vm.method_call = true;
                                    vm.execution_stack = self.execution_stack.clone();
                                    // vm.heap = self.heap.clone();

                                    // Run the vm
                                    vm.execute();

                                    for _x in 0..vm.stack_set {
                                        if let Some(_t) = self.execution_stack.pop() {}
                                    }
                                    if let Some(t) = vm.execution_stack.pop() {
                                        self.execute_token(&t);
                                    }
                                }
                                "len" => {
                                    let length = block.block.len();
                                    self.execution_stack.push(Token {
                                        proxy: None,
                                        token_type: TokenTypes::Number,
                                        value: length.to_string(),
                                        block: vec![],
                                        line_number: block.line_number,
                                        row: block.row,
                                    })
                                }
                                _ => {
                                    let mut parser = Parser::new();
                                    if self.debug {
                                        parser.debug = true
                                    }
                                    let shunted = parser.shunt(&block.block).clone();
                                    let mut vm = ManitcoreVm::new(&shunted, &block.value);
                                    if self.debug {
                                        vm.debug = true
                                    }

                                    vm.execute();

                                    if let Some(tok) = vm.heap.get(&id.value) {
                                        self.execution_stack.push(tok.clone())
                                    }
                                }
                            }
                        }
                        _ => {
                            if let Some(newid) = id.proxy {
                                let mut parser = Parser::new();
                                if self.debug {
                                    parser.debug = true
                                }
                                let shunted = parser.shunt(&block.block).clone();
                                let mut vm = ManitcoreVm::new(&shunted, &block.value);
                                if self.debug {
                                    vm.debug = true
                                }

                                vm.method_call = true;
                                vm.execute();

                                if let Some(tok) = vm.heap.get(&newid) {
                                    self.execution_stack.push(tok.clone())
                                }
                            }
                        }
                    }
                }
            }
            "loop" => {
                if let Some(block) = self.execution_stack.pop() {
                    loop {
                        let mut parser = Parser::new();
                        if self.debug {
                            parser.debug = true
                        }
                        let shunted = parser.shunt(&block.block).clone();
                        for t in &shunted {
                            self.execute_token(t)
                        }
                        if self.exit_loop {
                            break;
                        }
                    }
                }
            }
            "for" => {
                if let (Some(block), Some(list), Some(ident)) = (
                    self.execution_stack.pop(),
                    self.execution_stack.pop(),
                    self.execution_stack.pop(),
                ) {
                    for mut var in list.block {
                        let mut parser = Parser::new();
                        if self.debug {
                            parser.debug = true
                        }
                        let shunted = parser.shunt(&block.block).clone();
                        let mut vm = ManitcoreVm::new(&shunted, &block.value);
                        if self.debug {
                            vm.debug = true
                        }
                        vm.heap = self.heap.clone();

                        if let Some(p) = &ident.proxy {
                            var.proxy = Some(p.clone());
                            vm.heap.insert(p.clone(), var.clone());
                        } else {
                            var.proxy = Some(ident.value.clone());
                            vm.heap.insert(ident.value.clone(), var.clone());
                        }

                        vm.execute();
                        if vm.exit_loop {
                            break;
                        }
                        self.heap = vm.heap.clone();
                    }
                }
            }
            "range" => {
                if let (Some(end), Some(start)) =
                    (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut s: usize = 0;
                    let mut e: usize = 0;

                    if let Ok(v) = start.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }
                    if let Ok(v) = end.value.parse() {
                        e = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    let mut new_list: Vec<Token> = Vec::new();

                    for x in s..=e {
                        new_list.push(Token {
                            proxy: None,
                            token_type: TokenTypes::Number,
                            value: x.to_string(),
                            block: vec![],
                            line_number: 0,
                            row: 0,
                        })
                    }

                    self.execution_stack.push(Token {
                        proxy: None,
                        token_type: TokenTypes::List,
                        value: "list".to_string(),
                        block: new_list.clone(),
                        line_number: 0,
                        row: 0,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                }
            }
            "shc" => self.heap.clear(),
            "exist" => {
                if let Some(t) = self.execution_stack.pop() {
                    if t.token_type == TokenTypes::Nothing {
                        self.execution_stack.push(Token {
                            proxy: None,
                            token_type: TokenTypes::Bool,
                            value: "false".to_string(),
                            block: vec![],
                            line_number: 0,
                            row: 0,
                        })
                    } else {
                        self.execution_stack.push(Token {
                            proxy: None,
                            token_type: TokenTypes::Bool,
                            value: "true".to_string(),
                            block: vec![],
                            line_number: 0,
                            row: 0,
                        })
                    }
                }
            }
            "rm" => if let Some(_t) = self.execution_stack.pop() {},
            // Used to tie more than 1 token at a time from the stack
            "set" | "~" => {
                let mut variable_stack: Vec<String> = Vec::new();
                self.stack_set = 0;

                // Pop from stack untill no more identifiers
                while let Some(k) = self.execution_stack.last() {
                    if k.token_type == TokenTypes::Identifier {
                        if let Some(tok) = self.execution_stack.pop() {
                            variable_stack.push(tok.value.clone());
                        }
                    } else {
                        if let Some(t) = self.execution_stack.last() {
                            if t.token_type == TokenTypes::Break {
                                self.execution_stack.pop();
                            }
                        }
                        break;
                    }
                }

                // Tie each value into the heap using the tokens poped
                for values in variable_stack {
                    if let Some(mut tok) = self.execution_stack.pop() {
                        self.stack_set += 1;
                        tok.proxy = Some(values.clone());
                        self.heap.insert(values, tok.clone());
                    } else {
                        self.heap.insert(
                            values.clone(),
                            Token {
                                proxy: Some(values),
                                token_type: TokenTypes::Nothing,
                                value: "nothing".to_string(),
                                block: vec![],
                                line_number: 0,
                                row: 0,
                            },
                        );
                    }
                }
            }
            "?" => {
                if self.method_call {
                    self.exit_loop = true;
                }
            }
            // This function will pop off a block and execute it using the outer scope heap and stack
            "call" => {
                if let Some(a) = self.execution_stack.pop() {
                    // Create new vm
                    let mut parser = Parser::new();
                    if self.debug {
                        parser.debug = true;
                    }

                    // Shunt tokens in vm
                    let shunted = parser.shunt(&a.block).clone();
                    let mut vm = ManitcoreVm::new(&shunted, &a.value);
                    if self.debug {
                        vm.debug = true;
                    }

                    // Copy the stack and the heap inside the vm
                    vm.execution_stack = self.execution_stack.clone();
                    vm.heap = self.heap.clone();

                    // Run the vm
                    vm.execute();

                    self.heap = vm.heap.clone();
                    self.execution_stack = vm.execution_stack.clone();
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                }
            }
            // This function will pop off a block and execute it using the outer scope heap and stack
            "let" => {
                if let Some(a) = self.execution_stack.pop() {
                    let mut core_self = vec![];
                    for (key, value) in &self.heap {
                        core_self.push(Token {
                            proxy: None,
                            token_type: TokenTypes::Function,
                            value: "var".to_string(),
                            block: vec![],
                            line_number: 0,
                            row: 0,
                        });
                        core_self.push(Token {
                            proxy: None,
                            token_type: TokenTypes::Identifier,
                            value: key.to_string(),
                            block: vec![],
                            line_number: 0,
                            row: 0,
                        });
                        core_self.push(Token {
                            proxy: None,
                            token_type: TokenTypes::Symbol,
                            value: ":".to_string(),
                            block: vec![],
                            line_number: 0,
                            row: 0,
                        });
                        core_self.push(value.clone());
                        core_self.push(Token {
                            proxy: None,
                            token_type: TokenTypes::Symbol,
                            value: ";".to_string(),
                            block: vec![],
                            line_number: 0,
                            row: 0,
                        });
                    }

                    core_self.append(&mut a.block.clone());

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Block,
                        value: a.value.clone(),
                        line_number: 0,
                        row: 0,
                        block: core_self.clone(),
                        proxy: a.proxy,
                    });
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                }
            }
            // This function will pop off a block and execute it using the outer scope heap and stack
            "@" => {
                if let Some(a) = self.execution_stack.pop() {
                    // Create new vm
                    let mut parser = Parser::new();
                    if self.debug {
                        parser.debug = true;
                    }

                    // Shunt tokens in vm
                    let shunted = parser.shunt(&a.block).clone();
                    let mut vm = ManitcoreVm::new(&shunted, &a.value);
                    if self.debug {
                        vm.debug = true;
                    }

                    // Copy the stack and the heap inside the vm
                    vm.execution_stack = self.execution_stack.clone();
                    // vm.heap = self.heap.clone();

                    // Run the vm
                    vm.execute();

                    for _x in 0..vm.stack_set {
                        if let Some(_t) = self.execution_stack.pop() {}
                    }

                    // Copy the last item to return from inside the vm to the outside
                    if let Some(return_value) = vm.execution_stack.pop() {
                        self.execution_stack.push(return_value)
                    }
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                }
            }
            "ret" => {
                if let (Some(a), Some(b), Some(c)) = (
                    self.execution_stack.pop(),
                    self.execution_stack.pop(),
                    self.execution_stack.pop(),
                ) {
                    if c.value == b.value {
                        self.execution_stack.push(a)
                    }
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "if" => {
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    //if true single if statement
                    if b.token_type == TokenTypes::Bool {
                        if b.value == "true" {
                            let mut parser = Parser::new();
                            if self.debug {
                                parser.debug = true
                            }
                            let shunted = parser.shunt(&a.block).clone();
                            for t in &shunted {
                                self.execute_token(t)
                            }
                        }
                    } else if let Some(c) = self.execution_stack.pop() {
                        if c.token_type == TokenTypes::Bool {
                            if c.value == "true" {
                                let mut parser = Parser::new();
                                if self.debug {
                                    parser.debug = true
                                }
                                let shunted = parser.shunt(&b.block).clone();
                                for t in &shunted {
                                    self.execute_token(t)
                                }
                            } else {
                                let mut parser = Parser::new();
                                if self.debug {
                                    parser.debug = true
                                }
                                let shunted = parser.shunt(&a.block).clone();
                                for t in &shunted {
                                    self.execute_token(t)
                                }
                            }
                        }
                    }
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "sec" => {
                self.execution_stack.clear();
            }
            "rev" => {
                self.execution_stack.reverse();
            }
            "var" | "=" => {
                if let (Some(mut a), Some(b)) =
                    (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    if b.token_type == TokenTypes::Identifier || b.proxy.is_some() {
                        match a.token_type {
                            TokenTypes::Block => {
                                if let Some(p) = b.proxy {
                                    a.proxy = Some(p.clone());
                                    let mut recurse = vec![
                                        Token {
                                            proxy: None,
                                            token_type: TokenTypes::Function,
                                            value: "var".to_string(),
                                            block: vec![],
                                            line_number: 0,
                                            row: 0,
                                        },
                                        Token {
                                            proxy: None,
                                            token_type: TokenTypes::Identifier,
                                            value: p.clone(),
                                            block: vec![],
                                            line_number: 0,
                                            row: 0,
                                        },
                                        Token {
                                            proxy: None,
                                            token_type: TokenTypes::Symbol,
                                            value: ":".to_string(),
                                            block: vec![],
                                            line_number: 0,
                                            row: 0,
                                        },
                                        a.clone(),
                                        Token {
                                            proxy: None,
                                            token_type: TokenTypes::Symbol,
                                            value: ";".to_string(),
                                            block: vec![],
                                            line_number: 0,
                                            row: 0,
                                        },
                                    ];
                                    recurse.append(&mut a.block);
                                    self.heap.insert(
                                        p,
                                        Token {
                                            proxy: Some(b.value),
                                            token_type: TokenTypes::Block,
                                            value: "block".to_string(),
                                            block: recurse.clone(),
                                            line_number: 0,
                                            row: 0,
                                        },
                                    );
                                } else {
                                    a.proxy = Some(b.value.clone());
                                    let mut recurse = vec![
                                        Token {
                                            proxy: None,
                                            token_type: TokenTypes::Function,
                                            value: "var".to_string(),
                                            block: vec![],
                                            line_number: 0,
                                            row: 0,
                                        },
                                        Token {
                                            proxy: None,
                                            token_type: TokenTypes::Identifier,
                                            value: b.value.clone(),
                                            block: vec![],
                                            line_number: 0,
                                            row: 0,
                                        },
                                        Token {
                                            proxy: None,
                                            token_type: TokenTypes::Symbol,
                                            value: ":".to_string(),
                                            block: vec![],
                                            line_number: 0,
                                            row: 0,
                                        },
                                        a.clone(),
                                        Token {
                                            proxy: None,
                                            token_type: TokenTypes::Symbol,
                                            value: ";".to_string(),
                                            block: vec![],
                                            line_number: 0,
                                            row: 0,
                                        },
                                    ];
                                    recurse.append(&mut a.block);
                                    self.heap.insert(
                                        b.value.clone(),
                                        Token {
                                            proxy: Some(b.value),
                                            token_type: TokenTypes::Block,
                                            value: "block".to_string(),
                                            block: recurse.clone(),
                                            line_number: 0,
                                            row: 0,
                                        },
                                    );
                                }
                            }
                            _ => {
                                if let Some(p) = b.proxy {
                                    a.proxy = Some(p.clone());
                                    self.heap.insert(p, a);
                                } else {
                                    a.proxy = Some(b.value.clone());
                                    self.heap.insert(b.value, a);
                                }
                            }
                        }
                    } else {
                        print_error(
                            "expected an identifier",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "lss" => {
                // todo: does not support blocks atm
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut f: f32 = 0.0;
                    let mut s: f32 = 0.0;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a normal token",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if let Ok(v) = b.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a normal token",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Bool,
                        value: (s < f).to_string(),
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "gtr" => {
                // todo: does not support blocks atm
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut f: f32 = 0.0;
                    let mut s: f32 = 0.0;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a normal token",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if let Ok(v) = b.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a normal token",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }
                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Bool,
                        value: (s > f).to_string(),
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "equ" => {
                // todo: does not support blocks atm
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut f: String = String::new();
                    let mut s: String = String::new();

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a normal token",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if let Ok(v) = b.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a normal token",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Bool,
                        value: (f == s).to_string(),
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "not" => {
                // todo: does not support blocks atm
                if let Some(a) = self.execution_stack.pop() {
                    let mut f: bool = false;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a normal token",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Bool,
                        value: (!f).to_string(),
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "and" => {
                // todo: does not support blocks atm
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut f: bool = false;
                    let mut s: bool = false;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a normal token",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if let Ok(v) = b.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a normal token",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Bool,
                        value: (f && s).to_string(),
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "or" => {
                // todo: does not support blocks atm
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut f: bool = false;
                    let mut s: bool = false;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a normal token",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if let Ok(v) = b.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a normal token",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Bool,
                        value: (f || s).to_string(),
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "+" => {
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut f: f32 = 0.0;
                    let mut s: f32 = 0.0;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if let Ok(v) = b.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Number,
                        value: (f + s).to_string(),
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "-" => {
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut f: f32 = 0.0;
                    let mut s: f32 = 0.0;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if let Ok(v) = b.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Number,
                        value: (s - f).to_string(),
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "*" => {
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut f: f32 = 0.0;
                    let mut s: f32 = 0.0;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        );
                        println!("{}", a.value)
                    }

                    if let Ok(v) = b.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Number,
                        value: (f * s).to_string(),
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "/" => {
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut f: f32 = 0.0;
                    let mut s: f32 = 0.0;

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if let Ok(v) = b.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a number",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Number,
                        value: (s / f).to_string(),
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "concat" => {
                if let (Some(a), Some(b)) = (self.execution_stack.pop(), self.execution_stack.pop())
                {
                    let mut f: String = String::new();
                    let mut s: String = String::new();

                    if let Ok(v) = a.value.parse() {
                        f = v
                    } else {
                        print_error(
                            "expected a string",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    if let Ok(v) = b.value.parse() {
                        s = v
                    } else {
                        print_error(
                            "expected a string",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    }

                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Number,
                        value: s + &f,
                        line_number: 0,
                        row: 0,
                        block: vec![],
                        proxy: None,
                    })
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "dup" => {
                if let Some(a) = self.execution_stack.pop() {
                    self.execution_stack.push(a.clone());
                    self.execution_stack.push(a);
                } else {
                    print_error(
                        format!("not enough arguments for {}", i.value).as_str(),
                        i.line_number,
                        i.row,
                        &self.file,
                        &self.last_instruction,
                    )
                };
            }
            "println" => {
                let mut escape_char = false;
                if let Some(a) = self.execution_stack.pop() {
                    for c in a.value.chars() {
                        if !escape_char && c == '\\' {
                            escape_char = true;
                            continue;
                        }
                        if escape_char {
                            if c == 'n' {
                                println!();
                                escape_char = false;
                                continue;
                            }
                            if c == 't' {
                                print!("\t");
                                escape_char = false;
                                continue;
                            }
                        }
                        print!("{}", c)
                    }
                    println!();
                } else {
                    println!();
                };
            }
            "print" => {
                let mut escape_char = false;
                if let Some(a) = self.execution_stack.pop() {
                    for c in a.value.chars() {
                        if !escape_char && c == '\\' {
                            escape_char = true;
                            continue;
                        }
                        if escape_char {
                            if c == 'n' {
                                println!();
                                escape_char = false;
                                continue;
                            }
                            if c == 't' {
                                print!("\t");
                                escape_char = false;
                                continue;
                            }
                        }
                        print!("{}", c)
                    }
                } else {
                    println!()
                };
            }
            "flush" => {
                std::io::stdout().flush().unwrap();
            }
            "newline" => {}
            _ => {}
        }

        if i.value != "@" {
            self.last_instruction = i.value.to_owned();
        }
    }
}
