use crate::value::LightValue;

const CAPACITY: usize = 262144;

pub struct Memory {
    pub permanent_space: Vec<u8>,
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
            permanent_space: Vec::new(),
            from_space: Vec::with_capacity(CAPACITY),
            to_space: Vec::with_capacity(CAPACITY),
            free_pointer: 0,
            scan_pointer: 0,
            allocated_bytes: 0,
            garbage_collect_threshold: CAPACITY,
        }
    }

    pub fn push_to_heap(&mut self, value: &[u8], stack: &mut [LightValue]) {
        let size = value.len();
        if self.allocated_bytes + size >= self.garbage_collect_threshold {
            self.run_garbage_collection(stack);
            while self.allocated_bytes + size >= self.garbage_collect_threshold {
                self.from_space.reserve(self.from_space.len() * 2);
                self.to_space.reserve(self.to_space.len() * 2);
            }
            self.garbage_collect_threshold = (self.allocated_bytes + size) * 2
        }
        self.from_space.extend_from_slice(value);
        self.allocated_bytes += size
    }

    fn run_garbage_collection(&mut self, stack: &mut [LightValue]) {
        let (pointers, indices) = self.collect_live_pointers(stack);
        let new_pointers = self.garbage_collect(pointers);
        for (index, &value_index) in indices.iter().enumerate() {
            match &mut stack[value_index] {
                LightValue::StringHeapPointer(pointer) | LightValue::ArrayHeapPointer(pointer) => {
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
                LightValue::StringHeapPointer(index) | LightValue::ArrayHeapPointer(index) => {
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
            self.to_space[self.free_pointer..self.free_pointer + 8 + length as usize].copy_from_slice(&self.from_space[pointer..pointer + 8 + length as usize]);
            self.free_pointer += 8 + length as usize;
        };
        std::mem::swap(&mut self.from_space, &mut self.to_space);
        self.allocated_bytes = self.free_pointer;
        self.free_pointer = 0;
        new_pointers
    }
}
