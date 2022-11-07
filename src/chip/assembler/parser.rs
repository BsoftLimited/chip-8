use crate::chip::utils::from_hex;
use crate::chip::utils::{Lexer, Token};

const NEMONICS:[&str; 20] = [ 
    "CLR", "RET", "SYS", "CALL", "JP", "SE", "SNE", "LD", "ADD", "OR",
    "AND", "XOR", "SUB", "SHR", "SUBN", "SHL", "RND", "DRW", "SKP", "SKNP"
];

fn is_nemonic(name: &str)->bool{
    return NEMONICS.contains(&name);
}

fn v_value(code: &str)->u16{
    let mut init = code.to_uppercase();
    init.remove(0);
    return match init.as_ref(){
        "A" => 10, "B" => 11, "C" => 12,
        "D" => 13, "E" => 14, "F" => 15,
        _ => { init.parse().unwrap() }
    };
}

#[derive(Debug,Clone)]
pub enum Expression{
    Opcode(u16), Jump{ nemode: u16, address: String}, Subroutine{ subtype: String, name :String}, Sprite(Vec<u16>), None
}

pub struct Parser{ lexer:Box<Lexer>, errors: Vec<String> , current: Token }
impl Parser{
    pub fn new(data:&str)->Self{
        return Parser{ lexer: Box::new(Lexer::new(data)), errors: Vec::new() , current: Token::None };
    }
    
    pub fn next_token(&mut self)->bool{
        while self.lexer.has_next(){
            match self.lexer.get_next_token(){
                Err(error) =>{ self.errors.push(String::from(error)); }
                Ok(token) =>{
                    self.current = token;
                    return true;
                }
            }
        }
        self.current = Token::None;
        return false;
    }

    fn pop_token(&mut self)->Token{
        let init = self.current.clone();
        self.next_token();
        return init;
    }

    pub fn get_next(&mut self)->Expression{
        while !matches!(self.current, Token::None){
            if let Token::Name(name) = &self.current{
                match name.to_uppercase().as_str(){
                    "CLR"   => return Expression::Opcode(0x00e0),
                    "RET"   => return Expression::Opcode(0x00ee),
                    "SYS"   => return self.init_sys_call(0x0000),
                    "JP"    => return self.init_jp(),
                    "CALL"  => return self.init_sys_call(0x2000),
                    "SE"    => return self.init_se_sne([ 0x3000, 0x5000 ]),
                    "SNE"   => return self.init_se_sne([0x4000, 0x9000 ]),
                    "ADD"   => return self.init_add([ 0x7000, 0x8004, 0xf01e]),
                    "SKP"   => return self.init_skp_sknp_shr_shl(0xe00e),
                    "SKNP"  => return self.init_skp_sknp_shr_shl(0xe001),
                    "SHR"   => return self.init_skp_sknp_shr_shl(0x8006),
                    "SHL"   => return self.init_skp_sknp_shr_shl(0x800e),
                    "OR"    => return self.init_or_xor_sub_subn(0x8001),
                    "XOR"   => return self.init_or_xor_sub_subn(0x8003),
                    "SUB"   => return self.init_or_xor_sub_subn(0x8005),
                    "SUBN"  => return self.init_or_xor_sub_subn(0x8007),
                    "RND"   => return self.init_rnd(),
                    "DRW"   => return self.init_drw(),
                    "LD"    => return self.init_load(),
                    _ => return self.init_subroutine(),
                }
            }else if let Token::Number(_) = &self.current{
                return self.init_sprite();
            }
            let token = self.pop_token();
            self.errors.push(format!("Unexpected token: {:?}", token));
        }
        return Expression::None;
    }

    fn init_sprite(&mut self)->Expression{
        let mut init: Vec<u16> = Vec::new();
        while !matches!(self.current, Token::None){
            if let Token::Number(value) = &self.current{
                init.push(from_hex(value));
                self.next_token();
            }else if let Token::SemiColon = self.current{
                break;
            }
        }
        return Expression::Sprite(init);
    }

    fn init_subroutine(&mut self)->Expression{
        let (mut step, mut name, mut subtype): (u16, Option<String>, Option<String>) = ( 0, None, None);
        loop{
            if let Token::Name(value) = &self.current{
                if step == 0 && !is_nemonic(&value){
                    name = Some(value.clone());
                    step = 1;
                }if step == 2 && ["sprite", "commands", "text"].contains(&value.as_str()) {
                    subtype = Some(value.clone());
                    step = 3;
                }
            }else if let Token::Dot = &self.current{
                if step == 1{
                    step = 2;
                }
            }else if let Token::Colon = &self.current{
                if step == 3{
                    return Expression::Subroutine{ name: name.unwrap(), subtype: subtype.unwrap() };
                }else if step == 1 && name.as_ref().unwrap().eq_ignore_ascii_case("start"){
                    return Expression::Subroutine{ name: name.unwrap(), subtype: "commands".to_owned() };
                }
            }else if let Token::None = self.current{
                break;
            }
            self.next_token();
        }
        return Expression::None;
    }

