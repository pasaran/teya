use std::fmt;

use crate::token_kind::{ TokenKind, T };

#[derive(Clone,Copy)]
pub struct Token {
    pub kind: TokenKind,

    pub start: usize,
    pub end: usize,
}

impl fmt::Debug for Token {
    fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
        write!( f , "{:?} {} .. {}", self.kind, self.start, self.end )
    }
}

//  ---------------------------------------------------------------------------------------------------------------  //

#[derive(Copy,Clone,PartialEq,Debug)]
enum State {
    Normal,
    StringFragment,
    StringExpr,
}

#[derive(Clone)]
pub struct Lexer < 'a> {
    bytes: &'a [ u8 ],

    pos: usize,

    prev_kind: TokenKind,
    state: State,
    n_opened_curlies: u32,
    opened_quotes: Vec< u32 >,
}

impl < 'a > Lexer < 'a >{

    pub fn new( s: &'a str ) -> Self {
        Lexer {
            bytes: s.as_bytes(),
            pos: 0,

            prev_kind: TokenKind::None,
            state: State::Normal,
            n_opened_curlies: 0,
            opened_quotes: Vec::new(),
        }
    }

    fn byte_is( &self, pos: usize, b: u8 ) -> bool {
        match self.bytes.get( pos) {
            Some( &x ) if x == b => true,
            _ => false,
        }
    }

    fn byte_matches( &self, pos: usize, f: impl Fn( u8 ) -> bool ) -> bool {
        match self.bytes.get( pos ) {
            Some( &b ) if f( b ) => true,
            _ => false,
        }
    }

    fn iterate_while( &self, pos: usize, f: impl Fn( u8 ) -> bool ) -> usize {
        let mut i = pos;

        loop {
            match self.bytes.get( i ) {
                Some( &b ) if f( b ) => { i += 1; },
                _ => { break i; }
            }
        }
    }

