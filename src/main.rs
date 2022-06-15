extern crate clap;
extern crate colored;
extern crate dym;
extern crate rustyline; // not needed in Rust 2018

use clap::{App, Arg};
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;
use std::vec;

mod token;
use token::{Token,TokenTypes};

fn print_error(er: &str, line: usize, r: usize, file: &str, last: &str) {
    println!(
        "{}: on line {}, {}",
        "ERROR".red(),
        line,
        &er.bright_yellow()
    );
    if let Ok(lines) = read_lines(file) {
        // Consumes the iterator, returns an (Optional) String
        let mut linenumber = 0;
        for l in lines {
            linenumber += 1;
            if linenumber == line {
                if let Ok(ip) = l {
                    println!("  {}  | {}", ip.bright_blue(), line);
                    for _n in 0..r {
                        print!(" ");
                    }
                    println!("{}", "  ^".bright_yellow())
                }
            }
        }
    } else {
        println!("  {}", file.bright_blue());
        for _n in 0..r {
            print!(" ");
        }
        println!("{}", "  ^".bright_yellow());
    }

    if !last.is_empty() {
        println!(
            "  NOTE: Previous function call {}",
            last.yellow().underline()
        )
    }
}

fn read_lines<P>(filename: P) -> std::io::Result<std::io::Lines<std::io::BufReader<std::fs::File>>>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(filename)?;
    Ok(std::io::BufRead::lines(std::io::BufReader::new(file)))
}

fn is_string_number(data: &str) -> bool {
    let mut deci: bool = false;
    for c in data.chars() {
        //Checks to see if there is more than one period
        if c == '.' && deci {
            return false;
        }
        //Checks to see if it is a number, and makes sure it skips first period
        if !c.is_numeric() && c != '.' {
            return false;
        }
        //Changes deci to true after finding first period
        if c == '.' {
            deci = true
        }
    }
    true
}

struct Lexer {
    source: String,
    buffer: String,
    line_number: usize,
    row: usize,
    is_parsing_stringdq: bool,
    is_parsing_stringsq: bool,
    is_parsing_comment: bool,
    block_stack: Vec<Vec<Token>>,
    function_keywords: Vec<String>,
    bool_keywords: Vec<String>,
}

impl Lexer {
    fn new_from_file(filename: &str) -> Self {
        if let Ok(content) = std::fs::read_to_string(filename) {
            Lexer {
                source: content,
                buffer: String::new(),
                line_number: 1,
                row: 0,
                is_parsing_stringdq: false,
                is_parsing_stringsq: false,
                block_stack: vec![vec![]],
                function_keywords: vec![
                    "print".to_string(),
                    "println".to_string(),
                    "if".to_string(),
                    "call".to_string(),
                    "set".to_string(),
                    "@".to_string(),
                    "dup".to_string(),
                    "concat".to_string(),
                    "var".to_string(),
                    "exit".to_string(),
                ],
                bool_keywords: vec!["true".to_string(), "false".to_string()],
                is_parsing_comment: false,
            }
        } else {
            println!(
                "ERROR: file: {} could not be found. Exiting with error code 1",
                filename
            );
            std::process::exit(1);
        }
    }

    fn new_from_string(input: &str) -> Self {
        Lexer {
            source: input.to_string(),
            buffer: String::new(),
            line_number: 1,
            row: 0,
            is_parsing_stringdq: false,
            is_parsing_stringsq: false,
            block_stack: vec![vec![]],
            function_keywords: vec![
                "print".to_string(),
                "println".to_string(),
                "if".to_string(),
                "println".to_string(),
                "call".to_string(),
                "set".to_string(),
                "@".to_string(),
                "dup".to_string(),
                "concat".to_string(),
                "var".to_string(),
                "exit".to_string(),
            ],
            bool_keywords: vec!["true".to_string(), "false".to_string()],
            is_parsing_comment: false,
        }
    }
    #[allow(dead_code)]
    fn add_input(&mut self, input: &str) {
        self.source.push_str(input)
    }
    #[allow(dead_code)]
    fn clear_lexer(&mut self) {
        self.source.clear()
    }

