use std::io::Read;
use std::fs::File;

mod chip;
mod compiler;

fn main() {
    //chip::start("games/SYZYGY");
    match &mut File::open("scripts/test.bs"){
        Ok(file) =>{
            let mut init: String = String::new();
            if let Err(error) = file.read_to_string(&mut init){
                panic!("{}", error);
            }
            
            let mut paerser = compiler::Parser::new(init.as_str());
            while paerser.has_next(){
                println!("{:?}", paerser.get_next());
            }
            paerser.print_errors();
        },
        Err(error) =>{ panic!("{}", error); }
    }
}