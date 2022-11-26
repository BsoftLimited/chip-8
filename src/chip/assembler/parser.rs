use crate::chip::utils::{Lexer, Token, TokenType, Position};

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
    Opcode(u16), Jump{ nemode: u16, address: String}, Subroutine{ subtype: String, name :String}, Sprite(Vec<u16>), End
}

pub struct Parser{ lexer:Box<Lexer> , current: TokenType, position: Position }
impl Parser{
    pub fn new(data:&str)->Self{
        let token = Token::none();
        return Parser{ lexer: Box::new(Lexer::new(data)) , current: token.tokentype(), position: token.position() };
    }
    
    pub fn next_token(&mut self)->Result<bool, String>{
        while self.lexer.has_next(){
            match self.lexer.get_next_token(){
                Err(error) =>{ return Err(error); },
                Ok(token) =>{
                    self.current = token.tokentype();
                    self.position = token.position();
                    return Ok(true);
                }
            }
        }
        self.current = TokenType::None;
        return Ok(false);
    }

    pub fn get_next(&mut self)->Result<Expression, String>{
        if !matches!(self.current, TokenType::None){
            if let TokenType::Name(name) = &self.current{
                match name.to_uppercase().as_str(){
                    "CLR"   => return Ok(Expression::Opcode(0x00e0)),
                    "RET"   => return Ok(Expression::Opcode(0x00ee)),
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
                    _ => return self.init_subroutine(name.to_owned()),
                }
            }else if let TokenType::Number(_) = self.current{
                return self.init_sprite();
            }
            return Err(format!("Unexpected token: {:?} at {:?}", self.current, self.position));
        }
        return Ok(Expression::End);
    }

    fn init_sprite(&mut self)->Result<Expression, String>{
        let mut init: Vec<u16> = Vec::new();
        while !matches!(self.current, TokenType::None){
            if let TokenType::Number(value) = self.current{
                init.push(value);
                match self.next_token(){
                    Err(error) => return Err(error),
                    Ok(sub_has_next) =>{
                        if !sub_has_next{
                            return Err(format!("unexpected end of token at {:?}, expecting a number or semicolon(;)", self.position));
                        }
                    }
                }
            }else if let TokenType::SemiColon = self.current{
                break;
            }
        }
        return Ok(Expression::Sprite(init));
    }

    fn init_subroutine(&mut self, name: String)->Result<Expression, String>{
        let (mut step, mut subtype): (u16, Option<String>) = ( 0, None);
        match self.next_token(){
            Err(error) => return Err(error),
            Ok(has_next) =>{
                if has_next{
                    while matches!(self.current, TokenType::None){
                        if let TokenType::Name(value) = &self.current{
                            if step == 1 && ["sprite", "commands", "text"].contains(&value.as_str()) {
                                subtype = Some(value.clone());
                                step = 2;
                            }
                        }else if let TokenType::Dot = self.current{
                            if step == 0{
                                step = 1;
                            }
                        }else if let TokenType::Colon = self.current{
                            if step == 2{
                                return Ok(Expression::Subroutine{ name, subtype: subtype.unwrap() });
                            }else if step == 0 && name.eq_ignore_ascii_case("start"){
                                return Ok(Expression::Subroutine{ name, subtype: "commands".to_owned() });
                            }
                        }else if let TokenType::None = self.current{
                            break;
                        }
                        
                        match self.next_token(){
                            Err(error) => return Err(error),
                            Ok(sub_has_next) =>{
                                if !sub_has_next{
                                    return Err(format!("unexpected end of token at {:?}", self.position));
                                }
                            }
                        }
                    }
                }
            }
        }
        return Err(String::new());
    }

