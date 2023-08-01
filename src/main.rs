mod disassembler;
mod chunk;
mod vm;

use vm::Opcode;

extern crate num;
#[macro_use]
extern crate num_derive;


fn main() {
    let mut vm = vm::init_vm();

    //vm.add_constant_op(1.2, 123);
    vm.add_constant_op(3.14, 124);
    vm.add_opcode(Opcode::Return, 124);
    vm.run();
    //disassembler::disassemble_chunk(&vm.chunk, "code");
}
