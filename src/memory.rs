#[derive(Default)]
pub struct Memory {
    pub permanent_space: Vec<u8>,
    from_space: Vec<u8>,
    to_space: Vec<u8>,
    free_pointer: usize,
    scan_pointer: usize,
}

impl Memory {

}
