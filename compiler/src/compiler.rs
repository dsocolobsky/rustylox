use num_derive::FromPrimitive;

use common::{chunk::Chunk, Constant, disassembler::disassemble_chunk, opcode::Opcode};

use crate::scanner;
use crate::scanner::{Token, TokenType};

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
    prefix: Option<fn(&mut Parser, bool)>,
    infix: Option<fn(&mut Parser)>,
    precedence: Precedence
}

struct Local {
    name: Token,
    depth: i16 // Scope depth of the block where the variable was defined
}

fn parse_rule(token_type: &TokenType) -> ParseRule {
    use TokenType::*;
    match token_type {
        LeftParen =>
            ParseRule { prefix: Some(Parser::grouping), infix: None, precedence: Precedence::None },
        Bang =>
            ParseRule { prefix: Some(Parser::unary), infix: None, precedence: Precedence::None },
        Minus =>
            ParseRule { prefix: Some(Parser::unary), infix: Some(Parser::binary), precedence: Precedence::Term },
        Plus =>
            ParseRule { prefix: None, infix: Some(Parser::binary), precedence: Precedence::Term },
        Slash | Star =>
            ParseRule { prefix: None, infix: Some(Parser::binary), precedence: Precedence::Factor },
        Number =>
            ParseRule { prefix: Some(Parser::number), infix: None, precedence: Precedence::None },
        String =>
            ParseRule { prefix: Some(Parser::string), infix: None, precedence: Precedence::None },
        Nil | False | True  =>
            ParseRule { prefix: Some(Parser::literal), infix: None, precedence: Precedence::None },
        BangEqual | EqualEqual =>
            ParseRule { prefix: None, infix: Some(Parser::binary), precedence: Precedence::Equality },
        Greater | GreaterEqual | Less | LessEqual =>
            ParseRule { prefix: None, infix: Some(Parser::binary), precedence: Precedence::Comparison },
        Identifier =>
            ParseRule { prefix: Some(Parser::variable), infix: None, precedence: Precedence::None },
        _ =>
            ParseRule { prefix: None, infix: None, precedence: Precedence::None }
    }
}

pub fn compile(source: &str) -> Option<Chunk> {
    let mut parser = Parser::init(source);
    //let mut compiler = Compiler::init();

    parser.advance();
    while !parser.tmatch(TokenType::EOF) {
        parser.parse_declaration();
    }
    parser.consume(TokenType::EOF, "Expected end of expression");

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
    // compiler stuff
    locals: Vec<Local>,
    scope_depth: i16
}