    fn get_token_kind( &mut self ) -> ( TokenKind, usize ) {
        let bytes = self.bytes;
        let pos = self.pos;

        if pos >= bytes.len() {
            return match self.prev_kind {
                TokenKind::EOF => ( TokenKind::None, pos ),
                TokenKind::EOL => ( TokenKind::EOF, pos ),
                _ => ( TokenKind::EOL, pos ),
            };

        } else {
            let b = bytes.get( pos ).unwrap();
            let mut i = pos + 1;

            match ( b, self.state ) {
                ( b'"', State::Normal ) => {
                    self.state = State::StringFragment;

                    self.n_opened_curlies = 0;
                    self.opened_quotes.clear();

                    return ( T!['"'], i );
                }

                ( b'"', State::StringExpr ) => {
                    self.state = State::StringFragment;
                    self.opened_quotes.push( self.n_opened_curlies );
                    self.n_opened_curlies = 0;

                    return ( T!['"'], i );
                }

                ( b'"', State::StringFragment ) => {
                    if self.opened_quotes.is_empty() {
                        self.state = State::Normal;

                    } else {
                        self.n_opened_curlies = self.opened_quotes.pop().unwrap();
                        self.state = State::StringExpr;
                    }

                    return ( T!['"'], i );
                }

                ( b'$', State::StringFragment ) => {
                    if self.byte_is( i , b'{' ) {
                        self.state = State::StringExpr;
                        self.n_opened_curlies = 1;

                        return ( T!["${"], i + 1 );
                    }
                }

                ( b'{', State::StringExpr ) => {
                    self.n_opened_curlies += 1;

                    return ( T!['{'], i );
                }

                ( b'}', State::StringExpr ) => {
                    self.n_opened_curlies -= 1;
                    if self.n_opened_curlies == 0 {
                        self.state = State::StringFragment;
                    }

                    return ( T!['}'], i );
                }

                ( b'\n', _ ) => {
                    self.state = State::Normal;

                    return ( TokenKind::EOL, i );
                }

                ( _, State::StringFragment ) => {
                    while let Some( &b ) = self.bytes.get( i ) {
                        match b {
                            b'\n' | b'"' => { break; }
                            b'$' => {
                                if self.byte_is( i + 1, b'{' ) {
                                    break;

                                } else {
                                    i += 1;
                                }
                            }
                            _ => { i += 1; }
                        }
                    }

                    return ( TokenKind::StringFragment, i );
                }

                _ => (),
            }

            match b {
                b' ' => ( T![ ], self.iterate_while( i, is_space ) ),

                b'/' => {
                    match self.bytes.get( i ) {
                        Some( b'/' ) => {
                            i += 1;
                            loop {
                                match self.bytes.get( i ) {
                                    Some( b'\n' ) | None => { break; }
                                    _ => { i += 1 },
                                }
                            }
                            ( TokenKind::Comment, i )
                        }
                        Some( b'=' ) => ( T![/=], i + 1 ),
                        _ => ( T![/], i ),
                    }
                }

                b'A' ..= b'Z' |
                b'a' ..= b'z' |
                b'_' => {
                    i = self.iterate_while( i, is_id_next );

                    ( id_or_keyword( self.bytes.get(  pos .. i ).unwrap() ), i )
                }

                b'0' ..= b'9' => {
                    let i = self.iterate_while( i, is_digit );

                    if self.byte_is( i, b'.' ) && self.byte_matches( i, is_digit ) {
                        ( TokenKind::Number, self.iterate_while( i + 1, is_digit ) )
                    } else {
                        ( TokenKind::Number, i )
                    }
                }

                b'+' => {
                    if self.byte_is( i, b'=' ) {
                        ( T![+=], i + 1 )
                    } else {
                        ( T![+], i )
                    }
                }

                b'-' => {
                    if self.byte_is( i, b'=' ) {
                        ( T![-=], i + 1 )
                    } else {
                        ( T![-], i )
                    }
                }

                b'*' => {
                    if self.byte_is( i, b'=' ) {
                        ( T![*=], i + 1 )
                    } else {
                        ( T![*], i )
                    }
                }

                b'%' => {
                    if self.byte_is( i, b'=' ) {
                        ( T![ %= ], i + 1 )
                    } else {
                        ( T![%], i )
                    }
                }

                b'=' => {
                    if self.byte_is( i, b'=' ) {
                        ( T![==], i + 1 )
                    } else {
                        ( T![=], i )
                    }
                }

                b'<' => {
                    if self.byte_is( i, b'=' ) {
                        ( T![<=], i + 1 )
                    } else {
                        ( T![<], i )
                    }
                }

                b'>' => {
                    if self.byte_is( i, b'=' ) {
                        ( T![>=], i + 1 )
                    } else {
                        ( T![>], i )
                    }
                }

                b'&' => {
                    if self.byte_is( i, b'&' ) {
                        if self.byte_is( i + 1, b'=' ) {
                            ( T![&&=], i + 2 )
                        } else {
                            ( T![&&], i + 1 )
                        }
                    } else {
                        ( T![&], i )
                    }
                }

                b'|' => {
                    if self.byte_is( i, b'|' ) {
                        if self.byte_is( i + 1, b'=' ) {
                            ( T![||=], i + 2 )

                        } else {
                            ( T![||], i + 1 )
                        }

                    } else {
                        ( T![|], i )
                    }
                }

                b'!' => {
                    if self.byte_is( i, b'=' ) {
                        ( T![!=], i + 1 )
                    } else {
                        ( T![!], i )
                    }
                }

                b'.' => {
                    if self.byte_is( i, b'.' ) {
                        if self.byte_is( i + 1, b'.' ) {
                            ( T![...], i + 2 )

                        } else {
                            ( T![..], i + 1 )
                        }

                    } else {
                        ( T![.], i )
                    }
                }

                b'(' => ( T!['('], i ),
                b')' => ( T![')'], i ),
                b'[' => ( T!['['], i ),
                b']' => ( T![']'], i ),
                b'{' => ( T!['{'], i ),
                b'}' => ( T!['}'], i ),
                b':' => ( T![:], i ),
                b';' => ( T![;], i ),
                b',' => ( T![,], i ),
                b'@' => ( T![@], i ),
                b'#' => ( T![#], i ),
                b'^' => ( T![^], i ),
                b'~' => ( T![~], i ),
                b'?' => ( T![?], i ),
                b'\'' => ( T!['\''], i ),
                b'`' => ( T!['`'], i ),
                b'$' => ( T![$], i ),
                b'\\' => ( T!['\\'], i ),

                _ => ( TokenKind::Unknown, self.iterate_while( i, is_error ) ),
            }
        }
    }
}

impl < 'a > Iterator for Lexer < 'a > {
    type Item = Token;

    fn next( &mut self ) -> Option< Token > {
        let start = self.pos;
        let ( kind, end ) = self.get_token_kind();

        if kind == TokenKind::None {
            return None;
        }

        self.prev_kind = kind;
        self.pos = end;

        Some( Token {
            kind,
            start,
            end,
        } )
    }

}

#[inline]
fn is_space( b: u8 ) -> bool {
    b == b' '
}

#[inline]
fn is_id_next( b: u8 ) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[inline]
fn is_digit( b: u8 ) -> bool {
    b.is_ascii_digit()
}

#[inline]
fn is_error( b: u8 ) -> bool {
    b != b'\n' && ( b < b' ' || b > b'~' )
}

fn id_or_keyword( id: &[ u8 ] ) -> TokenKind {
    match id {
        b"struct" => T![struct],
        b"const" => T![const],
        b"let" => T![let],
        b"if" => T![if],
        b"for" => T![for],
        b"while" => T![while],
        b"fn" => T![fn],
        _ => TokenKind::Id,
    }
}
