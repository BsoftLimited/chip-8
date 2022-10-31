use rand::Rng;
use crate::chip::KeyPad;
use crate::chip::Memory;
use crate::chip::Opcode;
use crate::chip::Screen;

pub struct Delay{ sound: u8, timer: u8 }
impl Delay{
    fn new()->Self{ Delay{ sound: 0, timer: 0 } }
    fn update(&mut self){
        if self.sound > 0{ self.sound -= 1; }
        if self.timer > 0{ self.timer -= 1; }
    }

    pub fn reset(&mut self){
        self.sound = 0;
        self.timer = 0;
    }

    pub fn set_sound(&mut self, sound: u8){ self.sound = sound; }
    pub fn set_timer(&mut self, timer: u8){ self.timer = timer; }

    pub fn get_sound(&self)-> u8{ self.sound }
    pub fn get_timer(&self)-> u8{ self.timer }
}

pub struct Registers{ v:[u8; 16], r:[u8; 8] }
impl Registers{
    fn new()->Self{ Registers{ v: [0; 16], r: [0; 8]} }

    fn reset(&mut self){
        for i in 0..self.v.len(){ self.v[i] = 0; }
        for i in 0..self.r.len(){ self.r[i] = 0; }
    }
}

pub struct CPU{
    stack: Vec<u16>,            // stack pointer
    registers: Registers,       // 15 8-bit Registers v0-v15 and v16 carry flag 
    i: u16,                     // 16-bit index register
    pc: u16,                    // 16-bit program counter
    delay: Delay
}

impl CPU{
    pub fn new()->Self{
        return CPU{ stack: Vec::new(), registers: Registers::new(), i:0, pc:0x200, delay: Delay::new() }
    }

    pub fn reset(&mut self){
        self.pc = 0x200;    // clear program counter
        self.i = 0;         // reset current index register
        self.stack.clear(); // reset stack pointer

        // clear registers
        self.registers.reset();

        self.delay.reset();
    }

    pub fn fetch(&self, memory: &Memory)->u16{
        return ((memory.get(self.pc as usize) as u16) << 8) | (memory.get((self.pc + 1) as usize) as u16);
    }

    pub fn clear_screen(&mut self, screen: &mut Screen){
        screen.clear();
        self.pc += 2;
    }

    pub fn call(&mut self, opcode: &Opcode){
        self.stack.push(self.pc + 2);
        self.pc = opcode.nnn();
    }
    
    pub fn ret(&mut self){ self.pc = self.stack.pop().unwrap(); }
    pub fn jump(&mut self, opcode: &Opcode){ self.pc = opcode.nnn(); }

    pub fn if_eq(&mut self, opcode: &Opcode){
        let vx = self.registers.v[opcode.x()] as u16;
        self.pc += if vx == opcode.kk() { 4 } else { 2 };
    }
    
    pub fn if_not_eq(&mut self, opcode: &Opcode){
        let vx = self.registers.v[opcode.x()] as u16;
        self.pc += if vx != opcode.kk() { 4 } else { 2 };
    }
    
    pub fn if_eq_reg(&mut self, opcode: &Opcode){
        let vx = self.registers.v[opcode.x()];
        let vy = self.registers.v[opcode.y()];
        self.pc += if vx == vy { 4 } else { 2 };
    }

    pub fn set(&mut self, opcode: &Opcode){
        self.registers.v[opcode.x()] = opcode.kk() as u8;
        self.pc += 2;
    }
    
    pub fn add(&mut self, opcode: &Opcode){
        let init = self.registers.v[opcode.x()] as u16 + opcode.kk();
		self.registers.v[opcode.x()] = (if init >= 256 { init - 256 } else { init }) as u8;
        self.pc += 2;
    }
    
    pub fn assign(&mut self, opcode: &Opcode){
        self.registers.v[opcode.x()] = self.registers.v[opcode.y()];
        self.pc += 2;
    }
    
    pub fn bit_or(&mut self, opcode: &Opcode){
        self.registers.v[opcode.x()] |= self.registers.v[opcode.y()];
        self.pc += 2;
    }
    
    pub fn bit_and(&mut self, opcode: &Opcode){
        self.registers.v[opcode.x()] &= self.registers.v[opcode.y()];
        self.pc += 2;
    }
    
