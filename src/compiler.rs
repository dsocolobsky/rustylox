use crate::chunk::{Chunk, init_chunk};
use crate::scanner;
use crate::scanner::{Token, TokenType};
use crate::vm::Opcode;

enum Precedence {
    None,
    Assignment,  // =
    Or,          // or
    And,         // and
    Equality,    // == !=
    Comparison,  // < > <= >=
    Term,        // + -
    Factor,      // * /
    Unary,       // ! -
    Call,        // . ()
    Primary
}

pub(crate) fn compile(source: &str) -> Option<Chunk> {
    let mut parser = Parser {
        chunk: init_chunk(),
        scanner: scanner::init_scanner(source),
        current: None,
        previous: None,
        had_error: false,
        panic_mode: false,
    };
    parser.advance();
    parser.expression();
    parser.consume(TokenType::EOF, "Expected end of expression");
    parser.emit_return();

    if parser.had_error {
        None
    } else {
        Some(parser.chunk)
    }
}

struct Parser {
    chunk: Chunk,
    scanner: scanner::Scanner,
    current: Option<Token>,
    previous: Option<Token>,
    had_error: bool,
    panic_mode: bool,
}

impl Parser {
    fn advance(&mut self) {
        if let Some(val) = self.current.as_ref() {
            self.previous = Some(val.clone());
        }

        loop {
            self.current = Some(self.scanner.scan_token());
            if self.current_type_is(TokenType::Error) {
                let curr_token = self.current.as_ref().unwrap();
                self.error_at_current(curr_token.lexeme.clone().as_str());
            } else {
                break;
            }
        }
    }

    fn expression(&self) {
        // Parse the lowest possible precedence, which parses all other expressions
        self.parse_precedence(Precedence::Assignment)
    }

    fn parse_precedence(&self, precedence: Precedence) {
        todo!()
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn number(&mut self) {
        let value = self.previous().lexeme.parse::<f64>().expect("Could not parse number");
        self.emit_constant(value);
    }

    fn unary(&mut self) {
        let operator_type = self.previous().token_type.clone();

        // Allows to parse nested unary expressions like !!variable
        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_opcode(Opcode::Negate),
            _ => unreachable!(),
        }
    }

    fn emit_constant(&mut self, value: f64) {
        self.chunk.write_constant(value, self.previous().line);
    }

    fn emit_return(&mut self) {
        self.emit_opcode(Opcode::Return);
    }

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write_byte(byte, self.previous().line);
    }

    fn emit_opcode(&mut self, opcode: Opcode) {
        self.chunk.write_opcode(opcode, self.previous().line);
    }

    fn make_constant(&mut self, value: f64) -> usize {
        self.chunk.add_constant(value)
    }

    fn previous(&self) -> &Token {
        self.previous.as_ref().unwrap()
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.current_type_is(token_type) {
            self.advance();
            return;
        }
        self.error_at_current(message);
    }

    fn current_type_is(&self, token_type: TokenType) -> bool {
        match self.current.as_ref() {
            Some(token) => token.token_type == token_type,
            None => false,
        }
    }

    fn error_at_current(&mut self, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        let token = self.current.as_ref().unwrap();
        self.error_at(token, message);
    }

    fn error_at(&self, token: &Token, message: &str) {
        if self.panic_mode {
            return;
        }
        eprintln!("[line {0}] Error", token.line);

        match token.token_type {
            TokenType::EOF => eprint!(" at end"),
            TokenType::Error => (),
            _ => eprint!(" at {0}",  &token.lexeme),
        }
    }
}
