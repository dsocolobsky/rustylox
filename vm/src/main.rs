use common::chunk::Chunk;

use crate::vm::VM;

mod vm;
mod stack;
mod utils;

fn run_file(_path: &str) {
    //let chunk = compile_from_file(path).expect("Failed to compile");
    // TODO fix this, probably will have to move compiler code to common
    let mut vm = VM::init(Chunk::init());
    vm.run();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        2 => run_file(&args[1]),
        _ => println!("Usage: rlox [path]"),
    }
}