    fn check_token(&self) -> Option<Token> {
        if !self.buffer.is_empty() {
            if is_string_number(&self.buffer) {
                return Some(Token {
                    token_type: TokenTypes::Number,
                    value: self.buffer.clone(),
                    line_number: self.line_number,
                    row: self.row - self.buffer.len(),
                    block: vec![],
                });
            } else {
                //special identifiers
                if self.function_keywords.contains(&self.buffer) {
                    return Some(Token {
                        token_type: TokenTypes::Function,
                        value: self.buffer.clone(),
                        line_number: self.line_number,
                        row: self.row - self.buffer.len(),
                        block: vec![],
                    });
                }
                if self.bool_keywords.contains(&self.buffer) {
                    return Some(Token {
                        token_type: TokenTypes::Bool,
                        value: self.buffer.clone(),
                        line_number: self.line_number,
                        row: self.row - self.buffer.len(),
                        block: vec![],
                    });
                }
                return Some(Token {
                    token_type: TokenTypes::Identifier,
                    value: self.buffer.clone(),
                    line_number: self.line_number,
                    row: self.row - self.buffer.len(),
                    block: vec![],
                });
            }
        }
        Option::None
    }

    fn parse(&mut self) {
        // todo add escape key
        for c in self.source.chars() {
            if self.is_parsing_stringdq {
                if c != '"' {
                    self.buffer.push(c);
                    continue;
                } else {
                    self.is_parsing_stringdq = false;
                    if let Some(vec_last) = self.block_stack.last_mut() {
                        vec_last.push(Token {
                            token_type: TokenTypes::String,
                            value: self.buffer.clone(),
                            line_number: self.line_number,
                            row: self.row,
                            block: vec![],
                        })
                    }
                    self.row += self.buffer.len() + 1;
                    self.buffer.clear();
                    continue;
                }
            }

            if self.is_parsing_stringsq {
                if c != '\'' {
                    self.buffer.push(c);
                    continue;
                } else {
                    self.is_parsing_stringsq = false;
                    if let Some(vec_last) = self.block_stack.last_mut() {
                        vec_last.push(Token {
                            token_type: TokenTypes::String,
                            value: self.buffer.clone(),
                            line_number: self.line_number,
                            row: self.row,
                            block: vec![],
                        })
                    }
                    self.row += self.buffer.len() + 1;
                    self.buffer.clear();
                    continue;
                }
            }

            if self.is_parsing_comment {
                if c != '\n' {
                    continue;
                } else {
                    self.is_parsing_comment = false;
                    continue;
                }
            }

            match c {
                '\n' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }

                    if let Some(vec_last) = self.block_stack.last_mut() {
                        vec_last.push(Token {
                            token_type: TokenTypes::NewLine,
                            value: "newline".to_string(),
                            line_number: self.line_number,
                            row: self.row,
                            block: vec![],
                        })
                    }

                    self.line_number += 1;
                    self.row = 0;
                    continue;
                }
                '#' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }
                    self.is_parsing_comment = true;
                }
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                    self.buffer.push(c);
                }

                ' ' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }
                }

                '+' | '-' | '*' | '/' | '(' | ')' | '[' | ']' | '<' | '>' | '`' | '~' | '!'
                | '@' | '$' | '%' | '^' | '&' | ',' | '?' | ';' | ':' | '=' | '.' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }

                    if let Some(vec_last) = self.block_stack.last_mut() {
                        vec_last.push(Token {
                            token_type: TokenTypes::Symbol,
                            value: c.to_string(),
                            line_number: self.line_number,
                            row: self.row,
                            block: vec![],
                        })
                    }
                }

                '"' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }
                    self.is_parsing_stringdq = true;
                }
                '\'' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }
                    self.is_parsing_stringsq = true;
                }
                '{' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    }

                    self.block_stack.push(vec![]);
                }
                '}' => {
                    if let Some(t) = self.check_token() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(t)
                        }
                        self.buffer.clear();
                    };

                    if let Some(list) = self.block_stack.pop() {
                        if let Some(vec_last) = self.block_stack.last_mut() {
                            vec_last.push(Token {
                                token_type: TokenTypes::Block,
                                value: "block".to_string(),
                                line_number: self.line_number,
                                row: self.row,
                                block: list,
                            })
                        }
                    }
                }
                _ => println!("ERROR: {} is not defined. Line {1}", c, self.line_number),
            }
            self.row += 1;
        }

        if let Some(t) = self.check_token() {
            if let Some(vec_last) = self.block_stack.last_mut() {
                vec_last.push(t)
            }
            self.buffer.clear();
        };

        // for t in &self.block_stack[0] {
        //     println!(
        //         "token type: {:?} token value: {1} token line number: {2} ",
        //         t.token_type, t.value, t.line_number
        //     )
        // }
    }
}

