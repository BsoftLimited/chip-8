
use egui::Event;

pub struct KeyPad{ keys: [bool; 16] }

impl KeyPad{
    pub fn new()->Self{ KeyPad{ keys:[false; 16] } }

    pub fn reset(&mut self){
        for i in 0..self.keys.len(){
            self.keys[i] = false;
        }
    }

    pub fn get(&self, key: usize)->bool{ self.keys[key] }

    pub fn clear_key(&mut self, key: usize){ self.keys[key] = false; }

    pub fn update(&mut self, event: &Event){

    }
}