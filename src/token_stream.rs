#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,

    pub start: usize,
    pub end: usize,
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum TokenKind {
    None,

    Space,
    Bang,
    Quote,
    Pound,
    //  Dollar,
    Percent,
    Amp,
    //  Apos,
    OpenParen,
    CloseParen,
    Star,
    Plus,
    Comma,
    Minus,
    Dot,
    Slash,
    Colon,
    //  Semicolon,
    Lt,
    Eq,
    Gt,
    Question,
    At,
    OpenBracket,
    //  Backslash,
    CloseBracket,
    //  Caret,
    //  Underscore,
    //  Backtick,
    OpenBrace,
    Pipe,
    CloseBrace,
    //  Tilde,

    BangEq,
    PercentEq,
    AmpAmp,
    AmpAmpEq,
    StarEq,
    PlusEq,
    MinusEq,
    DotDot,
    DotDotDot,
    SlashEq,
    LtEq,
    EqEq,
    GtEq,
    PipePipe,
    PipePipeEq,

    EOL,
    EOF,
    //  Indent,
    //  Dedent,
    Comment,
    Ident,
    Int,
    Float,
    StringFragment,
    DollarOpenBrace,

    Struct,
    Let,
    Const,
    Fn,
    If,
    For,

    Error,
}

type K = TokenKind;

//  ---------------------------------------------------------------------------------------------------------------  //

#[derive(Copy,Clone,PartialEq,Debug)]
enum State {
    Normal,
    StringFragment,
    StringExpr,
}

#[derive(Clone)]
pub struct TokenStream < 'a> {
    bytes: &'a [ u8 ],

    pos: usize,

    prev_kind: TokenKind,
    state: State,
    n_opened_curlies: u32,
    opened_quotes: Vec< u32 >,
}

impl < 'a > TokenStream < 'a >{

