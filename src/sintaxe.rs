pub mod ast;
pub mod core;
pub mod interpreter;
pub mod lexer;
pub mod ministers;
pub mod parser;
pub mod token;

use colored::*;

// Agora os imports vão funcionar
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::{Token, TokenInfo};

fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        let file_: bool = std::path::Path::new(&args[1]).exists();

        if file_ {
            let codigo = std::fs::read_to_string(&args[1]).expect("Erro ao ler arquivo");

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
        } else {
            println!("O arquivo não existe...");
        }
    } else {
        println!("Erro de Argumento: Deve passar o caminho do arquivo");
    }
}
