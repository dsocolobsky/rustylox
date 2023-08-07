use phf_macros::phf_map;

#[derive(Eq, PartialEq, Debug, Clone)]
pub(crate) enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace, Comma, Dot, Minus, Plus, Semicolon, Slash, Star,
    // One or two character tokens.
    Bang, BangEqual, Equal, EqualEqual, Greater, GreaterEqual, Less, LessEqual,
    // Literals.
    Identifier, String, Number,
    // Keywords.
    And, Class, Else, False, Fun, For, If, Nil, Or, Print, Return, Super, This, True, Var, While,
    // Error and End of file.
    Error, EOF,
}

// List of keywords
static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "class" => TokenType::Class,
    "else" => TokenType::Else,
    "false" => TokenType::False,
    "for" => TokenType::For,
    "fun" => TokenType::Fun,
    "if" => TokenType::If,
    "nil" => TokenType::Nil,
    "or" => TokenType::Or,
    "print" => TokenType::Print,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "true" => TokenType::True,
    "var" => TokenType::Var,
    "while" => TokenType::While,
};

#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub(crate) token_type: TokenType,
    pub(crate) start: usize,
    pub(crate) length: usize,
    pub(crate) line: usize,
    pub(crate) lexeme: String,
}

pub(crate) struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

pub(crate) fn init_scanner(source: &str) -> Scanner {
    Scanner {
        source: source.to_string(),
        tokens: Vec::new(),
        start: 0,
        current: 0,
        line: 1,
    }
}

impl Scanner {
    pub(crate) fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(TokenType::EOF);
        }

        let c = self.advance();

        if c.is_alphabetic() || c == '_' {
            return self.identifier();
        }

        if c.is_digit(10) {
            return self.number();
        }

        match c {
            '(' => return self.make_token(TokenType::LeftParen),
            ')' => return self.make_token(TokenType::RightParen),
            '{' => return self.make_token(TokenType::LeftBrace),
            '}' => return self.make_token(TokenType::RightBrace),
            ';' => return self.make_token(TokenType::Semicolon),
            ',' => return self.make_token(TokenType::Comma),
            '.' => return self.make_token(TokenType::Dot),
            '-' => return self.make_token(TokenType::Minus),
            '+' => return self.make_token(TokenType::Plus),
            '/' => return self.make_token(TokenType::Slash),
            '*' => return self.make_token(TokenType::Star),
            '!' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::BangEqual);
                } else {
                    return self.make_token(TokenType::Bang);
                }
            },
            '=' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::EqualEqual);
                } else {
                    return self.make_token(TokenType::Equal);
                }
            },
            '<' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::LessEqual);
                } else {
                    return self.make_token(TokenType::Less);
                }
            },
            '>' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::GreaterEqual);
                } else {
                    return self.make_token(TokenType::Greater);
                }
            },
            '"' => return self.string(),
            _ => {}
        }

        self.error_token("Unexpected character")
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.char_at(self.current) != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();

            match c {
                ' ' | '\r' | '\t' => self.current += 1,
                '\n' => {
                    self.line += 1;
                    self.current += 1;
                },
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.current += 1;
                        }
                    } else {
                        return;
                    }
                },
                _ => return,
            }
        }
    }

    fn identifier(&mut self) -> Token {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.current += 1;
        }

        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        let text = &self.source[self.start..self.current];
        match KEYWORDS.get(text) {
            Some(&ref token_type) => token_type.clone(),
            None => TokenType::Identifier,
        }
    }

    fn number(&mut self) -> Token {
        while self.peek().is_digit(10) {
            self.current += 1;
        }

        // Look for a fractional part
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.current += 1;
            while self.peek().is_digit(10) {
                self.current += 1;
            }
        }

        self.make_token(TokenType::Number)
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.current += 1;
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string");
        }

        // Remove quotes from string
        self.start += 1;
        self.current -= 1;

        self.advance();
        let tok = self.make_token(TokenType::String);

        // Restore start and current to continue scanning correctly
        self.start -= 1;
        self.current += 1;

        tok
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token {
            token_type,
            start: self.start,
            length: (self.current - self.start),
            line: self.line,
            lexeme: self.source[self.start..self.current].to_string(),
        }
    }

    fn error_token(&self, message: &str) -> Token {
        Token {
            token_type: TokenType::Error,
            start: 0,
            length: message.len(),
            line: self.line,
            lexeme: message.to_string(),
        }
    }

    fn advance(&mut self) -> char {
        let current_char = self.char_at(self.current);
        self.current += 1;
        current_char
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.char_at(self.current)
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.char_at(self.current + 1)
    }

    fn char_at(&self, index: usize) -> char {
        self.source.chars().nth(index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn number() {
        // Parse a number
        let source = "123";
        let mut scanner = super::init_scanner(source);
        let token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::Number);
        assert_eq!(token.lexeme, "123");
    }

    #[test]
    fn symbols() {
        // Parse a number
        let source = "+ - * /";
        let mut scanner = super::init_scanner(source);
        let token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::Plus);
        assert_eq!(token.lexeme, "+");
        let token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::Minus);
        assert_eq!(token.lexeme, "-");
        let token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::Star);
        assert_eq!(token.lexeme, "*");
        let token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::Slash);
        assert_eq!(token.lexeme, "/");
    }

    #[test]
    fn string() {
        let source = "\"Hello, world!\"";
        let mut scanner = super::init_scanner(source);
        let token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::String);
        assert_eq!(token.lexeme, "Hello, world!");
    }

    #[test]
    fn keywords() {
        let source = "and class else";
        let mut scanner = super::init_scanner(source);
        let mut token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::And);
        token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::Class);
        token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::Else);
    }

    #[test]
    fn addition() {
        let source = "1 + 2";
        let mut scanner = super::init_scanner(source);
        let mut token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::Number);
        assert_eq!(token.lexeme, "1");
        token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::Plus);
        token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::Number);
        assert_eq!(token.lexeme, "2");
    }

    #[test]
    fn concat() {
        let source = "\"bat\" + \"man\"";
        let mut scanner = super::init_scanner(source);
        let mut token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::String);
        assert_eq!(token.lexeme, "bat");
        token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::Plus);
        token = scanner.scan_token();
        assert_eq!(token.token_type, super::TokenType::String);
        assert_eq!(token.lexeme, "man");
    }

    #[test]
    fn another() {
        panic!("Make this test fail");
    }
}
