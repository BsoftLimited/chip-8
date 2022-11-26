use std::fmt::Error;
use std::fmt::Formatter;
use std::fmt::Debug;
use std::ops::Add;

#[derive(Clone, Copy)]
pub struct Position{ column: usize, line: usize, width: usize }
impl Position{
    pub fn new(column: usize, line: usize, width: usize)->Self{
        Position{ column, line, width }
    }

    pub fn column(&self)->usize{ self.column }
    pub fn line(&self)->usize{ self.line }
    pub fn width(&self)->usize{ self.width }
}

impl PartialEq<Position> for Position{
    fn eq(&self, rhs : &Position) -> bool {
        return self.column == rhs.column && self.line == rhs.line && self.width == rhs.width;
    }
}

impl Add<Position> for Position{
    type Output = Position;
    
    fn add(self, rhs: Position) -> Self::Output {
        let column = if self.column < rhs.column { self.column } else { rhs.column };
        let line = if self.line < rhs.line { self.line } else { rhs.line };
        let width = self.width + rhs.width;

        return Position{ column, line, width };
    }
}

impl Debug for Position{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>{
        write!(f, "Line:{} at Column: {}", self.line, self.column)
    }
}