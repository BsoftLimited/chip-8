use crate::chip::compiler::syntax::is_datatype;
use crate::chip::compiler::syntax::is_keyword;
use crate::chip::utils::{ Token, Lexer, TokenType};

#[derive(Debug)]
pub enum Expression{
    String(String), Number(f32), Negation(Box<Expression>), Boolean(String), Array(Vec<Expression>),
    Binary{ left: Box<Expression>, op:String, right: Box<Expression>},
    Variable{ name: String, dt: String, value: Option<Box<Expression>>},
    Assignment{ variable: String, exp: Box<Expression> },
    ArgumentDefinition{ name: String, dt: String, value: Option<Box<Expression>> },
    ArgumentPassing{ name: String, value: Box<Expression> },
    FunctionDefinition{name: String, args: Vec<Expression>, rtype: Option<String>, body: Vec<Expression>},
    FunctionCall{name: String, args: Vec<Expression>}, None }

fn variant_equal(a: &Token, b: &Token)->bool{
    return std::mem::discriminant(&a.tokentype()) == std::mem::discriminant(&b.tokentype());
}

pub struct Parser{ lexer:Box<Lexer>, errors: Vec<String> , current: Token }
impl Parser{
    pub fn new(data:&str)->Self{
        let mut parser = Parser{ lexer: Box::new(Lexer::new(data)), errors: Vec::new() , current: Token::none() };
        parser.next_token();
        return parser;
    }

    pub fn has_next(&mut self)->bool{ self.lexer.has_next() }
    
    fn next_token(&mut self)->bool{
        while self.lexer.has_next(){
            match self.lexer.get_next_token(){
                Err(error) =>{ self.errors.push(String::from(error)); }
                Ok(token) =>{
                    self.current = token;
                    return true;
                }
            }
        }
        self.current = Token::none();
        return false;
    }

    fn pop_token(&mut self)->Token{
        let init = self.current.clone();
        self.next_token();
        return init;
    }

    fn unwrap(token:&Token)->String{
        let mut init = String::new();
        if let TokenType::Name(value) = token.tokentype(){
            init = value.clone();
        }else if let TokenType::String(value) = token.tokentype(){
            init = value.clone();
        } 
        init
    }

    pub fn get_next(&mut self)->Expression{
        while !matches!(self.current.tokentype(), TokenType::None){
            if let TokenType::Name(name) = self.current.tokentype(){
                match name.as_str(){
                    "let" => return self.initilaization(),
                    "fun" => return self.function_declaraton(),
                    _ => return self.get()
                }
            }
            let token = self.pop_token();
            self.errors.push(format!("Unexpected token: {:?}", token));
        }
        return Expression::None;
    }

    fn get(&mut self)->Expression{
        let init = self.pop_token();
        if let TokenType::OpenBracket = self.current.tokentype(){
            self.pop_token();
            return Expression::FunctionCall{ name: Parser::unwrap(&init), args: self.get_argument_passing() };
        }else if let TokenType::Equal = self.current.tokentype(){
            self.pop_token();
            return Expression::Assignment{ variable: Parser::unwrap(&init), exp: Box::new(self.make_conditional()) };
        }

        if !matches!(self.current.tokentype(), TokenType::SemiColon){
            self.errors.push(format!("expected a semi colon ';'"));
        }
        return Expression::None;
    }

    fn initilaization(&mut self)->Expression{
        let mut name: Option<String> = None;
        let mut data_type: Option<String> = None;
        let mut step: u8 = 0;

        while self.next_token(){
            if let TokenType::Name(value) = self.current.tokentype() {
                if is_keyword(value.as_str()) && step == 0{
                    self.errors.push(format!("the word: {} is a reserve word expecting a {}", value, if step == 0 { "name" } else { "Data type" }));
                }else if  name.is_none() && step == 0 {
                    name = Option::from(value.clone());
                    step = 1;
                }else if data_type.is_none() && step == 2 && is_datatype(value.as_str()){
                    data_type = Option::from(value.clone());
                    step = 3;
                }
            }else if let TokenType::Colon = self.current.tokentype() {
                if step == 1{
                    step = 2;
                }
            }else if let TokenType::Equal = &self.current.tokentype(){
                if step == 3{
                    self.next_token();
                    let value = Some(Box::new(self.make_conditional()));
                    if matches!(self.current.tokentype(), TokenType::SemiColon){
                        self.next_token();
                        return Expression::Variable{ name: name.unwrap(), dt: data_type.unwrap(), value };
                    }
                }
            }else if let TokenType::SemiColon = self.current.tokentype() {
                if step == 3 {
                    self.next_token();
                    return Expression::Variable{ name: name.unwrap(), dt: data_type.unwrap(), value: None};
                }
            }
        }
        return Expression::None;
    }