impl Parser {
    fn init(source: &str) -> Parser {
        Parser {
            chunk: Chunk::init(),
            scanner: scanner::init_scanner(source),
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

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

    fn parse_declaration(&mut self) {
        if self.tmatch(TokenType::Var) {
            self.parse_variable_declaration();
        } else {
            self.parse_statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn parse_variable_declaration(&mut self) {
        let global = self.parse_variable_name();

        if self.tmatch(TokenType::Equal) {
            self.expression(); // var a = expr;
        } else {
            self.emit_opcode(Opcode::Nil); // var a; -> var a = nil;
        }

        self.consume(TokenType::Semicolon,
                     "Expected ; after variable declaration");

        self.define_variable(global);
    }

    fn parse_variable_name(&mut self) -> usize {
        self.consume(TokenType::Identifier, "Expected variable name");

        self.declare_variable();
        if self.is_in_local_scope() {
            return 0; // Doesnt matter so return whatever
        }

        if let Some(ident_token) = self.previous.clone() {
            self.make_constant(Constant::String(ident_token.lexeme))
        } else {
            panic!("Expected previous to be an identifier")
        }
    }

    fn mark_initialized(&mut self) {
        if self.is_in_local_scope() {
            let local = self.locals.last_mut().unwrap();
            local.depth = self.scope_depth;
        }
    }

    fn declare_variable(&mut self) {
        if self.is_in_global_scope() {
            return; // We only do this for locals
        }

        let name = self.previous.clone().unwrap();

        if self.name_already_exists_in_scope(&name) {
            self.error_at_current("Variable with this name already declared in this scope");
        } else {
            self.add_local(name);
        }
    }

    fn variable(&mut self, can_assign: bool) {
        if let Some(name) = self.previous.clone() {
            self.named_variable(name, can_assign);
        } else {
            panic!("Expected previous to be a token")
        }
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let (index, get_opt, set_opt) = match self.resolve_local(name.clone()) {
            Some(local_index) => {
                (local_index, Opcode::GetLocal, Opcode::SetLocal)
            },
            None => {
                (self.chunk.write_identifier_constant(name.lexeme) as u8, Opcode::GetGlobal, Opcode::SetGlobal)
            }
        };

        if can_assign && self.tmatch(TokenType::Equal) {
            self.expression();
            self.emit_opcode(set_opt);
        } else {
            self.emit_opcode(get_opt);
        }

        self.emit_byte(index);
    }

    fn resolve_local(&mut self, name: Token) -> Option<u8> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if name.lexeme == local.name.lexeme {
                if local.depth == -1 {
                    self.error_at_current("Cannot read local variable in its own initializer");
                }
                return Some(i as u8);
            }
        }
        None
    }

    fn define_variable(&mut self, global_index: usize) {
        if self.scope_depth > 0 {
            self.mark_initialized();
            // If we are inside a local scope: return,
            // there is nothing left to do, the variable is already in the stack
            return;
        }
        self.emit_opcode(Opcode::DefineGlobal);
        self.emit_byte(global_index as u8);
    }

    fn parse_statement(&mut self) {
        if self.tmatch(TokenType::Print) {
            self.parse_print_statement();
        } else if self.tmatch(TokenType::Return) {
            self.parse_return_statement();
        } else if self.tmatch(TokenType::LeftBrace) {
            self.begin_scope();
            self.parse_block();
            self.end_scope();
        } else {
            self.parse_expression_statement();
        }
    }

    fn parse_print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_opcode(Opcode::Print);
    }

    fn parse_return_statement(&mut self) {
        if self.tmatch(TokenType::Semicolon) {
            self.emit_return();
        } else {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after return value.");
            self.emit_opcode(Opcode::Return);
        }
    }

    fn parse_block(&mut self) {
        while !self.current_type_is(TokenType::RightBrace) && !self.current_type_is(TokenType::EOF) {
            self.parse_declaration();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn expression(&mut self) {
        // Parse the lowest possible precedence, which parses all other expressions
        self.parse_precedence(Precedence::Assignment)
    }

    // An expression statement evaluates the expression and discards the result
    // for example a function call: myfun(arg);
    fn parse_expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression statement.");
        self.emit_opcode(Opcode::Pop);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        // This will determine if the expression can be assigned to
        let can_assign = precedence <= Precedence::Assignment;

        let prefix_rule = parse_rule(&self.previous_token_type()).prefix;
        prefix_rule.expect("Expected expression")(self, can_assign);

        while precedence <= parse_rule(&self.current_type()).precedence {
            self.advance();
            let infix_rule = parse_rule(&self.previous_token_type()).infix;
            infix_rule.expect("Expect expression")(self);
        }

        if can_assign && self.tmatch(TokenType::Equal) {
            self.error_at_current("Invalid assignment target");
        }
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn number(&mut self, _can_assign: bool) {
        let value = self.previous().lexeme.parse::<f64>().expect("Could not parse number");
        self.emit_constant(Constant::Number(value));
    }

    fn string(&mut self, _can_assign: bool) {
        let tok = self.previous().clone();
        self.emit_constant(Constant::String(tok.lexeme.clone()));
    }

    fn unary(&mut self, _can_assign: bool) {
        let operator_type = self.previous_token_type();

        // Allows to parse nested unary expressions like !!variable
        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Bang => self.emit_opcode(Opcode::Not),
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
            TokenType::BangEqual => {
                self.emit_opcode(Opcode::Equal);
                self.emit_opcode(Opcode::Not);
            },
            TokenType::EqualEqual => self.emit_opcode(Opcode::Equal),
            TokenType::Greater => self.emit_opcode(Opcode::Greater),
            TokenType::GreaterEqual => {
                self.emit_opcode(Opcode::Less);
                self.emit_opcode(Opcode::Not);
            },
            TokenType::Less => self.emit_opcode(Opcode::Less),
            TokenType::LessEqual => {
                self.emit_opcode(Opcode::Greater);
                self.emit_opcode(Opcode::Not);
            },
            _ => unreachable!(),
        }
    }

    fn literal(&mut self, _can_assign: bool) {
        match self.previous_token_type() {
            TokenType::Nil  => self.emit_opcode(Opcode::Nil),
            TokenType::False => self.emit_opcode(Opcode::False),
            TokenType::True => self.emit_opcode(Opcode::True),
            _ => unreachable!(),
        }
    }

    fn emit_constant(&mut self, constant: Constant) {
        self.chunk.write_constant(constant, self.previous().line);
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

    fn make_constant(&mut self, constant: Constant) -> usize {
        self.chunk.add_constant(constant)
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

    // match: this is match from the book, renamed as match is a keyword in Rust
    // TODO rename to something better since this also advances, or check if we can change
    fn tmatch(&mut self, token_type: TokenType) -> bool {
        if !self.current_type_is(token_type) {
            false
        } else {
            self.advance();
            true
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
        eprintln!("[line {0}] Error {1}", token.line, message);

        match token.token_type {
            TokenType::EOF => eprint!(" at end"),
            TokenType::Error => (),
            _ => eprint!(" at {0}",  &token.lexeme),
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while !self.current_type_is(TokenType::EOF) {
            if self.previous_token_type() == TokenType::Semicolon { return; }
            match self.current_type() {
                TokenType::Class | TokenType::Fun | TokenType::Var | TokenType::For | TokenType::If
                | TokenType::While | TokenType::Print | TokenType::Return => { return; },
                _ => {},
            }
        }
        self.advance();
    }

    // ============= COMPILER STUFF ======
    // TODO move to a separate file or struct
    fn is_in_global_scope(&self) -> bool {
        self.scope_depth == 0
    }

    fn is_in_local_scope(&self) -> bool {
        self.scope_depth > 0
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        while !self.locals.is_empty() && self.locals.last().unwrap().depth > self.scope_depth {
            self.emit_opcode(Opcode::Pop);
            self.locals.pop();
        }
    }

    fn add_local(&mut self, name: Token) {
        if self.locals.len() == u8::MAX as usize {
            panic!("Too many local variables in function");
        }
        self.locals.push(Local { name, depth: -1 });
    }

    fn name_already_exists_in_scope(&self, name: &Token) -> bool {
        // We iterate backwards since the current scope is going to be at the end
        for local in self.locals.iter().rev() {
            if local.depth != -1 && local.depth < self.scope_depth {
                break;
            }
            if name.lexeme == local.name.lexeme {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use common::{Constant, opcode::Opcode};

    use crate::compiler::compile;

    // Translates between a vector of Opcode and the u8 representation
    macro_rules! opcodes {
        ($($opcode:expr),*) => {
            vec![$($opcode as u8),*]
        };
    }

    #[test]
    fn return_a_number() {
        let Some(chunk) = compile("return 4;") else { panic!() };
        assert_eq!(chunk.code, opcodes![Opcode::Constant, 0, Opcode::Return]);
    }

    #[test]
    fn return_a_string() {
        let Some(chunk) = compile("return \"hello\";") else { panic!() };
        assert_eq!(chunk.code, opcodes![Opcode::Constant, 0, Opcode::Return]);
        assert_eq!(chunk.constants[0], Constant::String("hello".to_string()));
    }

    #[test]
    fn perform_math_operations() {
        let Some(chunk) = compile("return 3 + 4 * 5;") else { panic!() };
        assert_eq!(chunk.code, opcodes![
            Opcode::Constant, 0,
            Opcode::Constant, 1,
            Opcode::Constant, 2,
            Opcode::Multiply,
            Opcode::Add,
            Opcode::Return
        ]);
        assert_eq!(chunk.constants[0], Constant::Number(3.0));
        assert_eq!(chunk.constants[1], Constant::Number(4.0));
        assert_eq!(chunk.constants[2], Constant::Number(5.0));
    }

    #[test]
    fn equality() {
        let Some(chunk) = compile("return 1 == 2;") else { panic!() };
        assert_eq!(chunk.code, opcodes![
            Opcode::Constant, 0,
            Opcode::Constant, 1,
            Opcode::Equal,
            Opcode::Return
        ]);
        assert_eq!(chunk.constants[0], Constant::Number(1.0));
        assert_eq!(chunk.constants[1], Constant::Number(2.0));
    }

    #[test]
    fn global_variables() {
        let Some(chunk) = compile("var myvar = 4;\nreturn myvar;") else { panic!() };
        assert_eq!(chunk.constants[0], Constant::String("myvar".to_string()));
        assert_eq!(chunk.constants[1], Constant::Number(4.0));
        assert_eq!(chunk.code, opcodes![
            Opcode::Constant, 1, // Not 0 because we have "myvar" there
            Opcode::DefineGlobal, 0,
            Opcode::GetGlobal, 2, // Not sure why the 2 here but clox does the same
            Opcode::Return
        ]);
    }

    #[test]
    fn multiply_global_variables() {
        let Some(chunk) = compile("var a = 3;\nvar b = 4;return a*b;") else { panic!() };
        assert_eq!(chunk.constants[0], Constant::String("a".to_string()));
        assert_eq!(chunk.constants[1], Constant::Number(3.0));
        assert_eq!(chunk.constants[2], Constant::String("b".to_string()));
        assert_eq!(chunk.constants[3], Constant::Number(4.0));
        assert_eq!(chunk.code, opcodes![
            Opcode::Constant, 1, // Since in [0] we have "a"
            Opcode::DefineGlobal, 0, // Define global value for "a"
            Opcode::Constant, 3, // Since in [1] we have 3.0 and in [2] we have "b"
            Opcode::DefineGlobal, 2, // Define global value for "b"
            Opcode::GetGlobal, 4,
            Opcode::GetGlobal, 5,
            Opcode::Multiply,
            Opcode::Return
        ]);
    }

    // TODO: this is the same as in clox but I have to figure out better why the indexes are like this
    #[test]
    fn set_global_variable() {
        let Some(chunk) = compile("var a = 3;\na = 4;\nreturn a;") else { panic!() };
        assert_eq!(chunk.constants[0], Constant::String("a".to_string()));
        assert_eq!(chunk.constants[1], Constant::Number(3.0));
        assert_eq!(chunk.constants[2], Constant::String("a".to_string()));
        assert_eq!(chunk.constants[3], Constant::Number(4.0));
        assert_eq!(chunk.code, opcodes![
            Opcode::Constant, 1,
            Opcode::DefineGlobal, 0,
            Opcode::Constant, 3,
            Opcode::SetGlobal, 2,
            Opcode::Pop,
            Opcode::GetGlobal, 4,
            Opcode::Return
        ]);
    }

    #[test]
    fn local_variables() {
        let Some(chunk) = compile("{ var a = 4.0; print a; }") else { panic!() };
        /*assert_eq!(chunk.constants[0], Constant::String("a".to_string()));
        assert_eq!(chunk.constants[1], Constant::Number(3.0));
        assert_eq!(chunk.constants[2], Constant::String("b".to_string()));
        assert_eq!(chunk.constants[3], Constant::Number(4.0));
        assert_eq!(chunk.code, opcodes![
            Opcode::Constant, 1, // Since in [0] we have "a"
            Opcode::DefineGlobal, 0, // Define global value for "a"
            Opcode::Constant, 3, // Since in [1] we have 3.0 and in [2] we have "b"
            Opcode::DefineGlobal, 2, // Define global value for "b"
            Opcode::GetGlobal, 4,
            Opcode::GetGlobal, 5,
            Opcode::Multiply,
            Opcode::Return
        ]);*/
    }
}
