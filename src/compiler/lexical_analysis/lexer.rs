use std::cell::Cell;
use super::{Token, TokenType};

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    source_string: &'a str,
    start_index: Cell<usize>,
    current_index: Cell<usize>,
}

impl<'a> Lexer<'a> {
    pub fn new(source_string: &'a str) -> Self {
        Lexer {
            source_string,
            start_index: Cell::new(0),
            current_index: Cell::new(0),
        }
    }

    fn peek_char(&self) -> Option<char> {
        let x = &self.source_string[self.current_index.get()..];
        let c = x.chars().next();
        c
    }

    fn advance(&self) -> Option<char> {
        let c = self.peek_char();
        if let Some(c) = c {
            self.current_index
                .set(self.current_index.get() + c.len_utf8());
        }
        c
    }

    fn skip_whitespace(&self) {
        loop {
            match self.peek_char() {
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }

                // skip comments
                Some(c) if c == '#' => {
                    while !self.mmatch('\n') {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn make_token(&self, token_type: TokenType) -> Token<'a> {
        Token {
            index: self.start_index.get(),
            token_type,
            lexeme: &self.source_string[self.start_index.get()..self.current_index.get()],
        }
    }

    fn check(&self, expected: char) -> bool {
        matches!(self.peek_char(), Some(c) if c == expected)
    }

    fn mmatch(&self, expected: char) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn finish_number(&self) -> Token<'a> {
        while matches!(self.peek_char(), Some(c) if c.is_ascii_digit()) {
            self.advance();
        }
        self.mmatch('.');
        while matches!(self.peek_char(), Some(c) if c.is_ascii_digit()) {
            self.advance();
        }
        self.make_token(TokenType::Number)
    }

    fn finish_identifier(&self) -> Token<'a> {
        while matches!(self.peek_char(), Some(c) if c.is_alphanumeric()) {
            self.advance();
        }
        let mut token = self.make_token(TokenType::Identifier);

        token.token_type = match token.lexeme {
            "let" => TokenType::Let,
            "nil" => TokenType::Nil,
            "if" => TokenType::If,
            "elseif" => TokenType::ElseIf,
            "else" => TokenType::Else,
            "end" => TokenType::End,
            "then" => TokenType::Then,
            "print" => TokenType::Print,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            _ => TokenType::Identifier,
        };
        token
    }

    pub fn lex_token(&self) -> Token<'a> {
        self.skip_whitespace();
        self.start_index.set(self.current_index.get());

        let c = match self.advance() {
            None => return self.make_token(TokenType::Eof),
            Some(c) => c,
        };

        match c {
            '(' => self.make_token(TokenType::ParenOpen),
            ')' => self.make_token(TokenType::ParenClose),

            '+' => self.make_token(TokenType::Plus),
            '-' => self.make_token(TokenType::Minus),

            '=' => self.make_token(TokenType::Equal),

            '*' => self.make_token(if self.mmatch('*') {
                TokenType::DoubleStar
            } else {
                TokenType::Star
            }),

            '/' => self.make_token(if self.mmatch('/') {
                TokenType::DoubleSlash
            } else {
                TokenType::Slash
            }),

            '<' => self.make_token(if self.mmatch('=') {
                TokenType::LessEqual
            } else {
                TokenType::Less
            }),

            '>' => self.make_token(if self.mmatch('=') {
                TokenType::GreaterEqual
            } else {
                TokenType::Greater
            }),

            ':' if self.mmatch('=') => self.make_token(TokenType::SemicolonEqual),

            '!' if self.mmatch('=') => self.make_token(TokenType::BangEqual),

            c if c.is_ascii_digit() => self.finish_number(),

            c if c.is_alphabetic() => self.finish_identifier(),

            _ => self.make_token(TokenType::BadCharacter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, TokenType};

    #[test]
    fn lexing_test() {
        let source = "2 + 3 -     1";
        let lexer = Lexer::new(source);
        assert_eq!(lexer.lex_token().token_type, TokenType::Number);
        assert_eq!(lexer.lex_token().token_type, TokenType::Plus);
        assert_eq!(lexer.lex_token().token_type, TokenType::Number);
        assert_eq!(lexer.lex_token().token_type, TokenType::Minus);
        assert_eq!(lexer.lex_token().token_type, TokenType::Number);
    }
}