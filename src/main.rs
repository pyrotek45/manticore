use std::rc::Rc;
use std::time::{Duration, Instant};
extern crate clap;
extern crate colored;

mod lexer;
mod manticorevm;
mod parser;
mod string_utils;
mod token;
mod unit_test;

use clap::*;
use colored::Colorize;
use manticorevm::ManitcoreVm;
use parser::Parser;

use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};

use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, EventHandler, KeyCode, KeyEvent, Modifiers};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter, Validator};

#[derive(Completer, Helper, Highlighter, Hinter, Validator)]
struct InputValidator {
    #[rustyline(Validator)]
    brackets: MatchingBracketValidator,
}

fn main() {
    // Clap setup
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
        .arg(
            Arg::with_name("TIME")
                .value_name("TIME")
                .long("time")
                .takes_value(false)
                .short('t')
                .help("displays how long manticore takes to run"),
        )
        .get_matches();

    //used for bundling code and interpreter to create single file
    // let std = include_str!("../std.core");
    // let program = include_str!("../test7.core");
    // if !program.is_empty() {
    //     let mut lexer = lexer::Lexer::new_from_string(std);
    //     lexer.add_input(program);
    //     // Parse the file into tokens
    //     lexer.parse();
    //     let mut parser = Parser::new();
    //     if matches.is_present("DEBUG") {
    //         parser.debug = true;
    //     }

    //     // Store now parsed tokens into a new list
    //     let shunted = parser.shunt(&lexer.block_stack[0]).clone();
    //     let mut vm = ManitcoreVm::new(&shunted, "");
    //     if matches.is_present("DEBUG") {
    //         vm.debug = true;
    //     }

    //     // Execute the vm using parsed token list
    //     vm.execute();
    //     std::process::exit(0)
    // }

    // Repl or File
    if let Some(filename) = matches.value_of("FILE") {
        // Get filename from argument
        let start = Instant::now();
        let mut lexer = lexer::Lexer::new_from_file(filename);

        // Parse the file into tokens
        lexer.parse();
        let mut parser = Parser::new();

        // Store now parsed tokens into a new list
        let shunted = parser.shunt(&lexer.block_stack[0]);
        if matches.is_present("DEBUG") {
            parser.debug_output(0, Rc::new(lexer.block_stack[0].clone()));
        }
        if matches.is_present("TIME") {
            let duration = start.elapsed();
            println!("{} {:?}", ">> Lexing:".bright_green(), duration);
        }

        if matches.is_present("DEBUG") {
            parser.debug_output(0, Rc::new(parser.output_stack.clone()));
        }
        if matches.is_present("TIME") {
            let duration = start.elapsed();
            println!("{} {:?}", ">> Parsing:".bright_green(), duration);
        }
        let mut vm = ManitcoreVm::new(&shunted, filename);
        if !matches.is_present("DEBUG") {
            // Execute the vm using parsed token list
            vm.execute();
        }



        if matches.is_present("TIME") {
            let duration = start.elapsed();
            println!("{} {:?}", ">> Execution:".bright_green(), duration);
        }
        std::process::exit(0)
    } else {
        // Using Repl
        let h = InputValidator {
            brackets: MatchingBracketValidator::new(),
        };
        let mut rl = Editor::new().unwrap();
        rl.set_helper(Some(h));
        rl.bind_sequence(
            KeyEvent(KeyCode::Char('s'), Modifiers::CTRL),
            EventHandler::Simple(Cmd::Newline),
        );
        if rl.load_history("history.txt").is_err() {
            println!("No previous history.");
        }

        let mut repl = String::new();
        let mut repl_debug: bool = false;
        let mut vm = ManitcoreVm::new(&[], "");
        let mut parser = Parser::new();
        loop {
            // Repl prompt
            let readline = rl.readline("Manticore REPL :: $ ");
            match readline {
                Ok(line) => {
                    // Rustlyline History support
                    rl.add_history_entry(line.as_str());
                    rl.save_history("history.txt").unwrap();
                    //repl.push_str(&(" ".to_owned() + &line));

                    // Create new parsing and lexing engine

                    let mut lexer = lexer::Lexer::new_from_string(&line);
                    lexer.parse();

                    // Basic repl commands to check
                    if line.to_lowercase() == "exit" {
                        break;
                    };

                    if line.to_lowercase() == "clear" {
                        repl.clear();
                        continue;
                    };

                    if line.to_lowercase() == "debug" {
                        repl_debug = !repl_debug;
                        parser.debug = repl_debug;
                        continue;
                    };

                    // Shunt tokens and insert them into vm
                    parser._clear();
                    let shunted = parser.shunt(&lexer.block_stack[0]);

                    // Enable vm debug
                    if repl_debug {
                        vm.debug = true;
                    }

                    for i in shunted {
                        vm.execute_token(&i);
                        if vm.exit_loop {
                            break;
                        }
                    }

                    if let Some(tok) = vm.get_tokenifvar() {
                        println!(
                            " ---> [{}] : ({})",
                            tok.get_value_as_string(),
                            tok.get_type_as_string()
                        )
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
    }
}
