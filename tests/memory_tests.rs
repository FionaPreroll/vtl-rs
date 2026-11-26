#[cfg(test)]
#[allow(unused_mut)]
mod memory_tests {
    use vtl_rs::memory::Memory;

    #[test]
    fn test_memory_read_write() {
        let mut memory = Memory::new(1024);
        memory.write(100, 42);
        assert_eq!(memory.read(100), 42);
        memory.write(100, 23);
        assert_eq!(memory.read(100), 23);
        memory.write(100, 0);
        assert_eq!(memory.read(100), 0);
    }

    #[test]
    fn test_memory_bounds() {
        let mut memory = Memory::new(1024);
        assert_eq!(memory.read(1023), 0); // Valid upper bound
                                          // Potentially test for panic if going beyond bounds
    }
}
