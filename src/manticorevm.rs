use std::collections::HashMap;

use crate::{
    parser::Parser,
    string_utils::print_error,
    token::{Token, TokenTypes},
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
        for i in &self.instruction_tokens {
            // If token is an identifier and its value is found on the heap
            // push the heap value instead
            if i.token_type == TokenTypes::Identifier {
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
                continue;
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
                            "not enough arguments for +",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    };
                }
                // Clears out the heap
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

                        for _x in 0..vm.stack_set {
                            if let Some(_t) = self.execution_stack.pop() {}
                        }

                        // Copy the last item to return from inside the vm to the outside
                        if let Some(return_value) = vm.execution_stack.pop() {
                            self.execution_stack.push(return_value)
                        }
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
                            token_type: TokenTypes::Knot,
                            value: a.value.clone(),
                            line_number: 0,
                            row: 0,
                            block: new_stack.clone(),
                            proxy: a.proxy.clone(),
                        });
                    } else {
                        print_error(
                            "not enough arguments for mod",
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
                "tie" => {
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
                        print_error(
                            "not enough arguments for print",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
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
                        print_error(
                            "not enough arguments for print",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
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
        }
    }
}
