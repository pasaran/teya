use crate::TokenKind;

#[ derive( Copy, Clone ) ]
pub struct TokenSet {
    bits: u128,
}

impl TokenSet {

    pub const fn new( kinds: &[ TokenKind ] ) -> TokenSet {
        let mut bits = 0u128;

        let mut i = 0;
        while i < kinds.len() {
            bits |= mask( kinds[ i ] );
            i += 1;
        }

        TokenSet {
            bits,
        }
    }

    pub const fn contains( &self, kind: TokenKind ) -> bool {
        self.bits & mask( kind ) != 0
    }

    pub const fn union( self, other: TokenSet ) -> TokenSet {
        TokenSet {
            bits: self.bits | other.bits,
        }
    }

}

const fn mask( kind: TokenKind ) -> u128 {
    1 << ( kind as usize )
}

#[ cfg( test ) ]
mod tests {
    use super::*;

    #[ test ]
    fn token_set() {
        let set_1 = TokenSet::new( &[ TokenKind::Id, TokenKind::Number, TokenKind::Comment ] );
        assert!( set_1.contains( TokenKind::Id ) );
        assert!( set_1.contains( TokenKind::Number ) );
        assert!( set_1.contains( TokenKind::Comment ) );
        assert!( !set_1.contains( TokenKind::Space ) );

        let set_2 = TokenSet::new( &[ TokenKind::OpenBrace, TokenKind::CloseBrace ] );
        let set_2 = set_1.union( set_2 );
        assert!( set_2.contains( TokenKind::Id ) );
        assert!( set_2.contains( TokenKind::Number ) );
        assert!( set_2.contains( TokenKind::Comment ) );
        assert!( set_2.contains( TokenKind::OpenBrace ) );
        assert!( set_2.contains( TokenKind::CloseBrace ) );
        assert!( !set_2.contains( TokenKind::Space ) );
    }

}