    pub fn bit_xor(&mut self, opcode: &Opcode){
        self.registers.v[opcode.x()] ^= self.registers.v[opcode.y()];
        self.pc += 2;
    }
    
    pub fn add_reg(&mut self, opcode: &Opcode){
        let x = opcode.x();
        let y = opcode.y();
        let init = self.registers.v[x] as u16 + self.registers.v[y] as u16;
        self.registers.v[0xf] = ((init & 0xff00) >> 8) as u8; 
        self.registers.v[x] = (init & 0x00ff) as u8; 
        self.pc += 2;
    }
    
    pub fn sub_reg(&mut self, opcode: &Opcode){
        let x = opcode.x();
        let y = opcode.y();

        self.registers.v[0xf] = if self.registers.v[x] > self.registers.v[y]{ 1 } else { 0 };
        self.registers.v[x] = ((self.registers.v[x] as u16 - self.registers.v[y] as u16) & 0x00ff) as u8;
        self.pc += 2;
    }

    pub fn shift_right(&mut self, opcode: &Opcode){
        let x = opcode.x();
       
        self.registers.v[0xf] = self.registers.v[x] & 0x01;
        self.registers.v[x] = self.registers.v[x] >> 1;
        self.pc += 2;
    }
    
    pub fn sub_copy(&mut self, opcode: &Opcode){
        let x = opcode.x();
        let y = opcode.y();

        self.registers.v[0xf] = if self.registers.v[y] > self.registers.v[x]{ 1 } else { 0 };
        self.registers.v[x] = self.registers.v[y] - self.registers.v[x];
        self.pc += 2;
    }
    
    pub fn shift_left(&mut self, opcode: &Opcode){
        let x = opcode.x();
        /*if shift_quirk{
            let res = self.registers.v[x] << 1;
            self.registers.v[x] = res;
            self.registers.v[0xf] = (self.registers.v[x] & 0x80) >> 7;
        }else {
            let y = opcode.y();

            let res = self.registers.v[y] << 1;
            self.registers.v[x] = res;
            self.registers.v[0xf] = (self.registers.v[y] & 0x80) >> 7;
            self.registers.v[y] = res;
        }*/
        
        self.registers.v[0xf] = self.registers.v[x] >> 7;
        self.registers.v[x] = self.registers.v[x] << 1;
        self.pc += 2;
    }

    pub fn if_not_eq_reg(&mut self, opcode: &Opcode){
        let vx = self.registers.v[opcode.x()];
        let vy = self.registers.v[opcode.y()];
        self.pc += if vx != vy { 4 } else { 2 };
    }
    
    pub fn set_i(&mut self, opcode: &Opcode){
        self.i = opcode.nnn();  // set i = nnn
        self.pc += 2;
    }
    
    pub fn jump_v0(&mut self, opcode: &Opcode){
        self.pc = opcode.nnn() + (self.registers.v[0] as u16);
    }
    
    pub fn and_rand(&mut self, opcode: &Opcode){
        let mut rng = rand::thread_rng();
        let rand: u16 = rng.gen_range(0..256);
        self.registers.v[opcode.x()] = (((opcode.kk()) + rand) & 0xff) as u8;
        self.pc += 2;
    }

    pub fn get_delay(&mut self, opcode: &Opcode){
        self.registers.v[opcode.x()] = self.delay.get_timer();
        self.pc += 2;
    }
    
    pub fn set_delay(&mut self, opcode: &Opcode){
        self.delay.set_timer(self.registers.v[opcode.x()]);
        self.pc += 2;
    }
    
    pub fn set_sound(&mut self, opcode: &Opcode){
        self.delay.set_sound(self.registers.v[opcode.x()]);
        self.pc += 2;
    }

    pub fn add_i(&mut self, opcode: &Opcode){
        self.i += self.registers.v[opcode.x()] as u16;
        self.pc += 2;
    }
    
    pub fn get_font(&mut self, opcode: &Opcode){
        self.i = (self.registers.v[opcode.x()] as u16) * 5;
        self.pc += 2;
    }
    
    pub fn get_font16(&mut self, opcode: &Opcode){
        self.i = (self.registers.v[opcode.x()] as u16) * 10 + 80;
        self.pc += 2;
    }

