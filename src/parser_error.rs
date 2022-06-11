use std::fmt;

use crate::TokenKind;

pub struct ParserError {
    pos: usize,
    kind: ParserErrorKind,
}

impl ParserError {

    pub fn new( pos: usize, kind: ParserErrorKind ) -> Self {
        ParserError {
            pos,
            kind,
        }
    }

}

impl fmt::Debug for ParserError {

    fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
        write!( f , "ERROR: {:?} at {}", self.kind, self.pos )
    }

}
pub enum ParserErrorKind {
    TokenRequired( TokenKind ),
}

impl fmt::Debug for ParserErrorKind {

    fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
        match self {
            Self::TokenRequired( kind ) => write!( f, "Token required {:?}", kind ),
        }
    }
}


