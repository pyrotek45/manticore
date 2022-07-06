use std::collections::HashMap;
use std::process::Command;

use crate::{
    lexer::{self, Lexer},
    parser::Parser,
    string_utils::print_error,
    token::{self, Token, TokenTypes},
};

pub struct ManitcoreVm {
    instruction_tokens: Vec<Token>,
    execution_stack: Vec<Token>,
    last_instruction: String,
    stack_set: usize,
    file: String,
    pub debug: bool,
    heap: HashMap<String, Token>,
}

impl ManitcoreVm {
    // fn parse_chain(chain: String) -> Token {

    //     let mut buffer: String;
    //     let mut list: Vec<String>;

    //     //split by dots
    //     for c in chain.chars() {
    //         if c == '.' {
    //             list.push(buffer.clone());
    //             buffer.clear()
    //         } else {
    //             buffer += &c.to_string();
    //         }
    //     }

    //     // reverse list x y z -> z y x
    //     list.reverse();

    //     // recursive function, give it a list of strings
    //     // and it returns the token

    //     Token { proxy: (), token_type: (), value: (), block: (), line_number: (), row: () }

    // }

    pub fn new(tokenlist: &[Token], file: &str) -> Self {
        Self {
            instruction_tokens: tokenlist.to_vec(),
            execution_stack: Vec::new(),
            file: file.to_string(),
            last_instruction: String::new(),
            debug: false,
            heap: HashMap::new(),
            stack_set: 0,
        }
    }

