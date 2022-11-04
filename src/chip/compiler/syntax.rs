const KeyWords:[&str; 4] = ["in", "as", "is", "to"];
const DataType: [&str; 5] = ["num" ,"str", "bool", "char", "array"];
const DeclareWords: [&str; 2] = ["let", "fun"];
const Controls:[&str; 6] = ["for", "if", "when", "else", "while", "do"];
const Conditions: [&str; 4] = ["and", "or", "equals", "not"];
const BoolValues: [&str; 2] = ["true", "false"];

pub fn word_in(list: &[&str], key: &str)->bool{
    return list.contains(&key);
}

pub fn is_keyword(word: &str)->bool{
    return word_in(&KeyWords, word) | word_in(&DataType, word) | word_in(&DeclareWords, word) | word_in(&Controls, word) | word_in(&Conditions, word) | word_in(&BoolValues, word);
}

pub fn is_datatype(word: &str)->bool { return word_in(&DataType, word); }

pub fn char_in(list: &str, key: char)-> bool{
    return list.contains(key);
}