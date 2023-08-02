use crate::chunk::{Chunk, init_chunk};
use crate::disassembler::disassemble_chunk;
use crate::{scanner, value};
use crate::scanner::{Token, TokenType};
use crate::vm::Opcode;

#[repr(u8)]
#[derive(FromPrimitive, PartialEq, PartialOrd)]
enum Precedence {
    None = 0,
    Assignment = 1,  // =
    Or = 2,          // or
    And = 3,         // and
    Equality = 4,    // == !=
    Comparison = 5,  // < > <= >=
    Term = 6,        // + -
    Factor = 7,      // * /
    Unary = 8,       // ! -
    Call = 9,        // . ()
    Primary = 10,
}

/// A struct representing a rule for parsing
/// Represents a single row in the parsing table
struct ParseRule {
    prefix: Option<fn(&mut Parser)>,
    infix: Option<fn(&mut Parser)>,
    precedence: Precedence
}

fn parse_rule(token_type: &TokenType) -> ParseRule {
    match token_type {
        TokenType::LeftParen => ParseRule { prefix: Some(Parser::grouping), infix: None, precedence: Precedence::None },
        TokenType::Minus => ParseRule { prefix: Some(Parser::unary), infix: Some(Parser::binary), precedence: Precedence::Term },
        TokenType::Plus => ParseRule { prefix: None, infix: Some(Parser::binary), precedence: Precedence::Term },
        TokenType::Slash => ParseRule { prefix: None, infix: Some(Parser::binary), precedence: Precedence::Factor },
        TokenType::Star => ParseRule { prefix: None, infix: Some(Parser::binary), precedence: Precedence::Factor },
        TokenType::Number => ParseRule { prefix: Some(Parser::number), infix: None, precedence: Precedence::None },
        _ => ParseRule { prefix: None, infix: None, precedence: Precedence::None }
    }
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
        disassemble_chunk(&parser.chunk, "code");
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

    fn expression(&mut self) {
        // Parse the lowest possible precedence, which parses all other expressions
        self.parse_precedence(Precedence::Assignment)
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = parse_rule(&self.previous_token_type()).prefix;
        prefix_rule.expect("Expect expression")(self);

        while precedence <= parse_rule(&self.current_type()).precedence {
            self.advance();
            let infix_rule = parse_rule(&self.previous_token_type()).infix;
            infix_rule.expect("Expect expression")(self);
        }
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
        let operator_type = self.previous_token_type();

        // Allows to parse nested unary expressions like !!variable
        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_opcode(Opcode::Negate),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self) {
        let operator_type = self.previous_token_type();

        let rule = parse_rule(&operator_type);
        let precedence_to_parse = (rule.precedence as u8) + 1;
        let precedence: Option<Precedence> = num::FromPrimitive::from_u8(precedence_to_parse);
        self.parse_precedence(precedence.expect("Could not convert u8 to Precedence"));

        match operator_type {
            TokenType::Plus => self.emit_opcode(Opcode::Add),
            TokenType::Minus => self.emit_opcode(Opcode::Subtract),
            TokenType::Star => self.emit_opcode(Opcode::Multiply),
            TokenType::Slash => self.emit_opcode(Opcode::Divide),
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

    fn current_type(&self) -> TokenType {
        self.current.as_ref().unwrap().token_type.clone()
    }

    fn current_type_is(&self, token_type: TokenType) -> bool {
        match self.current.as_ref() {
            Some(token) => token.token_type == token_type,
            None => false,
        }
    }

    fn previous_token_type(&self) -> TokenType {
        self.previous().token_type.clone()
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
