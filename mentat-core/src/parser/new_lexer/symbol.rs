#[derive(Debug, PartialEq)]
pub(crate) enum Symbol {
    Plus,                     // +
    Minus,                    // -
    Star,                     // *
    Slash,                    // /
    Parentheses(Parantheses), // ( )
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
pub(crate) enum Parantheses {
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
