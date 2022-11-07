mod parser;
use std::collections::HashMap;
use std::iter::Map;
use crate::chip::utils::hex;
use crate::chip::Opcode;
use crate::chip::assembler::parser::{Parser, Expression};

struct Sub{ name: String, subtype:String }

pub struct Assemblier{ map: HashMap<String, (String, Vec<Expression>)> }
impl Assemblier{
    pub fn new()->Self{
        Assemblier{ map: HashMap::new() }
    }

    pub fn init(&mut self, data: &str){
        let mut parser = Parser::new(data);

        let mut sub: Option<Sub> = None;
        let mut codes: Vec<Expression> = Vec::new();

        while parser.next_token(){
            let init = parser.get_next();
            if (matches!(init, Expression::Jump{ nemode:_ , address:_ }) || matches!(init, Expression::Opcode(_))) && sub.is_some() && sub.as_ref().unwrap().subtype.eq_ignore_ascii_case("commands"){
                codes.push(init);
            }else if matches!(init, Expression::Sprite(_)) && sub.is_some() && sub.as_ref().unwrap().subtype.eq_ignore_ascii_case("sprite"){
                codes.push(init);
            }else if let Expression::Subroutine{name, subtype} = init{
                self.insert(&mut sub, &mut codes);
                sub = Some(Sub{name, subtype});
            }
        }
        self.insert(&mut sub, &mut codes);
    }

    fn insert(&mut self, sub: &mut Option<Sub>, codes: &mut Vec<Expression>){
        if sub.is_some() && !codes.is_empty(){
            self.map.insert(sub.as_ref().unwrap().name.clone(), (sub.as_ref().unwrap().subtype.clone(), codes.to_vec()));
            codes.clear();
        }
    }

    pub fn run(&mut self)->Vec<u8>{
        let mut codes:Vec<u8> = Vec::new();
        let mut addresses: Vec<(String, u16)> = Vec::new();

        addresses.push(("start".to_owned(), 0x200));
        let mut current: u16 = 0x200 + (self.map["start"].1.len() as u16 * 2);
        for address in self.map.keys(){
            if !address.eq_ignore_ascii_case("start"){
                addresses.push((address.clone(), current));
                if self.map[address].0.eq_ignore_ascii_case("sprite"){
                    if let Expression::Sprite(sprite) = &self.map[address].1[0]{
                        current += sprite.len() as u16;
                    }
                }else if self.map[address].0.eq_ignore_ascii_case("commands"){
                    current += self.map[address].1.len() as u16 * 2;
                }
            }
        }

        let init_addr = addresses.clone(); //println!("{:?}", addresses);
        self.process("start", &mut codes, addresses.as_ref());
        for addr in init_addr{
            if !addr.0.eq_ignore_ascii_case("start"){
                self.process(addr.0.as_str(), &mut codes, addresses.as_ref());
            }
        }
        
        /*let mut index = 0;
        while index < codes.len(){
            println!("{}: {}", hex(0x200 + index as u16), hex(codes[index] as u16));
            index += 1;
        }*/
        return codes;
    }

    fn process(&mut self, addr: &str, codes: &mut Vec<u8>, addresses: &[(String, u16)] ){
        let (_subtype, exps) = &self.map[addr];
            for exp in exps{
                if let Expression::Opcode(code) = exp{
                    //let opcode = Opcode::new(code.clone());
                    //println!("{}: {}", hex(code.clone()), opcode.dessemble());
                    codes.push(((code & 0xff00) >> 8) as u8);
                    codes.push((code & 0x00ff) as u8);
                }else if let Expression::Jump{nemode, address} = exp{
                    for ad in addresses{
                        if ad.0.eq_ignore_ascii_case(address){
                            let init = nemode | ad.1;
                            //let opcode = Opcode::new(init);
                            //println!("{}: {}", hex(init), opcode.dessemble());
                            codes.push(((init & 0xff00) >> 8) as u8);
                            codes.push((init & 0x00ff) as u8);
                        }
                    }
                }else if let Expression::Sprite(data) = exp{
                    //println!("Sprite: name {:?}", data);
                    for datum in data{
                        codes.push(datum.to_owned() as u8);
                    }
                }
            }
    }
}