    fn get_argument_definition(&mut self)->Expression{
        let name: String = Parser::unwrap(&self.current);
        let mut dt: Option<String> = None;
        let mut value: Option<Box<Expression>> = None;
        let mut step: u8 = 0;
        while self.next_token(){
            if let TokenType::Name(value) = self.current.tokentype(){
                if dt.is_none() && is_datatype(value.as_str()) && step == 1{
                    dt = Option::from(value.clone());
                    step = 2;
                }else if is_keyword(value.as_str()){
                    self.errors.push(format!("the word: {} is a reserve word expecting a {}", value, if step == 0 { "name" } else { "Data type" }));
                }
            }else if let TokenType::Colon = self.current.tokentype(){
                if step == 0{ step = 1; }
            }else if let TokenType::Equal = self.current.tokentype(){
                if step == 2{
                    value = Some(Box::new(self.make_conditional()));
                    step = 3;
                }
            }else if matches!(self.current.tokentype(), TokenType::Coma) || matches!(self.current.tokentype(), TokenType::ClosingBracket){
                if step == 2 || step == 3{
                    return Expression::ArgumentDefinition{ name, dt: dt.as_ref().unwrap().clone(), value };
                }
            }
        }
        return Expression::None;
    }

    fn function_declaraton(&mut self)->Expression{
        let mut args: Vec<Expression> = Vec::new();
        let mut body: Vec<Expression> = Vec::new();
        let mut name: Option<String> = None;
        let mut rtype: Option<String> = None;
        let mut step = 0;

        while self.next_token(){
            if let TokenType::Name(value) = self.current.tokentype() {
                if step == 0 && is_keyword(value.as_str()){
                    self.errors.push(format!("the word: {} is a reserve word expecting a {}", value, if step == 0 { "name" } else { "Data type" }));
                }else if  step == 0 && name.is_none() {
                    name = Option::from(value.clone());
                    step = 1;
                }else if rtype.is_none() && step == 4 && is_datatype(value.as_str()){
                    rtype = Option::from(value.clone());
                    step = 5;
                }else if step == 2{
                    args.push(self.get_argument_definition());
                }
            }else if let TokenType::OpenBracket = self.current.tokentype() {
                if step == 1{ step = 2; }
            }else if let TokenType::ClosingBracket = self.current.tokentype() {
                if step == 2{ step = 3; }
            }else if let TokenType::Colon = self.current.tokentype() {
                if step == 3{ step = 4; }
            }else if let TokenType::OpenCurlyBracket = self.current.tokentype() {
                if step == 3 || step == 5{
                    self.next_token();
                    while !(matches!(self.current.tokentype(), TokenType::ClosingCurlyBracket) || matches!(self.current.tokentype(), TokenType::None)){
                        body.push(self.get_next());
                    }
                    if let TokenType::None = self.current.tokentype() {
                        self.errors.push(format!("Unexpected end of tokens expecting a closing bracket ')'"));
                    }
                    return Expression::FunctionDefinition{ name:name.unwrap(), args, rtype, body };
                }
            }
        }
        return Expression::None;
    }

