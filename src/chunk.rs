pub(crate) struct Chunk {
    pub(crate) code: Vec<u8>,
    pub(crate) lines: Vec<usize>,
    pub(crate) constants: Vec<f64>
}

pub(crate) fn init_chunk() -> Chunk {
    Chunk {
        code: Vec::new(),
        lines: Vec::new(),
        constants: Vec::new(),
    }
}

impl Chunk {
    pub(crate) fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub(crate) fn write_value(&mut self, value: f64) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub(crate) fn code_len(&self) -> usize {
        self.code.len()
    }
}
