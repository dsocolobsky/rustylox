use crate::scanner::TokenType;

pub(crate) fn compile(source: &str) -> bool {
    let mut scanner = crate::scanner::init_scanner(source);
    let mut line: Option<usize> = None;
    loop {
        let token = scanner.scan_token();
        if let Some(curr_line) = line {
            if token.line != curr_line {
                print!("{:4} ", token.line);
                line = Some(token.line);
            } else {
                print!("   | ");
            }
        } else {
            // Handle the first iteration when line is None
            print!("{:4} ", token.line);
            line = Some(token.line);
        }
        println!("{:?}", token);
        if token.token_type == TokenType::EOF {
            break;
        }
    }
    true
}