    pub fn bcd(&mut self, opcode: &Opcode, memory: &mut Memory){
        let vx = self.registers.v[opcode.x()];
        memory.save(self.i as usize, vx / 100);
        memory.save((self.i + 1) as usize, (vx % 100) / 10);
        memory.save((self.i + 2) as usize, vx % 10);
        self.pc += 2
    }
    
    pub fn reg_dump(&mut self, opcode: &Opcode, memory: &mut Memory){
        let vx = opcode.x();
        for i in 0..vx{
            memory.save((self.i) as usize, self.registers.v[i as usize]); 
            self.i += 1;
        }
        //self.i += vx as u16;
        self.pc += 2;
    }
    
    pub fn reg_load(&mut self, opcode: &Opcode, memory: &Memory){
        let vx = opcode.x();
        for i in 0..vx{
            self.registers.v[i as usize] = memory.get((self.i) as usize); 
            self.i += 1;
        }
        //self.i += vx as u16;
        self.pc += 2;
    }

    /*pub fn user_reg_dump(&mut self, opcode: &Opcode){
        let vx = opcode.x();
        for i in 0..vx + 1{
            self.r[i as usize] = self.registers.v[i as usize]; 
            self.i += 1;
        }
        self.pc += 2;
    }
    
    pub fn user_reg_load(&mut self, opcode: &Opcode){
        let vx = opcode.x();
        for i in 0..vx + 1{
            self.registers.v[i as usize] = self.r[i as usize]; 
            self.i += 1;
        }
        self.pc += 2;
    }*/

    pub fn if_key(&mut self, opcode: &Opcode, keys: &KeyPad){
        self.pc = if keys.get(self.registers.v[opcode.x()]as usize){ 4 } else { 2 };
    }
    
    pub fn if_not_key(&mut self, opcode: &Opcode, keys: &KeyPad){
        self.pc = if !keys.get(self.registers.v[opcode.x()] as usize){ 4 } else { 2 };
    }

    pub fn update(&mut self){
        if self.delay.get_sound() == 1 {
            println!("BEEP!");
        }
        self.delay.update();
    }

    pub fn wait_key(&mut self, opcode: &Opcode, keys: &mut KeyPad){
        for i in 0..0x10{
            if keys.get(i){
                self.registers.v[(opcode & 0x0f00) as usize] = i as u8;
                keys.clear_key(i);
                self.pc += 2; 
            }
        } 
    }

    // Function that draw in Xor mode
    /*pub fn draw(&mut self, opcode: &Opcode, screen: &mut Screen, memory: &Memory){
        //display n-byte sprite starting at memory location I at (vx/vy), set vf=collision
        let height = opcode.n();
        let x = opcode.x();
        let y = opcode.y();
        let mut sprite: Vec<u16> = Vec::new();
        if height == 0 && screen.is_extended(){
            for i in (0..32).step_by(2){
                let left:u16 = (memory.get(self.i as usize + i) as u16) << 8;
                let right: u16 = memory.get(self.i as usize + i + 1) as u16;
                sprite.push(left | right);
            }
            self.registers.v[0xf] = screen.schip_draw(self.registers.v[x] as u16, self.registers.v[y] as u16, sprite);
        }else{
            for i in 0..height{
                sprite.push(memory.get(self.i as usize + i as usize) as u16); 
            }
            self.registers.v[0xf] = screen.draw(self.registers.v[x] as u16 , self.registers.v[y] as u16, height, sprite);
        }
        self.pc += 2;
    }*/

    pub fn invalid(&mut self, opcode: &Opcode){
        println!("unknown or invalid command: {}", opcode);
    }

    pub fn draw(&mut self, opcode: &Opcode, screen: &mut Screen, memory: &Memory){
        let x = opcode.x();
        let y = opcode.y();

        let mut sprite: Vec<u16> = Vec::new();
        for i in 0..opcode.n(){
            sprite.push(memory.get(self.i as usize + i as usize) as u16); 
        }
        self.registers.v[0xf] = screen.draw(self.registers.v[x] as u16 , self.registers.v[y] as u16, sprite);
        self.pc += 2;
    }
}