    fn get_argument_passing(&mut self)->Vec<Expression>{
        let mut args: Vec<Expression> = Vec::new();
        let mut name: Option<String> = None;
        let mut step = 0;
        loop{
            if matches!(self.current.tokentype(), TokenType::Name(_)) || matches!(self.current.tokentype(), TokenType::Colon) || matches!(self.current.tokentype(), TokenType::Coma){
                let token = self.pop_token();
                if let TokenType::SemiColon = token.tokentype(){
                    if name.is_some() && step == 1{
                        args.push( Expression::ArgumentPassing{ name: name.as_ref().unwrap().clone(), value: Box::new(self.make_conditional())});
                        step = 2;
                    }else{
                        self.errors.push(format!("Unexpected column expecting an argument"));
                    }
                }else if let TokenType::Coma = token.tokentype(){
                    if step == 1 {
                        if name.is_some(){
                            self.errors.push(format!("Unexpected token: {:?} exprcting colon(:)", self.current));    
                        }
                    }
                    name = None;
                    step = 0;
                }else if let TokenType::Name(value) = token.tokentype(){
                    if name.is_some() && step == 1{
                        self.errors.push(format!("Unexpected token: {:?} expecting a column(:)", &name));
                    }else if step == 0{
                        if is_keyword(value.as_str()){
                            self.errors.push(format!("the word: {} is a reserve word", value));
                        }else{
                            name = Some(value.clone());
                            step = 1;
                        }
                    } 
                }
            } else if matches!(self.current.tokentype(), TokenType::ClosingBracket) || matches!(self.current.tokentype(), TokenType::None){
                self.next_token();
                break;
            } else{
                let token = self.pop_token();
                self.errors.push(format!("Unexpected token: {:?}", token));
            }
        }
        return args;
    }

    pub fn make_conditional(&mut self)->Expression{
        let left = self.make_term();
        if let TokenType::Conditional(op) = self.current.tokentype().clone(){
            self.next_token();
            return Expression::Binary{ left: Box::new(left), op: op.clone(), right: Box::new(self.make_conditional()) }
        }
        return left;
    }

    pub fn make_term(&mut self)->Expression{
        let left = self.make_factor();
        if let TokenType::Term(op) = self.current.tokentype().clone(){
            self.next_token();
            return Expression::Binary{ left: Box::new(left), op: String::from(op.clone()), right: Box::new(self.make_conditional()) }
        }
        return left;
    }

    pub fn make_factor(&mut self)->Expression{
        let left = self.make_value();
        if let TokenType::Factor(op) = self.current.tokentype().clone(){
            self.next_token();
            return Expression::Binary{ left: Box::new(left), op: String::from(op.clone()), right: Box::new(self.make_conditional()) }
        }
        return left;
    }

    fn make_value(&mut self)->Expression{
        while !matches!(self.current.tokentype(), TokenType::None){
            if matches!(self.current.tokentype(), TokenType::String(_)) || matches!(self.current.tokentype(), TokenType::Boolean(_)) || matches!(self.current.tokentype(), TokenType::Number(_)){
                let init = self.pop_token();
                if let TokenType::String(value) = init.tokentype(){
                    return Expression::String(value.clone());
                }else if let TokenType::Boolean(value) = init.tokentype(){
                    return Expression::Boolean(value.clone());
                }else if let TokenType::Number(value) = init.tokentype(){
                    return Expression::Number(value.clone() as f32);
                }
            }else if matches!(self.current.tokentype(), TokenType::OpenSquareBracket){
                self.next_token();
                return self.make_array();
            }else{
                self.errors.push(format!("Unexpected token: {:?} expecting an string literal or boolean value", &self.current));
                self.next_token();
            }
        }
        return Expression::None;
    }

    fn make_array(&mut self)->Expression{
        let mut array = Vec::new();

        while !matches!(self.current.tokentype(), TokenType::None) && !matches!(self.current.tokentype(), TokenType::ClosingSquareBracket){
            array.push(self.make_value());
            if matches!(self.current.tokentype(), TokenType::ClosingSquareBracket){
                self.next_token();
                break;
            } if matches!(self.current.tokentype(), TokenType::Coma){
                self.next_token();
            }else{
                self.errors.push(format!("Unexpected token: {:?}, expected coma(,) or closing square bracket(])", self.current));
            }
        }
        return Expression::Array(array);
    }

    pub fn print_errors(&self){
        for error in &self.errors{
            println!("{}", error);
        }
    }
}