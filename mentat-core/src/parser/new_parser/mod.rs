/*
The parser is responsible for converting a collection of Tokens into a valid Mentat statement

We define our grammar to parse as the following

expression -> assignment | term
assignment -> IDENTIFIER (":=" term)*
term -> factor (("+" | "-") factor)*
factor -> unary ( ( "/" | "*") unary)*
unary -> "-" unary | primary
primary -> NUMBER | IDENTIFIER | "(" expresssion ")"
*/

pub mod parser;
