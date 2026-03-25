pub mod ast;
pub mod core;
pub mod interpreter;
pub mod lexer;
pub mod ministers;
pub mod parser;
pub mod token;

use colored::*;
/*

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::core::types::SuperValue;


#[tokio::main]
async fn main() {
    // Setup high-performance tracing for the "Auditoria Real"
    tracing_subscriber::fmt::init();

    println!("{}", "Welcome to Super (SPL) REPL!".green().bold());
    println!("{}", "Type 'exit' or 'quit' to leave.".yellow());

    // Initialize integration engines if we want
    let _ = ministers::python_bridge::init_python_engine();
    let _ = ministers::js_bridge::init_js_engine();
    let _ = ministers::java_bridge::init_java_engine();
    let _ = ministers::c_bridge::init_c_engine();

    let mut rl = DefaultEditor::new().unwrap();
    let mut interpreter = Interpreter::new();

    // Argument processing for file execution
    let args: Vec<String> = std::env::args().collect();
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
                             println!("{:?}", result);
                        }
                    }
                    Err(e) => {
                        println!("{} {}", "Runtime Error:".red().bold(), e);
                    }
                }
            }
            Err(e) => {
                println!("{} {}", "Syntax Error:".red().bold(), e);
            }
        }
        return;
    }

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
*/

// Agora os imports vão funcionar
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::{Token, TokenInfo};

fn main() {
    let codigo = std::fs::read_to_string("caos.super").expect("Erro ao ler arquivo");

    println!("{}", "--- [DEBUG: INICIANDO LEXER] ---".yellow().bold());

    let mut lexer = Lexer::new(&codigo);
    // 🎯 Use o nome 'tokens' para ser consistente com o Parser::new(tokens)
    let mut tokens = Vec::new();

    let mut token: TokenInfo;

    loop {
        // Agora chamamos a função que retorna a Struct com linha e coluna
        let info = lexer.next_token_info();

        println!(
            "[Token Debug] L:{}:C{}: {:?}",
            info.line, info.column, info.token
        );

        if info.token == Token::EOF {
            tokens.push(info.clone().token); // Armazenamos o TokenInfo completo
            break;
        }
        tokens.push(info.token);
    }

    println!(
        "{}",
        "--- [DEBUG: LEXER CONCLUÍDO COM SUCESSO] ---"
            .green()
            .bold()
    );

    // 🎯 O Parser agora recebe o Vec<TokenInfo>
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            println!("{}", "--- [AST GERADA COM SUCESSO] ---".blue().bold());
            println!("{:#?}", ast);
        }
        Err(mensagem_de_erro) => {
            // A mensagem 'e' virá formatada do Parser com Linha e Coluna
            println!("\n{}", "-------------------------------------------".red());
            println!(
                "{} {}",
                "❌ ERRO DE COMPILAÇÃO:".red().bold(),
                mensagem_de_erro
            );
            println!("{}", "-------------------------------------------".red());
        }
    }
}