struct ManitcoreVm {
    instruction_tokens: Vec<Token>,
    execution_stack: Vec<Token>,
    last_instruction: String,
    file: String,
    debug: bool,
    heap: HashMap<String, Token>,
}

impl ManitcoreVm {
    fn new(tokenlist: &[Token], file: &str) -> Self {
        Self {
            instruction_tokens: tokenlist.to_vec(),
            execution_stack: Vec::new(),
            file: file.to_string(),
            last_instruction: String::new(),
            debug: false,
            heap: HashMap::new(),
        }
    }

    fn execute(&mut self) {
        for i in &self.instruction_tokens {
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

            if i.token_type == TokenTypes::Identifier {
                self.execution_stack.push(i.clone());
                continue;
            }

            //Match values
            match i.value.to_lowercase().as_str() {
                "(" => print_error(
                    "Possibly missing ')' pair",
                    i.line_number,
                    i.row,
                    &self.file,
                    &self.last_instruction,
                ),
                "set" => {
                    let mut variable_stack: Vec<String> = Vec::new();
                    while let Some(k) = self.execution_stack.last() {
                        if k.token_type == TokenTypes::Identifier {
                            if let Some(tok) = self.execution_stack.pop() {
                                variable_stack.push(tok.value.clone());
                            }
                        } else {
                            break;
                        }
                    }
                    for values in variable_stack {
                        if let Some(tok) = self.execution_stack.pop() {
                            self.heap.insert(values.clone(), tok.clone());
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
                "." => {
                    if let Some(a) = self.execution_stack.pop() {
                        if self.heap.contains_key(&a.value) {
                            if let Some(t) = self.heap.get(&a.value) {
                                let mut parser = Parser::new();
                                if self.debug {
                                    parser.debug = true;
                                }
                                let shunted = parser.shunt(&t.block).clone();
                                let mut vm = ManitcoreVm::new(&shunted, &t.value);
                                if self.debug {
                                    vm.debug = true;
                                }
                                vm.execution_stack = self.execution_stack.clone();
                                vm.heap = self.heap.clone();
                                vm.execute();
                                if let Some(t) = vm.execution_stack.pop() {
                                    self.execution_stack.push(t)
                                }
                            }
                        } else {
                            let mut parser = Parser::new();
                            if self.debug {
                                parser.debug = true;
                            }
                            let shunted = parser.shunt(&a.block).clone();
                            let mut vm = ManitcoreVm::new(&shunted, &a.value);
                            if self.debug {
                                vm.debug = true;
                            }
                            vm.execution_stack = self.execution_stack.clone();
                            vm.heap = self.heap.clone();
                            vm.execute();
                            if let Some(t) = vm.execution_stack.pop() {
                                self.execution_stack.push(t)
                            }
                        }
                    } else {
                        print_error(
                            "not enough arguments for .",
                            i.line_number,
                            i.row,
                            &self.file,
                            &self.last_instruction,
                        )
                    };
                }
                "@" => {
                    if let Some(a) = self.execution_stack.pop() {
                        if self.heap.contains_key(&a.value) {
                            if let Some(t) = self.heap.get(&a.value) {
                                let mut parser = Parser::new();
                                if self.debug {
                                    parser.debug = true;
                                }
                                let shunted = parser.shunt(&t.block).clone();
                                let mut vm = ManitcoreVm::new(&shunted, &t.value);
                                if self.debug {
                                    vm.debug = true;
                                }
                                vm.execution_stack = self.execution_stack.clone();
                                vm.heap = self.heap.clone();
                                vm.execute();
                                self.execution_stack = vm.execution_stack.clone();
                                if let Some(t) = vm.execution_stack.pop() {
                                    self.execution_stack.push(t)
                                }
                            }
                        } else {
                            let mut parser = Parser::new();
                            if self.debug {
                                parser.debug = true;
                            }
                            let shunted = parser.shunt(&a.block).clone();
                            let mut vm = ManitcoreVm::new(&shunted, &a.value);
                            if self.debug {
                                vm.debug = true;
                            }
                            vm.execution_stack = self.execution_stack.clone();
                            vm.heap = self.heap.clone();
                            vm.execute();
                            self.execution_stack = vm.execution_stack.clone();
                            if let Some(t) = vm.execution_stack.pop() {
                                self.execution_stack.push(t)
                            }
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
                "if" => {
                    if let Some(a) = self.execution_stack.pop() {
                        if let Some(b) = self.execution_stack.pop() {
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
                "var" => {
                    if let (Some(a), Some(b)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
                    {
                        self.heap.insert(b.value, a.clone());
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
                "=" => {
                    if let (Some(a), Some(b)) =
                        (self.execution_stack.pop(), self.execution_stack.pop())
                    {
                        let mut f: String = String::new();
                        let mut s: String = String::new();

                        if self.heap.contains_key(&a.value) {
                            if let Some(tok) = self.heap.get(&a.value) {
                                if let Ok(v) = tok.value.parse() {
                                    f = v
                                } else {
                                    print_error(
                                        "expected a number from variable",
                                        i.line_number,
                                        i.row,
                                        &self.file,
                                        &self.last_instruction,
                                    )
                                }
                            }
                        } else if let Ok(v) = a.value.parse() {
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

                        if self.heap.contains_key(&b.value) {
                            if let Some(tok) = self.heap.get(&b.value) {
                                if let Ok(v) = tok.value.parse() {
                                    s = v
                                } else {
                                    print_error(
                                        "expected a token from variable",
                                        i.line_number,
                                        i.row,
                                        &self.file,
                                        &self.last_instruction,
                                    )
                                }
                            }
                        } else if let Ok(v) = b.value.parse() {
                            s = v
                        } else {
                            print_error(
                                "expected a token",
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

                        if self.heap.contains_key(&a.value) {
                            if let Some(tok) = self.heap.get(&a.value) {
                                if let Ok(v) = tok.value.parse() {
                                    f = v
                                } else {
                                    print_error(
                                        "expected a number from variable",
                                        i.line_number,
                                        i.row,
                                        &self.file,
                                        &self.last_instruction,
                                    )
                                }
                            }
                        } else if let Ok(v) = a.value.parse() {
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

                        if self.heap.contains_key(&b.value) {
                            if let Some(tok) = self.heap.get(&b.value) {
                                if let Ok(v) = tok.value.parse() {
                                    s = v
                                } else {
                                    print_error(
                                        "expected a number from variable",
                                        i.line_number,
                                        i.row,
                                        &self.file,
                                        &self.last_instruction,
                                    )
                                }
                            }
                        } else if let Ok(v) = b.value.parse() {
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

                        if self.heap.contains_key(&a.value) {
                            if let Some(tok) = self.heap.get(&a.value) {
                                if let Ok(v) = tok.value.parse() {
                                    f = v
                                } else {
                                    print_error(
                                        "expected a number from variable",
                                        i.line_number,
                                        i.row,
                                        &self.file,
                                        &self.last_instruction,
                                    )
                                }
                            }
                        } else if let Ok(v) = a.value.parse() {
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

                        if self.heap.contains_key(&b.value) {
                            if let Some(tok) = self.heap.get(&b.value) {
                                if let Ok(v) = tok.value.parse() {
                                    s = v
                                } else {
                                    print_error(
                                        "expected a number from variable",
                                        i.line_number,
                                        i.row,
                                        &self.file,
                                        &self.last_instruction,
                                    )
                                }
                            }
                        } else if let Ok(v) = b.value.parse() {
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

                        if self.heap.contains_key(&a.value) {
                            if let Some(tok) = self.heap.get(&a.value) {
                                if let Ok(v) = tok.value.parse() {
                                    f = v
                                } else {
                                    print_error(
                                        "expected a number from variable",
                                        i.line_number,
                                        i.row,
                                        &self.file,
                                        &self.last_instruction,
                                    )
                                }
                            }
                        } else if let Ok(v) = a.value.parse() {
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

                        if self.heap.contains_key(&b.value) {
                            if let Some(tok) = self.heap.get(&b.value) {
                                if let Ok(v) = tok.value.parse() {
                                    s = v
                                } else {
                                    print_error(
                                        "expected a number from variable",
                                        i.line_number,
                                        i.row,
                                        &self.file,
                                        &self.last_instruction,
                                    )
                                }
                            }
                        } else if let Ok(v) = b.value.parse() {
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

                        if self.heap.contains_key(&a.value) {
                            if let Some(tok) = self.heap.get(&a.value) {
                                if let Ok(v) = tok.value.parse() {
                                    f = v
                                } else {
                                    print_error(
                                        "expected a number from variable",
                                        i.line_number,
                                        i.row,
                                        &self.file,
                                        &self.last_instruction,
                                    )
                                }
                            }
                        } else if let Ok(v) = a.value.parse() {
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

                        if self.heap.contains_key(&b.value) {
                            if let Some(tok) = self.heap.get(&b.value) {
                                if let Ok(v) = tok.value.parse() {
                                    s = v
                                } else {
                                    print_error(
                                        "expected a number from variable",
                                        i.line_number,
                                        i.row,
                                        &self.file,
                                        &self.last_instruction,
                                    )
                                }
                            }
                        } else if let Ok(v) = b.value.parse() {
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

                        if self.heap.contains_key(&a.value) {
                            if let Some(tok) = self.heap.get(&a.value) {
                                if let Ok(v) = tok.value.parse() {
                                    f = v
                                } else {
                                    print_error(
                                        "expected a number from variable",
                                        i.line_number,
                                        i.row,
                                        &self.file,
                                        &self.last_instruction,
                                    )
                                }
                            }
                        } else if let Ok(v) = a.value.parse() {
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

                        if self.heap.contains_key(&b.value) {
                            if let Some(tok) = self.heap.get(&b.value) {
                                if let Ok(v) = tok.value.parse() {
                                    s = v
                                } else {
                                    print_error(
                                        "expected a string from variable",
                                        i.line_number,
                                        i.row,
                                        &self.file,
                                        &self.last_instruction,
                                    )
                                }
                            }
                        } else if let Ok(v) = b.value.parse() {
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
                        if self.heap.contains_key(&a.value) {
                            if let Some(tok) = self.heap.get(&a.value) {
                                for c in tok.value.chars() {
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
                            }
                            println!();
                        } else {
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
                "print" => {
                    let mut escape_char = false;
                    if let Some(a) = self.execution_stack.pop() {
                        if self.heap.contains_key(&a.value) {
                            if let Some(tok) = self.heap.get(&a.value) {
                                for c in tok.value.chars() {
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
                            }
                        } else {
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
                _ => {

                    // println!(
                    //     "{}:instruction {} is not implemented.",
                    //     "ERROR".red(),
                    //     i.value.yellow()
                    // );

                    // let mut functions = Lexicon::new();
                    // functions.insert("print");
                    // functions.insert("call");
                    // functions.insert("if");
                    // functions.insert("dup");
                    // functions.insert("concat");
                    // let corrections = functions.corrections_for(&i.value);
                    // if !corrections.is_empty() {
                    //     println!("  NOTE: Did you mean:");
                    //     for c in corrections {
                    //         println!("    {}", c.bright_blue());
                    //     }
                    // }
                }
            }
            self.last_instruction = i.value.to_owned();
        }
        if self.debug {
            for (k, v) in &self.heap {
                println!("{} -> {} ", k, v.value)
            }
            println!("--- latest ---")
        }
    }
}

struct Parser {
    //input_stack: Vec<Token>,
    operator_stack: Vec<Token>,
    output_stack: Vec<Token>,
    debug: bool,
}

impl Parser {
    fn new() -> Self {
        Parser {
            operator_stack: Vec::new(),
            output_stack: Vec::new(),
            debug: false,
        }
    }

    fn clear(&mut self) {
        self.operator_stack.clear();
        self.output_stack.clear();
    }

    fn shunt(&mut self, input: &[Token]) -> &Vec<Token> {
        for token in input {
            if token.token_type == TokenTypes::Number {
                self.output_stack.push(token.clone());
            }
            if token.token_type == TokenTypes::Block {
                self.output_stack.push(token.clone());
                if let Some(t) = self.operator_stack.last() {
                    if t.value == "@" {
                        if let Some(tok) = self.operator_stack.pop() {
                            self.output_stack.push(tok)
                        }
                    }
                }
                if let Some(t) = self.operator_stack.last() {
                    if t.value == "." {
                        if let Some(tok) = self.operator_stack.pop() {
                            self.output_stack.push(tok)
                        }
                    }
                }
            }
            if token.token_type == TokenTypes::String {
                self.output_stack.push(token.clone());
            }
            if token.token_type == TokenTypes::Bool {
                self.output_stack.push(token.clone());
            }
            if token.token_type == TokenTypes::Function{
                self.operator_stack.push(token.clone());
            }
            //consider using a flag or option for repl
            if token.token_type == TokenTypes::Identifier {
                if token.value.as_str() == "debug" {
                    self.debug = true;
                    continue;
                }
                self.operator_stack.push(token.clone());
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
                                    self.output_stack.push(t.clone())
                                }
                            }

                            if !self.operator_stack.is_empty()
                                && self.operator_stack.last().unwrap().token_type
                                    == TokenTypes::Identifier
                            {
                                if let Some(t) = self.operator_stack.pop() {
                                    self.output_stack.push(t.clone())
                                }

                                self.output_stack.push(Token {
                                    token_type: TokenTypes::Symbol,
                                    value: "@".to_string(),
                                    line_number: 0,
                                    row: 0,
                                    block: vec![],
                                })
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
                    ":" => {
                        if let Some(t) = self.operator_stack.pop() {
                            self.output_stack.push(t.clone())
                        }
                    }
                    "=" => self.operator_stack.push(token.clone()),
                    "@" => self.operator_stack.push(token.clone()),
                    "." => self.operator_stack.push(token.clone()),
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

fn main() {
    let matches = App::new("Manticore Parser")
        .version("0.1")
        .author("Pyrotek45 pyrotek45_gaming@yahoo.com")
        .about("Manticore VM")
        .arg(
            Arg::with_name("FILE")
                .value_name("FILE")
                .help("Sets the input file to be used")
                .index(1),
        )
        .arg(
            Arg::with_name("DEBUG")
                .value_name("DEBUG")
                .long("debug")
                .takes_value(false)
                .short('d')
                .help("displays debug information"),
        )
        .get_matches();

    if let Some(filename) = matches.value_of("FILE") {
        //Get filename from argument
        let mut lexer = Lexer::new_from_file(filename);

        //Parse the file into tokens
        lexer.parse();
        let mut parser = Parser::new();
        if matches.is_present("DEBUG") {
            parser.debug = true;
        }
        let shunted = parser.shunt(&lexer.block_stack[0]).clone();
        let mut vm = ManitcoreVm::new(&shunted, filename);
        if matches.is_present("DEBUG") {
            vm.debug = true;
        }
        vm.execute();
        std::process::exit(0)
        // for t in vm.instruction_tokens {
        //     println!("{}", t.value)
        // }
    } else {
        let mut rl = Editor::<()>::new();
        if rl.load_history("history.txt").is_err() {
            println!("No previous history.");
        }

        let mut parser = Parser::new();
        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str());
                    let mut lexer = Lexer::new_from_string(&line);
                    lexer.parse();
                    parser.clear();
                    if line.to_lowercase() == "exit" {break};
                    if line == "debugmode" {
                        parser.debug = !parser.debug;
                        println!("DEBUGMODE {}", parser.debug);
                    } else {
                        let shunted = parser.shunt(&lexer.block_stack[0]).clone();
                        let mut vm = ManitcoreVm::new(&shunted, &line);
                        if parser.debug {
                            vm.debug = true;
                        }
                        vm.execute();
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    break;
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        rl.save_history("history.txt").unwrap();
    }
}
