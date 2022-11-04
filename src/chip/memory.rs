use crate::chip::utils::fontset;

pub struct Memory{ memory: [u8; 4096] }

impl Memory{
    pub fn new()->Self { Memory{ memory: [0; 4096] }}
    pub fn clear(&mut self){
        let fontset = fontset();
        for i in 0..self.memory.len(){
            self.memory[i] = if i < fontset.len() { fontset[i] } else { 0 };
        }
    }

    pub fn load(&mut self, data: &[u8]){
        for i in 0..data.len(){
            self.memory[i + 0x200] = data[i];
        }
    }

    pub fn get(&self, index: usize)->u8{
        return self.memory[index];
    }

    pub fn save(&mut self, address:usize, data: u8){
        self.memory[address] = data;
    }
}