pub struct BinaryData {
    data: Vec<u8>,
}

impl BinaryData {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    
    pub fn add_byte(&mut self, byte: u8) {
        self.data.push(byte);
    }
    
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_binary_data() {
        let mut data = BinaryData::new();
        data.add_byte(0x01);
        data.add_byte(0x02);
        data.add_byte(0x03);
        
        assert_eq!(data.get_data(), &[0x01, 0x02, 0x03]);
    }
}
