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

// ... (teus imports continuam iguais)

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // 🎯 Movemos a inicialização para fora para ser partilhada
    let mut interpreter = Interpreter::new();
    let args: Vec<String> = std::env::args().collect();

    // === MODO EXECUÇÃO DE FICHEIRO ===
    if args.len() > 1 {
        let path = &args[1];
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let lexer = Lexer::new(&content);
                let tokens = lexer.tokenize();
                let mut parser = Parser::new(tokens);

                match parser.parse() {
                    Ok(program) => {
                        if let Err(e) = interpreter.eval_program(program) {
                            // 🎯 Erro com cor para destacar no Stress Test
                            println!("{} {}", "Runtime Error:".red().bold(), e);
                        }
                    }
                    Err(e) => println!("{} {}", "Syntax Error:".red().bold(), e),
                }
            }
            Err(e) => println!("{} Erro ao ler {}: {}", "IO Error:".red().bold(), path, e),
        }
        return; // Finaliza aqui se for um ficheiro
    }

    // === MODO REPL (INTERATIVO) ===
    println!("{}", "Welcome to Super (SPL) REPL!".green().bold());
    println!("{}", "Type 'exit' or 'quit' to leave.".yellow());

    let mut rl = DefaultEditor::new().unwrap();
    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() { "spl> " } else { "...> " };
        match rl.readline(prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() && buffer.is_empty() { continue; }
                if trimmed == "exit" || trimmed == "quit" { break; }

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
                                    // 🎯 Usamos o Display (sem {:?}) para um output limpo
                                    println!("{}", result); 
                                }
                            }
                            Err(e) => println!("{} {}", "Runtime Error:".red().bold(), e),
                        }
                        buffer.clear(); 
                    }
                    Err(e) => {
                        // Se for erro de fim de ficheiro inesperado, continuamos a pedir input
                        if e.contains("found EOF") || e.contains("Expected") {
                            continue;
                        } else {
                            println!("{} {}", "Syntax Error:".red().bold(), e);
                            buffer.clear();
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => { println!("Error: {:?}", err); break; }
        }
    }
}