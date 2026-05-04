#[derive(Default)]
pub struct Memory {
    pub permanent_space: Vec<u8>,
    pub from_space: Vec<u8>,
    to_space: Vec<u8>,
    free_pointer: usize,
    scan_pointer: usize,
}

impl Memory {
    pub fn garbage_collect(&mut self, pointers: Vec<usize>) -> Vec<usize> {
        let mut new_index = Vec::with_capacity(pointers.len());
        for pointer in pointers {
            new_index.push(self.free_pointer);
            let length = u64::from_le_bytes(self.from_space[pointer..pointer + 8].try_into().unwrap());
            self.to_space[self.free_pointer..self.free_pointer + 8 + length as usize].copy_from_slice(&self.from_space[pointer..pointer + 8 + length as usize]);
            self.free_pointer += 8 + length as usize;
        };
        std::mem::swap(&mut self.from_space, &mut self.to_space);
        self.free_pointer = 0;
        new_index
    }
}
