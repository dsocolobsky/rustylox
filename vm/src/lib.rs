use common::chunk::Chunk;

pub mod vm;
pub mod stack;

fn run_file(_path: &str) {
    //let chunk = compile_from_file(path).expect("Failed to compile");
    // TODO fix this, probably will have to move compiler code to common
    let mut vm = vm::VM::init(Chunk::init());
    vm.run();
}