    // Proccess each token
    pub fn execute(&mut self) {
        let mut core_self: Vec<Token> = vec![];
        for i in &self.instruction_tokens {
            // If token is an identifier and its value is found on the heap
            // push the heap value instead
            if i.token_type == TokenTypes::Identifier {
                if i.value == "self" {
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

                    self.execution_stack.push(Token {
                        proxy: None,
                        token_type: TokenTypes::Block,
                        value: "self".to_string(),
                        block: core_self.clone(),
                        line_number: 0,
                        row: 0,
                    });
                    continue;
                }

                if let Some(tok) = self.heap.get(&i.value) {
                    // Push token from heap , change type to knot
                    self.execution_stack.push(Token {
                        token_type: TokenTypes::Knot,
                        value: tok.value.clone(),
                        line_number: 0,
                        row: 0,
                        block: tok.block.clone(),
                        proxy: tok.proxy.clone(),
                    });
                    continue;
                }
                self.execution_stack.push(i.clone());
                //continue;
            }

            // Strings , blocks, list, numbers and bools get pushed
            // onto the execution stack automatically
            if i.token_type == TokenTypes::String {
                self.execution_stack.push(i.clone());
                continue;
            }

            if i.token_type == TokenTypes::Block {
                self.execution_stack.push(i.clone());
                continue;
            }

            if i.token_type == TokenTypes::Number {
                self.execution_stack.push(i.clone());
                continue;
            }
            if i.token_type == TokenTypes::Bool {
                self.execution_stack.push(i.clone());
                continue;
            }
            if i.token_type == TokenTypes::Break {
                self.execution_stack.push(i.clone());
                continue;
            }
            if i.token_type == TokenTypes::List {
                self.execution_stack.push(i.clone());
                continue;
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
                "end" =>{
                    break;
                },
                "command" => {
                    if let (Some(a), Some(b)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
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
                            "not enough arguments for import",
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
                            "not enough arguments for neg",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    };
                }
                "." => {
                    if let (Some(id), Some(block)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
                    {
                        let mut parser = Parser::new();
                        if self.debug {
                            parser.debug = true
                        }
                        let shunted = parser.shunt(&block.block).clone();
                        let mut vm = ManitcoreVm::new(&shunted, &block.value);
                        if self.debug {
                            vm.debug = true
                        }
                        //vm.heap = self.heap.clone();
                        vm.execute();

                        if let Some(name) = id.proxy {
                            if let Some(tok) = vm.heap.get(&name) {
                                // Push token from heap , change type to knot
                                self.execution_stack.push(tok.clone())
                            }
                        } else if let Some(tok) = vm.heap.get(&id.value) {
                            // Push token from heap , change type to knot
                            self.execution_stack.push(tok.clone())
                        }
                    }
                }
                "loop" => {
                    if let (Some(block), Some(num)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
                    {
                        if let Ok(v) = num.value.parse() {
                            for _ in 1..=v {
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
                                vm.execute();
                                self.heap = vm.heap.clone();
                            }
                        } else {
                            print_error(
                                "expected a number",
                                i.line_number,
                                i.row,
                                &self.file,
                                &self.last_instruction,
                            )
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
                    }
                }
                "shc" => self.heap.clear(),
                "pop" => if let Some(_t) = self.execution_stack.pop() {},
                // Used to tie more than 1 token at a time from the stack
                "set" => {
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
                        self.stack_set += 1;
                        if let Some(mut tok) = self.execution_stack.pop() {
                            tok.proxy = Some(values.clone());
                            self.heap.insert(values, tok.clone());
                        } else {
                            print_error(
                                "not enough arguments for set",
                                i.line_number,
                                i.row,
                                &self.file,
                                &self.last_instruction,
                            )
                        }
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
                            "not enough arguments for call",
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
                        let mut new_stack = vec![];

                        for t in a.block {
                            if let Some(tok) = self.heap.get(&t.value) {
                                new_stack.push(tok.clone())
                            } else {
                                new_stack.push(t)
                            }
                        }

                        self.execution_stack.push(Token {
                            token_type: TokenTypes::Block,
                            value: a.value.clone(),
                            line_number: 0,
                            row: 0,
                            block: new_stack.clone(),
                            proxy: a.proxy.clone(),
                        });
                    } else {
                        print_error(
                            "not enough arguments for let",
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
                            "not enough arguments for @",
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
                            "not enough arguments for ret",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    };
                }
                "if" => {
                    if let (Some(a), Some(b)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
                    {
                        //if true single if statement
                        if b.token_type == TokenTypes::Bool {
                            if b.value == "true" {
                                let mut parser = Parser::new();
                                if self.debug {
                                    parser.debug = true
                                }
                                let shunted = parser.shunt(&a.block).clone();
                                let mut vm = ManitcoreVm::new(&shunted, &a.value);
                                if self.debug {
                                    vm.debug = true
                                }
                                vm.heap = self.heap.clone();
                                vm.execute();
                                self.heap = vm.heap.clone();
                                                        // Copy the last item to return from inside the vm to the outside
                                if let Some(return_value) = vm.execution_stack.pop() {
                                    self.execution_stack.push(return_value)
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
                                    let mut vm = ManitcoreVm::new(&shunted, &b.value);
                                    if self.debug {
                                        vm.debug = true
                                    }
                                    vm.heap = self.heap.clone();
                                    vm.execute();
                                    self.heap = vm.heap.clone();
                                    if let Some(return_value) = vm.execution_stack.pop() {
                                        self.execution_stack.push(return_value)
                                    }
                                } else {
                                    let mut parser = Parser::new();
                                    if self.debug {
                                        parser.debug = true
                                    }
                                    let shunted = parser.shunt(&a.block).clone();
                                    let mut vm = ManitcoreVm::new(&shunted, &a.value);
                                    if self.debug {
                                        vm.debug = true
                                    }
                                    vm.heap = self.heap.clone();
                                    vm.execute();
                                    self.heap = vm.heap.clone();
                                    if let Some(return_value) = vm.execution_stack.pop() {
                                        self.execution_stack.push(return_value)
                                    }
                                }
                            }
                        } else {
                            print_error(
                                "not enough arguments for if",
                                i.line_number,
                                i.row,
                                &self.file,
                                &self.last_instruction,
                            )
                        }
                    } else {
                        print_error(
                            "not enough arguments for if",
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
                "var" | "is" => {
                    if let (Some(mut a), Some(b)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
                    {
                        if b.token_type == TokenTypes::Identifier || b.proxy.is_some() {
                            if let Some(p) = b.proxy {
                                a.proxy = Some(p.clone());
                                self.heap.insert(p, a.clone());
                            } else {
                                a.proxy = Some(b.value.clone());
                                self.heap.insert(b.value, a.clone());
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
                            "not enough arguments for var",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    };
                }
                "<" => {
                    // todo: does not support blocks atm
                    if let (Some(a), Some(b)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
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
                            "not enough arguments for <",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    };
                }
                ">" => {
                    // todo: does not support blocks atm
                    if let (Some(a), Some(b)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
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
                            "not enough arguments for >",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    };
                }
                "=" => {
                    // todo: does not support blocks atm
                    if let (Some(a), Some(b)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
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
                            "not enough arguments for =",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    };
                }
                "+" => {
                    if let (Some(a), Some(b)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
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
                            "not enough arguments for +",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    };
                }
                "-" => {
                    if let (Some(a), Some(b)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
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
                            "not enough arguments for -",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    };
                }
                "*" => {
                    if let (Some(a), Some(b)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
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
                            "not enough arguments for *",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    };
                }
                "/" => {
                    if let (Some(a), Some(b)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
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
                            "not enough arguments for /",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    };
                }
                "concat" => {
                    if let (Some(a), Some(b)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
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
                            "not enough arguments for concat",
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
                            "not enough arguments for dup",
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
                "newline" => {}
                _ => {}
            }

            self.last_instruction = i.value.to_owned();
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

        // for t in core_self {
        //     println!("{} in core ", t.value)
        // }
    }
}
