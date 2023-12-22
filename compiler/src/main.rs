use common::chunk::Chunk;

mod compiler;
mod scanner;

pub fn compile_from_file(path: &str) -> Option<Chunk> {
    let content = std::fs::read_to_string(path).
        expect("Something went wrong reading the file");
    compiler::compile(&content)
}

fn run_file(path: &str) {
    let chunk = compile_from_file(path).expect("Failed to compile");
    // Print chunk
    println!("=== Chunk ===");
    for (i, byte) in chunk.code.iter().enumerate() {
        println!("{:04} {:04} {}", i, chunk.lines[i], byte);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        2 => run_file(&args[1]),
        _ => println!("Usage: rlox [path]"),
    }
}
