mod disassembler;
mod chunk;
mod vm;
mod scanner;
mod compiler;
mod value;
mod stack;

use std::io::Write;

extern crate num;
#[macro_use]
extern crate num_derive;


fn repl() {
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let mut vm = vm::init_vm();
        vm.interpret(&input);
    }
}

fn read_file(path: &str) -> String {
    std::fs::read_to_string(path).expect("Something went wrong reading the file")
}

fn run_file(path: &str) {
    let content = read_file(path);
    let mut vm = vm::init_vm();
    vm.interpret(&content);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => println!("Usage: rlox [path]"),
    }
    //let mut vm = vm::init_vm();
    //vm.add_constant_op(1.2, 123);
    //vm.chunk.write_constant(2.50, 124);
    //vm.chunk.write_constant(2.00, 124);
    //vm.chunk.write_opcode(Opcode::Multiply, 124);
    //vm.chunk.write_constant(1.00, 125);
    //vm.chunk.write_opcode(Opcode::Subtract, 125);
    //vm.chunk.write_opcode(Opcode::Return, 125);
    //vm.run();
    //disassembler::disassemble_chunk(&vm.chunk, "code");
}
