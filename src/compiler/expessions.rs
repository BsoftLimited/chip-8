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