    fn init_sys_call(&mut self, nemode: u16)->Expression{
        self.next_token();
        if let Token::Name(value) = &self.current {
            if !is_nemonic(value.as_ref()){
                return Expression::Jump{ nemode, address: value.clone() };
            }
        }
        return Expression::None;
    }

    fn init_jp(&mut self)->Expression{
        let mut step : u16 = 0;
        while self.next_token(){
            if let Token::Name(value) = &self.current {
                if step == 0  && value.to_uppercase().eq_ignore_ascii_case("V0"){
                    step = 1;
                }else if (step == 0 || step == 1) && !is_nemonic(value.as_ref()){
                    let nemode = if step == 0 { 0x1000 } else { 0xb000 };
                    return Expression::Jump{ nemode , address: value.clone() };
                }
            }
        }
        return Expression::None;
    }

    fn init_se_sne(&mut self, opcodes: [u16; 2])->Expression{
        let (mut vx, mut step) : (u16, usize) = (0, 0);
        while self.next_token(){
            if let Token::Name(value) = &self.current {
                if step == 0  && value.to_uppercase().starts_with("V"){
                    vx = v_value(value.to_uppercase().as_ref());
                    step = 1;
                }else if step == 2 && value.to_uppercase().starts_with("V"){
                    let vy = v_value(value.to_uppercase().as_ref());
                    return Expression::Opcode(opcodes[1] | (vx << 8) | (vy << 4));
                }
            }else if let Token::Coma = self.current{
                if step == 1{ step = 2; }
            }else if let Token::Number(value) = &self.current{
                if step == 2{
                    let init:u16 = from_hex(value);
                    return Expression::Opcode(opcodes[0] | (vx << 8) | init);
                }
            }
        }
        return Expression::None;
    }

    fn init_add(&mut self, opcodes: [u16; 3])->Expression{
        let (mut vx, mut step, mut is_i) : (u16, usize, bool) = (0, 0, false);
        while self.next_token(){
            if let Token::Name(value) = &self.current {
                if value.to_uppercase().starts_with("V"){
                    if step == 0{
                        vx = v_value(value.to_uppercase().as_ref());
                        step = 1;
                    }else if step == 2{
                        if is_i{
                            return Expression::Opcode(opcodes[2] | (vx << 8));
                        }
                        let vy = v_value(value.to_uppercase().as_ref());
                        return Expression::Opcode(opcodes[1] | (vx << 8) | (vy << 4));
                    }
                }if step == 0  && value.eq_ignore_ascii_case("I"){
                    step = 1;
                    is_i = true;
                }
            }else if let Token::Coma = self.current{
                if step == 1{ step = 2; }
            }else if let Token::Number(value) = &self.current{
                if step == 2 && !is_i{
                    let init:u16 = from_hex(value);
                    return Expression::Opcode(opcodes[0] | (vx << 8) | init);
                }
            }
        }
        return Expression::None;
    }

    fn init_skp_sknp_shr_shl(&mut self, opcode: u16)->Expression{
        while self.next_token(){
            if let Token::Name(value) = &self.current {
                if value.to_uppercase().starts_with("V"){
                    let vx = v_value(value.to_uppercase().as_ref());
                    return Expression::Opcode(opcode | (vx << 8));
                }
            }
        }
        return Expression::None;
    }

    fn init_or_xor_sub_subn(&mut self, opcode: u16)->Expression{
        let (mut vx, mut step) : (u16, usize) = (0, 0);
        while self.next_token(){
            if let Token::Name(value) = &self.current {
                if step == 0  && value.to_uppercase().starts_with("V"){
                    vx = v_value(value.to_uppercase().as_ref());
                    step = 1;
                }else if step == 2 && value.to_uppercase().starts_with("V"){
                    let vy = v_value(value.to_uppercase().as_ref());
                    return Expression::Opcode(opcode | (vx << 8) | (vy << 4));
                }
            }else if let Token::Coma = self.current{
                if step == 1{ step = 2; }
            }
        }
        return Expression::None;
    }

