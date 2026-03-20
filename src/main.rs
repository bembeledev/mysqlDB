pub mod ast;
pub mod core;
pub mod interpreter;
pub mod lexer;
pub mod ministers;
pub mod parser;
pub mod token;

use colored::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::core::types::SuperValue;

fn main() {
    println!("{}", "Welcome to Super (SPL) REPL!".green().bold());
    println!("{}", "Type 'exit' or 'quit' to leave.".yellow());

    // Initialize integration engines if we want
    let _ = ministers::python_bridge::init_python_engine();
    let _ = ministers::js_bridge::init_js_engine();

    let mut rl = DefaultEditor::new().unwrap();
    let mut interpreter = Interpreter::new();

    // Very simple multiline string builder for the REPL
    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() { "spl> " } else { "...> " };
        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() && buffer.is_empty() {
                    continue;
                }
                if trimmed == "exit" || trimmed == "quit" {
                    break;
                }

                let _ = rl.add_history_entry(line.as_str());
                buffer.push_str(&line);
                buffer.push('\n');

                // Determine if we should attempt parsing.
                // A very basic check: does it end with a block or semicolon?
                // This is a rough heuristic to make the interactive REPL friendlier.
                // In a robust implementation, the parser would return an `Incomplete` error to trigger more input.
                // We'll just try to parse, and if it fails with EOF expectations, we accumulate.

                let lexer = Lexer::new(&buffer);
                let tokens = lexer.tokenize();
                let mut parser = Parser::new(tokens);

                match parser.parse() {
                    Ok(program) => {
                        // Persist global state correctly
                        match interpreter.eval_program(program) {
                            Ok(result) => {
                                if result != SuperValue::Void {
                                     println!("{:?}", result);
                                }
                            }
                            Err(e) => {
                                println!("{} {}", "Runtime Error:".red().bold(), e);
                            }
                        }
                        buffer.clear(); // executed successfully or handled runtime error
                    }
                    Err(e) => {
                        if e.contains("found EOF") || e.contains("Expected") {
                            // Likely an incomplete line, keep buffering
                        } else {
                            println!("{} {}", "Syntax Error:".red().bold(), e);
                            buffer.clear(); // Unrecoverable syntax error
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
