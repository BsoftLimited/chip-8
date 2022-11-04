mod parser;
use crate::chip::utils::hex;
use crate::chip::Opcode;
use crate::chip::assembler::parser::{Parser, Expression};

pub struct Assemblier{ parser: Parser }
impl Assemblier{
    pub fn new(data: &str)->Self{
        Assemblier{ parser: Parser::new(data) }
    }

    pub fn run(&mut self)->Vec<u8>{
        let mut codes:Vec<u8> = Vec::new();
        while self.parser.next_token(){
            let init = self.parser.get_next();
            if let Expression::Opcode(code) = init{
                let opcode = Opcode::new(code);
                println!("{}: {}", hex(code), opcode.dessemble());
                codes.push(((code & 0xff00) >> 8) as u8);
                codes.push((code & 0x00ff) as u8);
            }else if let Expression::Subroutine(name) = init{
                println!("Subroutine: {}", name);
            }
        }
        return codes;
    }
}