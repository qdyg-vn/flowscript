use crate::value::LightValue;

const CAPACITY: usize = 262144;

#[derive(Default, Debug)]
pub struct Memory {
    pub permanent_space: Vec<u8>,
    pub functions: Vec<u8>,
    pub from_space: Vec<u8>,
    to_space: Vec<u8>,
    free_pointer: usize,
    scan_pointer: usize,
    allocated_bytes: usize,
    garbage_collect_threshold: usize,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            from_space: Vec::with_capacity(CAPACITY),
            to_space: Vec::with_capacity(CAPACITY),
            garbage_collect_threshold: CAPACITY,
            ..Self::default()
        }
    }

    pub fn allocate(&mut self, size: usize, stack: &mut [LightValue]) {
        if self.from_space.len() + size >= self.garbage_collect_threshold {
            self.run_garbage_collection(stack);
            if self.from_space.len() + size >= self.garbage_collect_threshold {
                let memory_needed = std::cmp::max(self.from_space.capacity() + (self.from_space.capacity() >> 1), (self.from_space.len() + size) * 100 / 75) - self.from_space.capacity();
                self.from_space.reserve(memory_needed);
                self.to_space.reserve(memory_needed);
                self.garbage_collect_threshold = self.from_space.capacity() * 75 / 100
            }
        }
        self.allocated_bytes = self.from_space.len();
        self.from_space.resize(self.from_space.len() + size, 0);
    }

    pub fn push_to_heap(&mut self, value: &[u8]) {
        self.from_space[self.allocated_bytes..self.allocated_bytes + value.len()].copy_from_slice(value);
        self.allocated_bytes += value.len();
    }

    fn run_garbage_collection(&mut self, stack: &mut [LightValue]) {
        let (pointers, indices) = self.collect_live_pointers(stack);
        let new_pointers = self.garbage_collect(pointers);
        for (index, &value_index) in indices.iter().enumerate() {
            match &mut stack[value_index] {
                LightValue::StringHeapPointer(pointer) | LightValue::ArrayPointer(pointer) => {
                    *pointer = new_pointers[index] as u32;
                },
                _ => unreachable!(),
            };
        }
    }

    fn collect_live_pointers(&self, stack: &[LightValue]) -> (Vec<usize>, Vec<usize>) {
        let mut pointers = Vec::new();
        let mut indices = Vec::new();
        for (value_index, value) in stack.iter().enumerate() {
            match value {
                LightValue::StringHeapPointer(index) | LightValue::ArrayPointer(index) => {
                    pointers.push(*index as usize);
                    indices.push(value_index);
                },
                _ => ()
            }
        }
        (pointers, indices)
    }

    fn garbage_collect(&mut self, pointers: Vec<usize>) -> Vec<usize> {
        let mut new_pointers = Vec::with_capacity(pointers.len());
        for pointer in pointers {
            new_pointers.push(self.free_pointer);
            let length = u64::from_le_bytes(self.from_space[pointer..pointer + 8].try_into().unwrap());
            self.to_space.extend_from_slice(&self.from_space[pointer..pointer + 8 + length as usize]);
            self.free_pointer += 8 + length as usize;
        };
        std::mem::swap(&mut self.from_space, &mut self.to_space);
        self.allocated_bytes = self.free_pointer;
        self.free_pointer = 0;
        new_pointers
    }
}
