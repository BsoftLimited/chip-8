

use std::ops::Add;

#[derive(Debug, Clone, Copy)]
pub struct Character{ value: char }
impl Character{
    pub fn new(value: char)->Self{ Character{ value } }

    pub fn is_alphabetic(&self)->bool{
        let value = self.value as u32;
        return (value >= 65 && value <= 91) || (value >= 97 && value <= 123) || self.value == '_';
    }
    
    pub fn is_numeric(&self)->bool{
        let value = self.value as u32;
        return value >= 48 && value <= 57;
    }
    
    pub fn is_alphanumeric(&self)->bool{
        return self.is_alphabetic() || self.is_numeric();
    }
    
    pub fn is_hexdigit(&self)->bool{
        if self.is_numeric() || self.value == 'x'{
            return true;
        }
        let value = self.value as u32;
        return (value >= 65 && value <= 70) || (value >= 97 && value <= 102);
    }

    pub fn is_whitespace(&self)->bool{
        return [' ', '\n', '\t'].contains(&self.value);
    }

    pub fn unwrap(&self)->char{ return self.value; }
}

impl PartialEq<Character> for Character{
    fn eq(&self, rhs: &Character) -> bool {
        return self.value == rhs.value;
    }
}

impl PartialEq<char> for Character{
    fn eq(&self, rhs: &char) -> bool {
        return &self.value == rhs;
    }
}

impl Add<Character> for Character{
    type Output = String;
    
    fn add(self, rhs: Character) -> Self::Output {
        return format!("{}{}", self.value, rhs.value);
    }
}

impl Add<char> for Character{
    type Output = String;
    
    fn add(self, rhs: char) -> Self::Output {
        return format!("{}{}", self.value, rhs);
    }
}

impl Add<String> for Character{
    type Output = String;
    
    fn add(self, rhs: String) -> Self::Output {
        return format!("{}{}", self.value, rhs);
    }
}