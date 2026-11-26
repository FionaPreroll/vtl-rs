pub struct Memory {
    // TODO make switchable in width based on config (e.g. 8, 16, or 64-bit)
    pub data: Vec<i64>,     // Shared memory for both program and array storage
    pub program_end: usize, // Track where the program ends and arrays start
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Memory {
            data: vec![0; size], // Initialize memory
            program_end: 0,      // Initialize the end of the program
        }
    }

    pub fn read(&self, address: usize) -> i64 {
        self.data[address]
    }

    pub fn write(&mut self, address: usize, value: i64) {
        self.data[address] = value;
    }

    // Write a line of code at a specific line number
    pub fn write_program_line(&mut self, line_number: usize, code: i64) {
        // Store code at a location indexed by line_number
        self.data[line_number] = code;
        if line_number > self.program_end {
            self.program_end = line_number;
        }
    }

    // Retrieve the code at a specific line number
    pub fn read_program_line(&self, line_number: usize) -> Option<i64> {
        if line_number <= self.program_end {
            Some(self.data[line_number])
        } else {
            None // No line found
        }
    }

    // Write to array memory, beyond program_end
    pub fn write_array(&mut self, index: usize, value: i64) {
        let array_address = self.program_end + 1 + index;
        self.data[array_address] = value;
    }

    // Read from array memory
    pub fn read_array(&self, index: usize) -> Option<i64> {
        let array_address = self.program_end + 1 + index;
        if array_address < self.data.len() {
            Some(self.data[array_address])
        } else {
            None
        }
    }
}
