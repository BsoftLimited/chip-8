use std::fmt::{ Error, Formatter, Display};
use std::ops::{ BitXor, BitOr, BitAnd };


pub fn hex(value: u16)->String{
    let mut init = value;
    let mut builder = String::new();
    while init > 0{
        let ch = format!("{}", init % 16);
        builder.insert_str(0, match ch.as_ref(){
            "10" => "A",
            "11" => "B",
            "12" => "C",
            "13" => "D",
            "14" => "E",
            "15" => "F",
            __ => ch.as_ref()
        });
        init /= 16;
    }
    return if builder.is_empty() { String::from("0") } else { builder };
}

pub struct Opcode{ code: u16 }
impl Opcode{
    pub fn new(code: u16 )->Self{ Opcode{ code } }

    pub fn x(&self)->usize{ ((self.code & 0x0f00) >> 8) as usize }
    pub fn y(&self)->usize{ ((self.code & 0x00f0) >> 4) as usize }
    pub fn nnn(&self)->u16{ self.code & 0x0fff }
    pub fn kk(&self)->u16{ self.code & 0x00ff }
    pub fn n(&self)->u16{ self.code & 0x000f }

    pub fn clear(&mut self){ self.code = 0; }

    pub fn dessemble(&self)->String{
        match self.code & 0xf000{
            0x0000 =>{
                match self.code & 0x00ff{
                    0x00e0 => return String::from("CLR"), // clear screen
                    0x00ee => return String::from("RET"), // pop stack pointer
                    _ => return format!("SYS {}", hex(self.nnn())),
                }
            },
            0x1000 => return format!("JP {}", hex(self.nnn())),      // jump to address in opcode
            0x2000 => return format!("CALL {}", hex(self.nnn())),      // jump to address in opcode,
            0x3000 => return format!("SE V{}, {}", hex(self.x() as u16), hex(self.kk())),     //skip next instruction if vx is equal to kk
            0x4000 => return format!("SNE V{}, {}", hex(self.x() as u16), hex(self.kk())),    //skip next instruction if vx is not equal to kk
            0x5000 => return format!("SE V{}, V{}", hex(self.x() as u16), hex(self.y() as u16)),   //skip next instruction if vx is equal to vy
            0x6000 => return format!("LD V{}, {}", hex(self.x() as u16), hex(self.kk())),      // set vx = kk
            0x7000 => return format!("ADD V{}, {}", hex(self.x() as u16), hex(self.kk())),        // 7XNN - add kk to vx
            0x8000 =>{
                match self.code & 0x000f{
                    0x0000 => return format!("LD V{}, V{}", hex(self.x() as u16), hex(self.y() as u16)),    // set vx to vy
                    0x0001 => return format!("OR V{}, V{}", hex(self.x() as u16), hex(self.y() as u16)),     // vx = vx or vy
                    0x0002 => return format!("AND V{}, V{}", hex(self.x() as u16), hex(self.y() as u16)),    // vx = vx and vy
                    0x0003 => return format!("XOR V{}, V{}", hex(self.x() as u16), hex(self.y() as u16)),    // vx = vx xor vy
                    0x0004 => return format!("ADD V{}, V{}", hex(self.x() as u16), hex(self.y() as u16)),    // vx = vx + vy, vf = carry
                    0x0005 => return format!("SUB V{}, V{}", hex(self.x() as u16), hex(self.y() as u16)),    // vx = vx-vy vf-not borrow if vx > vy vf = 1 else vf = 0
                    0x0006 => return format!("SHR V{}", hex(self.x() as u16)),    // SHR vx{, vy}  vx = vx shr 1
                    0x0007 => return format!("SUBN V{}, V{}", hex(self.x() as u16), hex(self.y() as u16)),   // vx = vy-vx vf-not borrow if vy > vx vf = 1 else vf = 0
                    0x000e => return format!("SHL V{}", hex(self.x() as u16)),
                    _ => return format!("{}", self.code),
                }
            },
            0x9000 => return format!("SNE V{}, V{}", hex(self.x() as u16), hex(self.y() as u16)),    // skip next instruction if vx not equal to vy
            0xa000 => return format!("LD I, {}", hex(self.nnn())),          // set i = nnn
            0xb000 => return format!("JP V0, {}", hex(self.nnn())),
            0xc000 => return format!("RND V{}, {}", hex(self.x() as u16), hex(self.kk())),       // vx = random byte and kk
            0xd000 => return format!("DRW V{}, V{}, {}", hex(self.x() as u16), hex(self.y() as u16), hex(self.kk())), //display n-byte sprite starting at memory location I at (vx/vy), set vf=collision
            0xe000 =>{ 
                match self.code & 0x000f{
                    0x000e => return format!("SKP V{}", hex(self.x() as u16)), // skip new instruction if keys with the value of vx is pressed
                    0x0001 => return format!("SKNP V{}", hex(self.x() as u16)), // skip new instruction if keys with the value of vx is not pressed
                    _ => return format!("{}", self.code),
                }
            },
            0xf000 =>{
                match self.code & 0x00ff{
                    0x0007 => return format!("LD V{}, DT", hex(self.x() as u16)), // get vx = delay timer table
                    0x000a => return format!("LD V{}, K", hex(self.x() as u16)),
                    0x0015 => return format!("LD DT, V{}", hex(self.x() as u16)),      // set delay timer = vx,
                    0x0018 => return format!("LD ST, V{}", hex(self.x() as u16)),      // set sound timer = vx
                    0x001e => return format!("ADD I, V{}", hex(self.x() as u16)),         // I = I + vx
                    0x0029 => return format!("LD F, V{}", hex(self.x() as u16)),       // set i = location of 5 bit sprite for digit vx
                    0x0033 => return format!("LD B, V{}", hex(self.x() as u16)),           // store BCD representation of vx in memory location I, I + 1, I + 2
                    0x0055 => return format!("LD [I], V{}", hex(self.x() as u16)),       // read register v0 through vx in memory starting at location I
                    0x0065 => return format!("LD V{}, [I]", hex(self.x() as u16)),      // read register v0 through vx in memory starting at location I
                    //0x0075 => self.cpu.user_reg_dump(&opcode),   // store register v0 through vx in memory starting at location I
                    //0x0085 => self.cpu.user_reg_load(&opcode),   // read register v0 through vx in memory starting at location I
                    _ => return format!("{}", self.code),
                }
            },
            _ => return format!("{}", self.code),
        }
    }
}

impl BitAnd<u16> for &Opcode{
    type Output = u16;
    
    fn bitand(self, rhs: u16) -> Self::Output {
        return self.code & rhs;
    }
}

impl BitOr<u16> for &Opcode{
    type Output = u16;
    
    fn bitor(self, rhs: u16) -> Self::Output {
        return self.code | rhs;
    }
}

impl BitXor<u16> for &Opcode{
    type Output = u16;
    
    fn bitxor(self, rhs: u16) -> Self::Output {
        return self.code ^ rhs;
    }
}

impl Display for Opcode{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>{
        let mut init = self.code;
        let mut builder = String::new();
        while init > 0{
            let ch = format!("{}", init % 16);
            builder.insert_str(0, match ch.as_ref(){
                "10" => "A",
                "11" => "B",
                "12" => "C",
                "13" => "D",
                "14" => "E",
                "15" => "F",
                __ => ch.as_ref()
            });
            init /= 16;
        }
        return write!(f, "Opcode :{}", builder);
    }
}