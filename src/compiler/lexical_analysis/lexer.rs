use crate::compiler::string_handling::{StringAtom, StringInterner};

use super::{Token, TokenType};
use std::cell::Cell;

#[derive(Debug)]
pub struct Lexer<'a> {
    source_string: &'a str,
    start_index: Cell<usize>,
    current_index: Cell<usize>,
    interner: StringInterner,
    keyword_atoms: KeywordAtoms,
}

#[derive(Debug)]
struct KeywordAtoms {
    k_let: StringAtom,
    k_nil: StringAtom,
    k_if: StringAtom,
    k_elseif: StringAtom,
    k_else: StringAtom,
    k_end: StringAtom,
    k_then: StringAtom,
    k_print: StringAtom,
    k_true: StringAtom,
    k_false: StringAtom,
    k_and: StringAtom,
    k_or: StringAtom,
    k_not: StringAtom,
    k_block: StringAtom,
}

impl KeywordAtoms {
    fn with_interner(interner: &StringInterner) -> Self {
        KeywordAtoms {
            k_let: interner.intern("let"),
            k_nil: interner.intern("nil"),
            k_if: interner.intern("if"),
            k_elseif: interner.intern("elseif"),
            k_else: interner.intern("else"),
            k_end: interner.intern("end"),
            k_then: interner.intern("then"),
            k_print: interner.intern("print"),
            k_true: interner.intern("true"),
            k_false: interner.intern("false"),
            k_and: interner.intern("and"),
            k_or: interner.intern("or"),
            k_not: interner.intern("not"),
            k_block: interner.intern("block"),
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(source_string: &'a str, interner: StringInterner) -> Self {
        Lexer {
            source_string,
            start_index: Cell::new(0),
            current_index: Cell::new(0),
            keyword_atoms: KeywordAtoms::with_interner(&interner),
            interner,
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
                    self.advance(); // skip '#'
                    if self.mmatch('/') {
                        let mut comment_level = 1;

                        while comment_level > 0 {
                            let c = self.advance();
                            let pc = self.peek_char();

                            match (c, pc) {
                                // If we encounter a close comment, go higher
                                (Some('/'), Some('#')) => {
                                    comment_level -= 1;
                                    self.advance(); // skip '#'
                                }
                                // If we encounter a start comment, go deeper
                                (Some('#'), Some('/')) => {
                                    comment_level += 1;
                                    self.advance(); // '/'
                                }
                                // if we encounter some other sequence of characters, carry on
                                (Some(_), Some(_)) => {}

                                // if some of them were none, we ran out of characters and should stop
                                _ => break,
                            }
                        }
                    } else {
                        while !self.mmatch('\n') {
                            self.advance();
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token {
            index: self.start_index.get(),
            token_type,
            lexeme: self
                .interner
                .intern(&self.source_string[self.start_index.get()..self.current_index.get()]),
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

    fn finish_number(&self) -> Token {
        while matches!(self.peek_char(), Some(c) if c.is_ascii_digit()) {
            self.advance();
        }
        self.mmatch('.');
        while matches!(self.peek_char(), Some(c) if c.is_ascii_digit()) {
            self.advance();
        }
        self.make_token(TokenType::Number)
    }

    fn finish_identifier(&self) -> Token {
        while matches!(self.peek_char(), Some(c) if c.is_alphanumeric()) {
            self.advance();
        }
        let mut token = self.make_token(TokenType::Identifier);

        let keywords = &self.keyword_atoms;

        token.token_type = match &token.lexeme {
            w if w == &keywords.k_let => TokenType::Let,
            w if w == &keywords.k_nil => TokenType::Nil,
            w if w == &keywords.k_if => TokenType::If,
            w if w == &keywords.k_elseif => TokenType::ElseIf,
            w if w == &keywords.k_else => TokenType::Else,
            w if w == &keywords.k_end => TokenType::End,
            w if w == &keywords.k_then => TokenType::Then,
            w if w == &keywords.k_print => TokenType::Print,
            w if w == &keywords.k_true => TokenType::True,
            w if w == &keywords.k_false => TokenType::False,
            w if w == &keywords.k_and => TokenType::And,
            w if w == &keywords.k_or => TokenType::Or,
            w if w == &keywords.k_not => TokenType::Not,
            w if w == &keywords.k_block => TokenType::Block,
            _ => TokenType::Identifier,
        };
        token
    }

    pub fn lex_token(&self) -> Token {
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
    use crate::compiler::string_handling::StringInterner;

    #[test]
    fn lexing_test() {
        let source = "2 + 3 -     1";
        let interner = StringInterner::new();

        let lexer = Lexer::new(source, interner);
        assert_eq!(lexer.lex_token().token_type, TokenType::Number);
        assert_eq!(lexer.lex_token().token_type, TokenType::Plus);
        assert_eq!(lexer.lex_token().token_type, TokenType::Number);
        assert_eq!(lexer.lex_token().token_type, TokenType::Minus);
        assert_eq!(lexer.lex_token().token_type, TokenType::Number);
    }
}
