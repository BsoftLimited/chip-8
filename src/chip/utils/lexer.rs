use crate::chip::utils::{ Character, Token};


pub struct Lexer{ index:usize, current:Character, data: String, to_newline: bool, column: usize, row:usize }
impl Lexer{
    pub fn new(data:&str)->Self{
        let string = String::from(data);
        let init = Character::new(string.chars().nth(0).unwrap());
        Lexer{ index:0, current:init, data:string, to_newline:false, column:0, row:0 }
    }
    
    pub fn has_next(&mut self)->bool{
        while self.index < self.data.len(){
            self.current = Character::new(self.data.chars().nth(self.index).unwrap());
			if self.current.unwrap() == '\n'{
				self.column = 0;
				self.row += 1;
				self.to_newline = false;
			}else if !self.to_newline && !self.current.is_whitespace(){
				return true; 
			} 
            self.index += 1;
			self.column += 1;
        }
        return false;
    }
    
    fn pop(&mut self)->char{
        let init = self.data.chars().nth(self.index).unwrap();
        self.index += 1;
        return init;
    }
    
    pub fn get_next_token(&mut self)->Result<Token, String>{
		if self.current.is_alphabetic(){
			return self.get_name_token();
		}else if self.current.is_numeric(){
			return self.get_number_token();
		}else if self.current == '"'{
			return self.get_string_token();
		}else if self.current == '/'{
			let init = self.pop();
			if self.index < self.data.len(){
				let next = self.data.chars().nth(self.index).unwrap();
				if next == '/'{
					self.to_newline = true;
					self.has_next();
					return self.get_next_token();
				}
			}
			return Ok(Token::from_char(init, self.column, self.row));
		}
		return Ok(Token::from_char(self.pop(), self.column, self.row));
	}

	fn get_name_token(&mut self)->Result<Token, String>{
		let mut builder = String::new();
		let column = self.column;
		let row = self.row;
		while self.index < self.data.len(){
            self.current = Character::new(self.data.chars().nth(self.index).unwrap());
            let passable = !self.current.is_alphanumeric();
            if passable { break; }else { builder.push(self.current.unwrap()); }
            self.index += 1;
			self.column += 1;
        }
		return Ok(Token::from_str(builder.as_str(), column, row));
	}

	fn get_number_token(&mut self)->Result<Token, String>{
		let mut builder = String::new();
		let column = self.column;
		let row = self.row;
		while self.index < self.data.len(){
            self.current = Character::new(self.data.chars().nth(self.index).unwrap());
            let important = !self.current.is_whitespace() && self.current.is_hexdigit();
            if important { builder.push(self.current.unwrap()); } else { break; };
            self.index += 1;
			self.column += 1;
        }
		return Ok(Token::from_number(builder.as_str(), column, row));
	}
	
	fn get_string_token(&mut self)->Result<Token, String>{
		let open = self.pop();
		let mut builder = String::from(open);
		let column = self.column;
		let row = self.row;
		while self.index < self.data.len(){
		    let close = self.data.chars().nth(self.index).unwrap();
			if close == open{
                self.pop();
				builder.push(close);
                return Ok(Token::from_str(builder.as_str(), column, row));
			}else{
				builder.push(self.pop());
			}
		}
		return Err(format!("Expecting a closing {} at row:{}, column:{}", if open == '\''{ "'"} else {"\""}, self.row, self.column));
	}
}