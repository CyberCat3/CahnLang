#[derive(Debug)]
pub struct PanickingByteBufferReader<'a> {
    bytes: &'a [u8],
    i: usize,
}

impl<'a> PanickingByteBufferReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, i: 0 }
    }

    pub fn current_index(&self) -> usize {
        self.i
    }

    pub fn is_at_end(&self) -> bool {
        self.i >= self.bytes.len()
    }

    pub fn read_u8(&mut self) -> u8 {
        let val = self.bytes[self.i];
        self.i += 1;
        val
    }

    pub fn read_u16_le(&mut self) -> u16 {
        let val = u16::from_le_bytes([self.bytes[self.i], self.bytes[self.i + 1]]);
        self.i += 2;
        val
    }

    pub fn read_u32_le(&mut self) -> u32 {
        let val = u32::from_le_bytes([
            self.bytes[self.i],
            self.bytes[self.i + 1],
            self.bytes[self.i + 2],
            self.bytes[self.i + 3],
        ]);
        self.i += 4;
        val
    }

    pub fn read_u64_le(&mut self) -> u64 {
        let val = u64::from_le_bytes([
            self.bytes[self.i],
            self.bytes[self.i + 1],
            self.bytes[self.i + 2],
            self.bytes[self.i + 3],
            self.bytes[self.i + 4],
            self.bytes[self.i + 5],
            self.bytes[self.i + 6],
            self.bytes[self.i + 7],
        ]);
        self.i += 8;
        val
    }
}
