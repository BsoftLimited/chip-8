use std::fmt::Error;
use std::fmt::Formatter;
use std::fmt::Debug;
use crate::chip::utils::from_hex;
use crate::chip::utils::Position;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum TokenType{
	Name(String), String(String), Number(u16), Boolean(String), Conditional(String), 
	ForwardSlash, OpenSquareBracket, ClosingSquareBracket, Term(char), Factor(char),
	OpenCurlyBracket, ClosingCurlyBracket, OpenBracket, ClosingBracket, Colon, SemiColon, Coma, Equal, Dot, None, Unknown(char)}

	#[derive(Clone)]
pub struct Token{ tokentype: TokenType, position: Position }
impl Token{
	pub fn from_char(init: char, x: usize, y: usize)->Self{
		let tokentype = match init{
			'{' => TokenType::OpenCurlyBracket,
			'}' => TokenType::ClosingCurlyBracket,
			'[' => TokenType::OpenSquareBracket,
			']' => TokenType::ClosingSquareBracket,
			'(' => TokenType::OpenBracket,
			')' => TokenType::ClosingBracket,
			':' => TokenType::Colon,
			';' => TokenType::SemiColon,
			',' => TokenType::Coma,
            '*' => TokenType::Factor('*'),
			'+' => TokenType::Term('+'),
			'-' => TokenType::Term('-'),
			'=' => TokenType::Equal,
			'.' => TokenType::Dot,
            _=> TokenType::Unknown(init)
		};
        Token{ tokentype, position: Position::new(x, y, 1) }
	}

    pub fn from_str(init: &str, x: usize, y: usize)->Self{
        if (init.starts_with("\"") && init.ends_with("\"")) || (init.starts_with("'") && init.ends_with("'")){
			let mut sub = String::new();
			for index in 1..init.len() - 1{
				sub.push(init.chars().nth(index).unwrap());
			}
			return Token{ tokentype: TokenType::String(sub), position: Position::new(x, y, init.len() - 2) }
		}

		let width = init.len();
		if init.eq("true") || init.eq("false"){
			return Token{ tokentype: TokenType::Boolean(init.to_owned()), position: Position::new(x, y, width) }
		}
		return Token{ tokentype: TokenType::Name(init.to_owned()), position: Position::new(x, y, width) }
    }

	pub fn from_number(init: &str, x: usize, y: usize)->Self{
		let value = from_hex(init);
		let width = init.len();

		return Token{ tokentype: TokenType::Number(value), position: Position::new(x, y, width) }
	}

	pub fn none()->Self{
		return Token{ tokentype: TokenType::None, position: Position::new(0, 0, 0) }
	}

	pub fn tokentype(&self)->TokenType{ self.tokentype.clone() }
	pub fn position(&self)->Position{ self.position }
}

impl Debug for Token{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>{
        write!(f, "Token:{:?} in {:?}", self.tokentype, self.position)
    }
}