    fn init_sys_call(&mut self, nemode: u16)->Result<Expression, String>{
        match self.next_token(){
            Err(error) => return Err(error),
            Ok(has_next) =>{
                if has_next{
                    if let TokenType::Name(value) = &self.current{
                        if !is_nemonic(value.as_ref()){
                            return Ok(Expression::Jump{ nemode, address: value.clone() });
                        }
                        return Err(format!("token:{:?} as {:?} is a reserved word, try another word",self.current, self.position));
                    }
                }else{
                    return Err(format!("unexpected end of token after {:?}", self.position));
                }
            }
        }
        return Ok(Expression::End);
    }

    fn init_jp(&mut self)->Result<Expression, String>{
        let mut step : u16 = 0;
        loop{
            let next = self.next_token();
            if let Err(error) = next{
                return Err(error)
            }else if let Ok(has_next) = next{
                if has_next{
                    if let TokenType::Name(value) = &self.current{
                        if step == 0  && value.to_uppercase().eq_ignore_ascii_case("V0"){
                            step = 1;
                        }else if (step == 0 || step == 1) && !is_nemonic(value.as_ref()){
                            let nemode = if step == 0 { 0x1000 } else { 0xb000 };
                            return Ok(Expression::Jump{ nemode , address: value.clone() });
                        }
                    }
                }
            }else{
                break;
            }
        }
        return Ok(Expression::End);
    }

    fn init_se_sne(&mut self, opcodes: [u16; 2])->Result<Expression, String>{
        let (mut vx, mut step) : (u16, usize) = (0, 0);
        loop{
            let next = self.next_token();
            if let Err(error) = next{
                return Err(error)
            }else if let Ok(has_next) = next{
                if has_next{
                    if let TokenType::Name(value) = &self.current{
                        if step == 0  && value.to_uppercase().starts_with("V"){
                            vx = v_value(value.to_uppercase().as_ref());
                            step = 1;
                        }else if step == 2 && value.to_uppercase().starts_with("V"){
                            let vy = v_value(value.to_uppercase().as_ref());
                            return Ok(Expression::Opcode(opcodes[1] | (vx << 8) | (vy << 4)));
                        }
                    }else if let TokenType::Coma = self.current{
                        if step == 1{ step = 2; }
                    }else if let TokenType::Number(value) = self.current{
                        if step == 2{
                            return Ok(Expression::Opcode(opcodes[0] | (vx << 8) | value));
                        }
                    }
                }
            }else{
                break;
            }
        }
        return Ok(Expression::End);
    }

    fn init_add(&mut self, opcodes: [u16; 3])->Result<Expression, String>{
        let (mut vx, mut step, mut is_i) : (u16, usize, bool) = (0, 0, false);
        loop{
            let next = self.next_token();
            if let Err(error) = next{
                return Err(error)
            }else if let Ok(has_next) = next{
                if has_next{
                    if let TokenType::Name(value) = &self.current{
                        if value.to_uppercase().starts_with("V"){
                            if step == 0{
                                vx = v_value(value.to_uppercase().as_ref());
                                step = 1;
                            }else if step == 2{
                                if is_i{
                                    return Ok(Expression::Opcode(opcodes[2] | (vx << 8)));
                                }
                                let vy = v_value(value.to_uppercase().as_ref());
                                return Ok(Expression::Opcode(opcodes[1] | (vx << 8) | (vy << 4)));
                            }
                        }if step == 0  && value.eq_ignore_ascii_case("I"){
                            step = 1;
                            is_i = true;
                        }
                    }else if let TokenType::Coma = self.current{
                        if step == 1{ step = 2; }
                    }else if let TokenType::Number(value) = self.current{
                        if step == 2 && !is_i{
                            return Ok(Expression::Opcode(opcodes[0] | (vx << 8) | value));
                        }
                    }
                }
            }else{
                break;
            }
        }
        return Ok(Expression::End);
    }

    fn init_skp_sknp_shr_shl(&mut self, opcode: u16)->Result<Expression, String>{
        loop{
            let next = self.next_token();
            if let Err(error) = next{
                return Err(error)
            }else if let Ok(has_next) = next{
                if has_next{
                    if let TokenType::Name(value) = &self.current{
                        if value.to_uppercase().starts_with("V"){
                            let vx = v_value(value.to_uppercase().as_ref());
                            return Ok(Expression::Opcode(opcode | (vx << 8)));
                        }
                    }
                }
            }else{
                break;
            }
        }
        return Ok(Expression::End);
    }

