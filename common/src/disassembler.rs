use crate::chunk::Chunk;
use crate::opcode::Opcode;

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {name} ==");
    let mut offset: usize = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

/// Disassemble a single instruction
pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
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
        Some(op) => {
            let disasm = |name| {
                disassemble_simple(name, offset)
            };
            match op {
                Opcode::Return => disasm("RETURN"),
                Opcode::Not => disasm("NOT"),
                Opcode::Equal => disasm("EQUAL"),
                Opcode::Greater => disasm("GREATER"),
                Opcode::Less => disasm("LESS"),
                Opcode::Negate => disasm("NEGATE"),
                Opcode::Add => disasm("ADD"),
                Opcode::Subtract => disasm("SUBTRACT"),
                Opcode::Multiply => disasm("MULTIPLY"),
                Opcode::Divide => disasm("DIVIDE"),
                Opcode::Constant => disassemble_constant("CONSTANT", chunk, offset),
                Opcode::Nil => disasm("NIL"),
                Opcode::False => disasm("FALSE"),
                Opcode::True => disasm("TRUE"),
                Opcode::Print => disasm("PRINT"),
                Opcode::Pop => disasm("POP"),
                Opcode::DefineGlobal => disassemble_constant("DEFINE_GLOBAL", chunk, offset),
                Opcode::GetGlobal => disassemble_constant("GET_GLOBAL", chunk, offset),
                Opcode::SetGlobal => disassemble_constant("SET_GLOBAL", chunk, offset),
                Opcode::GetLocal => disassemble_get_local("GET_LOCAL", chunk, offset),
                Opcode::SetLocal => disassemble_get_local("SET_LOCAL", chunk, offset),
                Opcode::Jump => disassemble_short_jump("JUMP", 1, chunk, offset),
                Opcode::JumpIfFalse => disassemble_short_jump("JUMP_IF_FALSE", 1, chunk, offset),
                Opcode::Push => disassemble_get_local("PUSH", chunk, offset),
            }
        }
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
    let value = &chunk.constants[constant];
    println!("{value}'");
    offset + 2
}

fn disassemble_get_local(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.code[offset + 1].clone() as usize;
    println!("{:<16} {:>4} '", name, constant);
    offset + 2
}

fn disassemble_short_jump(name: &str, sign: i8, chunk: &Chunk, offset: usize) -> usize {
    let byte1 = chunk.code[offset + 1].clone() as usize;
    let byte2 = chunk.code[offset + 2].clone() as usize;
    let jump = byte1 << 8 | byte2;
    let j2 = (sign as i32) * (jump as i32);
    if let Some(total_jump) = add_offset(offset + 3, j2) {
        println!("{:<16} {:>4} -> {:4}'", name, offset, total_jump);
    } else {
        println!("{:<16} {:>4} -> ???'", name, offset);
    }
    offset + 3
}

fn add_offset(u: usize, i: i32) -> Option<usize> {
    if i.is_negative() {
        u.checked_sub(i.wrapping_abs() as u32 as usize)
    } else {
        u.checked_add(i as usize)
    }
}
