use colored::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

pub mod ast;
pub mod core;
pub mod interpreter;
pub mod lexer;
pub mod ministers;
pub mod parser;
pub mod token;

use crate::core::types::SuperValue;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;

#[tokio::main]
async fn main() {
    // Setup high-performance tracing for the "Auditoria Real"
    tracing_subscriber::fmt::init();

    // Argument processing for file execution
    let args: Vec<String> = std::env::args().collect();

    // Initialize integration engines
    let _ = ministers::python_bridge::init_python_engine();
    let _ = ministers::js_bridge::init_js_engine();
    let _ = ministers::java_bridge::init_java_engine();
    let _ = ministers::c_bridge::init_c_engine();

    let mut interpreter = Interpreter::new();

    if args.len() > 1 {
        let path = &args[1];
        let content = std::fs::read_to_string(path).expect("Could not read file");

        let lexer = Lexer::new(&content);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);

        match parser.parse() {
            Ok(program) => {
                match interpreter.eval_program(program) {
                    Ok(result) => {
                        if result != SuperValue::Void {
                             println!("{}", result);
                        }
                    }
                    Err(e) => {
                        println!("{}", e.to_string().red().bold());
                    }
                }
            }
            Err(e) => {
                println!("{} {}", "Syntax Error:".red().bold(), e);
            }
        }
        return;
    }

    println!("{}", "Welcome to Super (SPL) REPL!".green().bold());
    println!("{}", "Type 'exit' or 'quit' to leave.".yellow());

    let mut rl = DefaultEditor::new().unwrap();
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

                let lexer = Lexer::new(&buffer);
                let tokens = lexer.tokenize();
                let mut parser = Parser::new(tokens);

                match parser.parse() {
                    Ok(program) => {
                        match interpreter.eval_program(program) {
                            Ok(result) => {
                                if result != SuperValue::Void {
                                     println!("{}", result);
                                }
                            }
                            Err(e) => {
                                println!("{}", e.to_string().red().bold());
                            }
                        }
                        buffer.clear();
                    }
                    Err(e) => {
                        if e.contains("EOF") || e.contains("Expected") {
                            // Incomplete, wait for more
                        } else {
                            println!("{} {}", "Syntax Error:".red().bold(), e);
                            buffer.clear();
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
