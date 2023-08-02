use crate::chunk::{Chunk, Opcode};

/// Disassemble the code section of a chunk
pub(crate) fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {name} ==");
    let mut offset: usize = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

/// Disassemble a single instruction
pub(crate) fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);

    // If this instruction is in the same line as the previous don't show a new line show a |
    // Else, if it has changed, show the line number.
    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.lines[offset]);
    }

    let instruction = chunk.code[offset];
    let maybe_opcode = num::FromPrimitive::from_u8(instruction);
    let to_ret = match maybe_opcode {
        Some(Opcode::Return) => disassemble_simple("RETURN", offset),
        Some(Opcode::Not) => disassemble_simple("NOT", offset),
        Some(Opcode::Equal) => disassemble_simple("EQUAL", offset),
        Some(Opcode::Greater) => disassemble_simple("GREATER", offset),
        Some(Opcode::Less) => disassemble_simple("LESS", offset),
        Some(Opcode::Negate) => disassemble_simple("NEGATE", offset),
        Some(Opcode::Add) => disassemble_simple("ADD", offset),
        Some(Opcode::Subtract) => disassemble_simple("SUBTRACT", offset),
        Some(Opcode::Multiply) => disassemble_simple("MULTIPLY", offset),
        Some(Opcode::Divide) => disassemble_simple("DIVIDE", offset),
        Some(Opcode::Constant) => disassemble_constant("CONSTANT", chunk, offset),
        Some(Opcode::Nil) => disassemble_simple("NIL", offset),
        Some(Opcode::False) => disassemble_simple("FALSE", offset),
        Some(Opcode::True) => disassemble_simple("TRUE", offset),
        None => {
            println!("Unknown opcode {instruction}");
            offset + 1
        }
    };
    to_ret
}

// Disassemble a simple (1 byte) opcode
fn disassemble_simple(name: &str, offset: usize) -> usize {
    println!("{name}");
    offset + 1
}

/// Disassemble a CONSTANT opcode
fn disassemble_constant(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.code[offset + 1].clone() as usize;
    print!("{:<16} {:>4} '", name, constant);
    let value = chunk.constants[constant];
    println!("{value} '");
    offset + 2
}

pub(crate) fn trace_stack(stack: &Vec<f64>) {
    print!("          ");
    for slot in stack {
        print!("[ {slot} ]");
    }
    print!("\n");
}