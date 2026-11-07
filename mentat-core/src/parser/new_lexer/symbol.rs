#[derive(Debug, PartialEq)]
pub(crate) enum Symbol {
    Plus,                     // +
    Minus,                    // -
    Star,                     // *
    Slash,                    // /
    Parentheses(Parentheses), // ( )
    Angle(Angle),             // < >
    Brackets(Brackets),       // [ ]
    Braces(Braces),           // { }
    Comma,                    // ,
    Caret,                    // ^
    Assignment,               // :=
    ThinArrow,                // ->
    FatArrow,                 // =>
}

#[derive(Debug, PartialEq)]
pub(crate) enum Parentheses {
    Open,   // (
    Closed, // )
}

#[derive(Debug, PartialEq)]
pub(crate) enum Angle {
    Left,  // <
    Right, // >
}

#[derive(Debug, PartialEq)]
pub(crate) enum Brackets {
    Open,   // [
    Closed, // ]
}

#[derive(Debug, PartialEq)]
pub(crate) enum Braces {
    Open,   // {
    Closed, // }
}

impl Into<String> for &Symbol {
    fn into(self) -> String {
        match self {
            Symbol::Plus => String::from("+"),
            Symbol::Minus => String::from("-"),
            Symbol::Star => String::from("*"),
            Symbol::Slash => String::from("/"),
            Symbol::Parentheses(parentheses) => parentheses.into(),
            Symbol::Angle(angle) => angle.into(),
            Symbol::Brackets(brackets) => brackets.into(),
            Symbol::Braces(braces) => braces.into(),
            Symbol::Comma => String::from(","),
            Symbol::Caret => String::from("^"),
            Symbol::Assignment => String::from(":="),
            Symbol::ThinArrow => String::from("->"),
            Symbol::FatArrow => String::from("=>"),
        }
    }
}

impl Into<String> for &Parentheses {
    fn into(self) -> String {
        match self {
            Parentheses::Open => String::from("("),
            Parentheses::Closed => String::from(")"),
        }
    }
}

impl Into<String> for &Angle {
    fn into(self) -> String {
        match self {
            Angle::Left => String::from("<"),
            Angle::Right => String::from(">"),
        }
    }
}

impl Into<String> for &Brackets {
    fn into(self) -> String {
        match self {
            Brackets::Open => String::from("["),
            Brackets::Closed => String::from("]"),
        }
    }
}

impl Into<String> for &Braces {
    fn into(self) -> String {
        match self {
            Braces::Open => String::from("{"),
            Braces::Closed => String::from("}"),
        }
    }
}