    fn init_or_xor_sub_subn(&mut self, opcode: u16)->Result<Expression, String>{
        let (mut vx, mut step) : (u16, usize) = (0, 0);
        loop{
            let next = self.next_token();
            if let Err(error) = next{
                return Err(error)
            }else if let Ok(has_next) = next{
                if has_next{
                    if let TokenType::Name(value) = &self.current{
                        if step == 0  && value.to_uppercase().starts_with("V"){
                            vx = v_value(value.to_uppercase().as_ref());
                            step = 1;
                        }else if step == 2 && value.to_uppercase().starts_with("V"){
                            let vy = v_value(value.to_uppercase().as_ref());
                            return Ok(Expression::Opcode(opcode | (vx << 8) | (vy << 4)));
                        }
                    }else if let TokenType::Coma = self.current{
                        if step == 1{ step = 2; }
                    }
                }
            }else{
                break;
            }
        }
        return Ok(Expression::End);
    }

    fn init_drw(&mut self)->Result<Expression, String>{
        let (mut vx, mut vy, mut step) : (u16, u16, usize) = (0, 0, 0);
        loop{
            let next = self.next_token();
            if let Err(error) = next{
                return Err(error)
            }else if let Ok(has_next) = next{
                if has_next{
                    if let TokenType::Name(value) = &self.current{
                        if step == 0  && value.to_uppercase().starts_with("V"){
                            vx = v_value(value.to_uppercase().as_ref());
                            step = 1;
                        }else if step == 2 && value.to_uppercase().starts_with("V"){
                            vy = v_value(value.to_uppercase().as_ref());
                            step = 3;
                            
                        }
                    }else if let TokenType::Coma = self.current{
                        if step == 1 || step == 3{ step += 1; }
                    }else if let TokenType::Number(value) = self.current{
                        if step == 4{
                            return Ok(Expression::Opcode(0xd000 | (vx << 8) | (vy << 4) | value));
                        }
                    }
                }
            }else{
                break;
            }
        }
        return Ok(Expression::End);
    }

    fn init_rnd(&mut self)->Result<Expression, String>{
        let (mut vx, mut step) : (u16, usize) = (0, 0);
        loop{
            let next = self.next_token();
            if let Err(error) = next{
                return Err(error)
            }else if let Ok(has_next) = next{
                if has_next{
                    if let TokenType::Name(value) = &self.current{
                        if step == 0  && value.to_uppercase().starts_with("V"){
                            vx = v_value(value.to_uppercase().as_ref());
                            step = 1;
                        }
                    }else if let TokenType::Coma = self.current{
                        if step == 1{ step = 2; }
                    }else if let TokenType::Number(value) = self.current{
                        if step == 2{
                            return Ok(Expression::Opcode(0xc000 | (vx << 8) | value));
                        }
                    }
                }
            }else{
                break;
            }
        }
        return Ok(Expression::End);
    }