    fn init_drw(&mut self)->Expression{
        let (mut vx, mut vy, mut step) : (u16, u16, usize) = (0, 0, 0);
        while self.next_token(){
            if let Token::Name(value) = &self.current {
                if step == 0  && value.to_uppercase().starts_with("V"){
                    vx = v_value(value.to_uppercase().as_ref());
                    step = 1;
                }else if step == 2 && value.to_uppercase().starts_with("V"){
                    vy = v_value(value.to_uppercase().as_ref());
                    step = 3;
                    
                }
            }else if let Token::Coma = self.current{
                if step == 1 || step == 3{ step += 1; }
            }else if let Token::Number(value) = &self.current{
                if step == 4{
                    let init:u16 = from_hex(value);
                    return Expression::Opcode(0xd000 | (vx << 8) | (vy << 4) | init);
                }
            }
        }
        return Expression::None;
    }

    fn init_rnd(&mut self)->Expression{
        let (mut vx, mut step) : (u16, usize) = (0, 0);
        while self.next_token(){
            if let Token::Name(value) = &self.current {
                if step == 0  && value.to_uppercase().starts_with("V"){
                    vx = v_value(value.to_uppercase().as_ref());
                    step = 1;
                }
            }else if let Token::Coma = self.current{
                if step == 1{ step = 2; }
            }else if let Token::Number(value) = &self.current{
                if step == 2{
                    let init:u16 = value.parse().unwrap();
                    return Expression::Opcode(0xc000 | (vx << 8) | init);
                }
            }
        }
        return Expression::None;
    }

    fn init_load(&mut self)->Expression{
        let  mut step :  usize = 0;
        while self.next_token(){
            if let Token::Name(value) = &self.current {
                let name = value.to_uppercase();
                if name.starts_with("V"){
                    let vx = v_value(value.to_uppercase().as_ref());
                    while self.next_token(){
                        if let Token::Name(init) = &self.current {
                            if init.to_uppercase().starts_with("V") && step == 1{
                                let vy = v_value(init.to_uppercase().as_ref());
                                return Expression::Opcode(0x8000 | (vx << 8) | (vy << 4));
                            }else if init.eq_ignore_ascii_case("K") && step == 1{
                                return Expression::Opcode(0xf00a | (vx << 8));
                            }else if init.eq_ignore_ascii_case("DT") && step == 1{
                                return Expression::Opcode(0xf007 | (vx << 8));
                            }else if init.eq_ignore_ascii_case("I") && step == 2{
                                step = 3;
                            }
                        }else if let Token::Coma = self.current{
                            if step == 0{ step = 1; }
                        }else if let Token::Number(value) = &self.current{
                            if step == 1{
                                let init:u16 = from_hex(value);
                                return Expression::Opcode(0x6000 | (vx << 8) | init);
                            }
                        }else if let Token::OpenSquareBracket = self.current{
                            if step == 1{ step = 2; }
                        }else if let Token::ClosingSquareBracket = self.current{
                            if step == 3{ return Expression::Opcode(0xf065 | (vx << 8)); }
                        }
                    }
                }else if ["DT", "ST", "F", "B"].contains(&(name.as_ref())){
                    while self.next_token(){
                        if let Token::Name(init) = &self.current {
                            if step == 1 && init.to_uppercase().starts_with("V"){
                                let code: u16 = match name.to_uppercase().as_ref(){
                                    "DT" => 0xf015, "ST" => 0xf018, "F" => 0xf029, _ => 0xf033
                                };
                                let vx = v_value(init.to_uppercase().as_ref());
                                return Expression::Opcode(code | (vx << 8));
                            }
                        }else if let Token::Coma = self.current{
                            if step == 0{ step = 1; }
                        }
                    }
                }else if value.eq_ignore_ascii_case("I"){
                    while self.next_token(){
                        if let Token::Coma = self.current{
                            if step == 0{ step = 1; }
                        }else if let Token::Number(value) = &self.current{
                            if step == 1{
                                let init:u16 = from_hex(value);
                                return Expression::Opcode(0xa000 | init);
                            }
                        }else if let Token::Name(name) = &self.current{
                            if step == 1{
                                return Expression::Jump{ nemode:0xa000, address: name.clone() };
                            }
                        }
                    }   
                }
            }else if let Token::OpenSquareBracket = self.current {
                while self.next_token(){
                    if let Token::Name(value) = &self.current {
                        if step == 0  && value.eq_ignore_ascii_case("I"){
                            step = 1;
                        }else if step == 3 && value.to_uppercase().starts_with("V"){
                            let vx = v_value(value.to_uppercase().as_ref());
                            return Expression::Opcode(0xf055 | (vx << 8));
                        }
                    }else if let Token::ClosingSquareBracket = self.current{
                        if step == 1{ step = 2; }
                    }else if let Token::Coma = self.current{
                        if step == 2{ step = 3; }
                    }
                }
            }
        }
        return Expression::None;
    }
}