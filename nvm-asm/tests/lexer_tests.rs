// nvm-asm/tests/lexer_tests.rs
//
// Integration tests for the lexer.
pub mod lexer_tests {
    mod number_tests;
    mod position_tests;
    mod token_tests;

    use nvm_asm::{
        lexer::{
            Lexer,
            err::LexerError,
            token::{Token, TokenKind},
        },
        src::SourceCode,
        str_pool::StrPool,
    };

    // Parses the source code and returns the found tokens and errors.
    pub fn tokenize(src: &str) -> (Vec<Token>, Vec<LexerError>) {
        let source = SourceCode::new(src.to_string());
        let mut str_pool = StrPool::from_source(&source);
        let mut lexer = Lexer::new(source, &mut str_pool);

        let tokens = lexer.tokenize().to_vec();

        (tokens, lexer.errors)
    }

    // Parses the source code and returns only the token kinds.
    pub fn kinds(src: &str) -> Vec<TokenKind> {
        tokenize(src)
            .0
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }
}