    fn init_load(&mut self)->Result<Expression, String>{
        let  mut step :  usize = 0;
        loop{
            let next = self.next_token();
            if let Err(error) = next{
                return Err(error)
            }else if let Ok(has_next) = next{
                if has_next{
                    if let TokenType::Name(value) = &self.current{
                        let name = value.to_uppercase();
                        if name.starts_with("V"){
                            let vx = v_value(value.to_uppercase().as_ref());
                            loop{
                                let sub_next = self.next_token();
                                if let Err(error) = sub_next{
                                    return Err(error)
                                }else if let Ok(sub_has_next) = sub_next{
                                    if sub_has_next{
                                        if let TokenType::Name(init) = &self.current{
                                            if init.to_uppercase().starts_with("V") && step == 1{
                                                let vy = v_value(init.to_uppercase().as_ref());
                                                return Ok(Expression::Opcode(0x8000 | (vx << 8) | (vy << 4)));
                                            }else if init.eq_ignore_ascii_case("K") && step == 1{
                                                return Ok(Expression::Opcode(0xf00a | (vx << 8)));
                                            }else if init.eq_ignore_ascii_case("DT") && step == 1{
                                                return Ok(Expression::Opcode(0xf007 | (vx << 8)));
                                            }else if init.eq_ignore_ascii_case("I") && step == 2{
                                                step = 3;
                                            }
                                        }else if let TokenType::Coma = self.current{
                                            if step == 0{ step = 1; }
                                        }else if let TokenType::Number(value) = self.current{
                                            if step == 1{
                                                return Ok(Expression::Opcode(0x6000 | (vx << 8) | value));
                                            }
                                        }else if let TokenType::OpenSquareBracket = self.current{
                                            if step == 1{ step = 2; }
                                        }else if let TokenType::ClosingSquareBracket = self.current{
                                            if step == 3{ return Ok(Expression::Opcode(0xf065 | (vx << 8))); }
                                        }
                                    }
                                }else{
                                    break;
                                }
                            }
                        }else if ["DT", "ST", "F", "B"].contains(&(name.as_ref())){
                            loop{
                                let sub_next = self.next_token();
                                if let Err(error) = sub_next{
                                    return Err(error)
                                }else if let Ok(sub_has_next) = sub_next{
                                    if sub_has_next{
                                        if let TokenType::Name(init) = &self.current{
                                            if step == 1 && init.to_uppercase().starts_with("V"){
                                                let code: u16 = match name.to_uppercase().as_ref(){
                                                    "DT" => 0xf015, "ST" => 0xf018, "F" => 0xf029, _ => 0xf033
                                                };
                                                let vx = v_value(init.to_uppercase().as_ref());
                                                return Ok(Expression::Opcode(code | (vx << 8)));
                                            }
                                        }else if let TokenType::Coma = self.current{
                                            if step == 0{ step = 1; }
                                        }
                                    }
                                }else{
                                    break;
                                }
                            }
                        }else if value.eq_ignore_ascii_case("I"){
                            loop{
                                let sub_next = self.next_token();
                                if let Err(error) = sub_next{
                                    return Err(error)
                                }else if let Ok(sub_has_next) = sub_next{
                                    if sub_has_next{
                                        if let TokenType::Coma = self.current{
                                            if step == 0{ step = 1; }
                                        }else if let TokenType::Number(value) = self.current{
                                            if step == 1{
                                                return Ok(Expression::Opcode(0xa000 | value));
                                            }
                                        }else if let TokenType::Name(name) = &self.current{
                                            if step == 1{
                                                return Ok(Expression::Jump{ nemode:0xa000, address: name.clone() });
                                            }
                                        }
                                    }
                                }else{
                                    break;
                                }
                            }  
                        }
                    }else if let TokenType::OpenSquareBracket = self.current{
                        loop{
                            let sub_next = self.next_token();
                            if let Err(error) = sub_next{
                                return Err(error)
                            }else if let Ok(sub_has_next) = sub_next{
                                if sub_has_next{
                                    if let TokenType::Name(value) = &self.current{
                                        if step == 0  && value.eq_ignore_ascii_case("I"){
                                            step = 1;
                                        }else if step == 3 && value.to_uppercase().starts_with("V"){
                                            let vx = v_value(value.to_uppercase().as_ref());
                                            return Ok(Expression::Opcode(0xf055 | (vx << 8)));
                                        }
                                    }else if let TokenType::ClosingSquareBracket = self.current{
                                        if step == 1{ step = 2; }
                                    }else if let TokenType::Coma = self.current{
                                        if step == 2{ step = 3; }
                                    }
                                }
                            }else{
                                break;
                            }
                        }
                    }
                }
            }else{
                break;
            }
        }
        return Ok(Expression::End);
    }
}