    pub fn new( s: &'a str ) -> Self {
        TokenStream {
            bytes: s.as_bytes(),
            pos: 0,

            prev_kind: K::None,
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
                K::EOF => ( K::None, pos ),
                K::EOL => ( K::EOF, pos ),
                _ => ( K::EOL, pos ),
            };

        } else {
            let b = bytes.get( pos ).unwrap();
            let mut i = pos + 1;

            match ( b, self.state ) {
                ( b'"', State::Normal ) => {
                    self.state = State::StringFragment;

                    self.n_opened_curlies = 0;
                    self.opened_quotes.clear();

                    return ( K::Quote, i );
                }

                ( b'"', State::StringExpr ) => {
                    self.state = State::StringFragment;
                    self.opened_quotes.push( self.n_opened_curlies );
                    self.n_opened_curlies = 0;

                    return ( K::Quote, i );
                }

                ( b'"', State::StringFragment ) => {
                    if self.opened_quotes.is_empty() {
                        self.state = State::Normal;

                    } else {
                        self.n_opened_curlies = self.opened_quotes.pop().unwrap();
                        self.state = State::StringExpr;
                    }

                    return ( K::Quote, i );
                }

                ( b'$', State::StringFragment ) => {
                    if self.byte_is( i , b'{' ) {
                        self.state = State::StringExpr;
                        self.n_opened_curlies = 1;

                        return ( K::DollarOpenBrace, i + 1 );
                    }
                }

                ( b'{', State::StringExpr ) => {
                    self.n_opened_curlies += 1;

                    return ( K::OpenBrace, i );
                }

                ( b'}', State::StringExpr ) => {
                    self.n_opened_curlies -= 1;
                    if self.n_opened_curlies == 0 {
                        self.state = State::StringFragment;
                    }

                    return ( K::CloseBrace, i );
                }

                ( b'\n', _ ) => {
                    self.state = State::Normal;

                    return ( K::EOL, i );
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

                    return ( K::StringFragment, i );
                }

                _ => (),
            }

            match b {
                b' ' => return ( K::Space, self.iterate_while( i, is_space ) ),

                b'/' => {
                    return match self.bytes.get( i ) {
                        Some( b'/' ) => {
                            i += 1;
                            loop {
                                match self.bytes.get( i ) {
                                    Some( b'\n' ) | None => { break; }
                                    _ => { i += 1 },
                                }
                            }
                            ( K::Comment, i )
                        }
                        Some( b'=' ) => ( K::SlashEq, i + 1 ),
                        _ => ( K::Slash, i ),
                    }
                }

                b'A' ..= b'Z' |
                b'a' ..= b'z' |
                b'_' => {
                    i = self.iterate_while( i, is_id_next );

                    return ( id_or_keyword( self.bytes.get(  pos .. i ).unwrap() ), i );
                }

                b'0' ..= b'9' => {
                    let i = self.iterate_while( i, is_digit );
                    if self.byte_is( i, b'.' ) && self.byte_matches( i, is_digit ) {
                        return ( K::Float, self.iterate_while( i + 1, is_digit ) );
                    }
                    return ( K::Int, i );
                }

                b'+' => {
                    if self.byte_is( i, b'=' ) { return ( K::PlusEq, i + 1 ); }
                    return ( K::Plus, i );
                }

                b'-' => {
                    if self.byte_is( i, b'=' ) { return ( K::MinusEq, i + 1 ); }
                    return ( K::Minus, i );
                }

                b'*' => {
                    if self.byte_is( i, b'=' ) { return ( K::StarEq, i + 1 ); }
                    return ( K::Star, i );
                }

                b'%' => {
                    if self.byte_is( i, b'=' ) { return ( K::PercentEq, i + 1 ); }
                    return ( K::Percent, i );
                }

                b'=' => {
                    if self.byte_is( i, b'=' ) { return ( K::EqEq, i + 1 ); }
                    return ( K::Eq, i );
                }

                b'<' => {
                    if self.byte_is( i, b'=' ) { return ( K::LtEq, i + 1 ); }
                    return ( K::Lt, i );
                }

                b'>' => {
                    if self.byte_is( i, b'=' ) { return ( K::GtEq, i + 1 ); }
                    return ( K::Gt, i );
                }

                b'&' => {
                    if self.byte_is( i, b'&' ) {
                        if self.byte_is( i + 1, b'=' ) { return ( K::AmpAmpEq, i + 2 ); }
                        return ( K::AmpAmp, i + 1 );
                    }
                    return ( K::Amp, i );
                }

                b'|' => {
                    if self.byte_is( i, b'|' ) {
                        if self.byte_is( i + 1, b'=' ) { return ( K::PipePipeEq, i + 2 ); }
                        return ( K::PipePipe, i + 1 );
                    }
                    return ( K::Pipe, i );
                }

                b'!' => {
                    if self.byte_is( i, b'=' ) { return ( K::BangEq, i + 1 ); }
                    return ( K::Bang, i );
                }

                b'.' => {
                    if self.byte_is( i, b'.' ) {
                        if self.byte_is( i + 1, b'.' ) { return ( K::DotDotDot, i + 2 ); }
                        return ( K::DotDot, i + 1 );
                    }
                    return ( K::Dot, i );
                }

                b'(' => return ( K::OpenParen, i ),
                b')' => return ( K::CloseParen, i ),
                b'[' => return ( K::OpenBracket, i ),
                b']' => return ( K::CloseBracket, i ),
                b'{' => return ( K::OpenBrace, i ),
                b'}' => return ( K::CloseBrace, i ),
                b':' => return ( K::Colon, i ),
                //  b';' => return ( K::Semicolon, i ),
                b',' => return ( K::Comma, i ),
                b'@' => return ( K::At, i ),
                b'#' => return ( K::Pound, i ),
                //  b'^' => return ( K::Caret, i ),
                //  b'~' => return ( K::Tilde, i ),
                b'?' => return ( K::Question, i ),
                //  b'\'' => return ( K::Apos, i ),
                //  b'`' => return ( K::Backtick, i ),
                //  b'$' => return ( K::Dollar, i ),
                //  b'\\' => return ( K::Backslash, i ),

                _ => {
                    return ( K::Error, self.iterate_while( i, is_error ) );
                }
            }
        }
    }
}

impl < 'a> Iterator for TokenStream < 'a > {
    type Item = Token;

    fn next( &mut self ) -> Option< Token > {
        let start = self.pos;
        let ( kind, end ) = self.get_token_kind();

        if kind == K::None {
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
    b < b'\n' ||
    b > b'\n' && b < b' ' ||
    b > b'~'
}

fn id_or_keyword( id: &[ u8 ] ) -> TokenKind {
    match id {
        b"struct" => K::Struct,
        b"const" => K::Const,
        b"let" => K::Let,
        b"if" => K::If,
        b"for" => K::For,
        b"fn" => K::Fn,
        _ => K::Ident,
    }
}
