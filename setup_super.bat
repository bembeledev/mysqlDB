@echo off
echo 👑 [Super Programming Language] - Iniciando construcao da estrutura...

:: 1. Criando as pastas
powershell -Command "New-Item -ItemType Directory -Path src/lexer, src/parser, src/core, src/semantics, src/ministers, examples -Force"

:: 2. Criando src/lexer/mod.rs e src/lexer/token.rs
echo pub mod token; > src/lexer/mod.rs
(
echo #[derive(Debug, Clone, PartialEq)]
echo pub enum Token {
echo     Public, Private, Class, Static, Void,
echo     Int, Double, String, Bool,
echo     Identifier(String^),
echo     IntLiteral(i64^),
echo     LeftBrace, RightBrace, Semicolon, Assign,
echo     PythonBlock(String^),
echo     JavaBlock(String^),
echo }
) > src/lexer/token.rs

:: 3. Criando src/parser/mod.rs e src/parser/ast.rs
echo pub mod ast; > src/parser/mod.rs
(
echo use crate::core::types::SuperType;
echo pub struct SuperClass {
echo     pub name: String,
echo     pub blocks: Vec^<String^>,
echo }
) > src/parser/ast.rs

:: 4. Criando o Sistema de Tipos (src/core/types.rs)
echo pub mod types; > src/core/mod.rs
(
echo #[derive(Debug, Clone, PartialEq)]
echo pub enum SuperType {
echo     Int, Double, String, Bool, Void,
echo }
) > src/core/types.rs

:: 5. Criando os Ministros (Bridges)
(
echo pub mod python_bridge;
echo pub mod js_bridge;
echo pub mod java_bridge;
) > src/ministers/mod.rs

echo fn executar_python(codigo: ^&str^) { println!("Executando Python: {}", codigo); } > src/ministers/python_bridge.rs
echo fn executar_js(codigo: ^&str^) { println!("Executando JS: {}", codigo); } > src/ministers/js_bridge.rs
echo fn executar_java(codigo: ^&str^) { println!("Executando Java: {}", codigo); } > src/ministers/java_bridge.rs

:: 6. Criando o main.rs (O Rei)
(
echo mod lexer;
echo mod parser;
echo mod core;
echo mod semantics;
echo mod ministers;
echo.
echo fn main(^) {
echo     println!("--- [ Super Programming Language (.super) ] ---");
echo     println!("Rei: Sistema de tipagem rigida estilo Java carregado.");
echo }
) > src/main.rs

:: 7. Criando exemplo .super
echo public class Main { int versao = 1; python:: { print('Hello') } } > examples/Main.super

echo ✅ Estrutura completa criada com sucesso!
pause