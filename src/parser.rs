use crate::SyntaxNode;
use crate::{ Lexer, Token, TokenKind, T, SyntaxKind, ParserEvent, ParserError, ParserErrorKind, token_set::TokenSet };
use crate::parser_event::process;

#[ derive( Clone, Copy ) ]
pub enum Skipper {
    None,
    Inline,
    Block,
}

pub struct Marker {
    pos: usize,
}

impl Marker {

    pub fn new( pos: usize ) -> Self {
        Marker {
            pos,
        }
    }

    pub fn complete( self, p: &mut Parser, kind: SyntaxKind ) -> CompletedMarker {
        // println!( "Marker::complete( {:?} )", kind );
        match &mut p.events[ self.pos ] {
            ParserEvent::Start { kind: slot, .. } => {
                *slot = kind;
            }
            _ => unreachable!( "Ivalid event" ),
        }

        p.push_event( ParserEvent::Finish );

        CompletedMarker::new(self.pos, kind)
    }

    pub fn abandon( self, p: &mut Parser ) {
        //  TODO.
    }

}

pub struct CompletedMarker {
    pos: usize,
    kind: SyntaxKind,
}

impl CompletedMarker {

    pub fn new( pos: usize, kind: SyntaxKind ) -> Self {
        CompletedMarker {
            pos,
            kind,
        }
    }

    pub fn precede( &self, p: &mut Parser ) -> Marker {
        let m = p.start();
        let index = self.pos as usize;
        match &mut p.events[ index ] {
            ParserEvent::Start { forward_parent, .. } => {
                *forward_parent = Some( m.pos - self.pos );
            }
            _ => unreachable!( "Invalid event" ),
        }

        m
    }

}

//  ---------------------------------------------------------------------------------------------------------------  //

pub struct Parser< 'a > {

    input: &'a str,
    pub tokens: Vec< Token< 'a > >,

    events: Vec< ParserEvent< 'a > >,
    // errors: Vec< ParserError >,

    pos: usize,

    skipper: Skipper,
    skippers: Vec< Skipper >,

    last_eaten_token_pos: usize,
}

impl < 'a > Parser< 'a > {

    pub fn new( input: &'a str ) -> Self {
        Parser {
            input,
            tokens: Lexer::new( input ).collect(),

            events: vec![],
            // errors: vec![],

            pos: 0,

            skipper: Skipper::None,
            skippers: vec![],

            last_eaten_token_pos: 0,
        }
    }

    pub fn parse( mut self, rule: fn ( parser: &mut Parser ) -> CompletedMarker ) -> SyntaxNode< 'a > {
        let _node = rule( &mut self );

        process( self.events )
    }

    pub fn push_event( &mut self, event: ParserEvent< 'a > ) {
        self.events.push( event );
    }

    pub fn error( &mut self, kind: ParserErrorKind ) {
        let error = ParserEvent::Error{
            error: ParserError::new( self.pos, kind ),
        };
        self.push_event( error );
    }

    pub fn error_and_bump( &mut self, kind: ParserErrorKind ) {
        self.error( kind );
        self.eat_any();
    }

    pub fn error_recover( &mut self, error_kind: ParserErrorKind, recovery: TokenSet ) {
        match self.current() {
            T![ '{' ] | T![ '}' ] => {
                self.error( error_kind );
                return;
            }
            _ => (),
        }

        if self.at_ts( recovery ) {
            self.error( error_kind );

            return;
        }

        let m = self.start();
        self.error( error_kind );
        self.eat_any();
        m.complete(self, SyntaxKind::Error );
    }

    pub fn nth( &self, n: usize ) -> Option< &Token > {
        self.tokens.get( self.pos + n )
    }

    pub fn nth_kind( &self, n: usize ) -> TokenKind {
        self.nth( n ).map_or( TokenKind::EOF, | t | t.kind )
    }

    pub fn current( &self ) -> TokenKind {
        self.nth_kind( 0 )
    }

    pub fn at( &self, kind: TokenKind ) -> bool {
        self.nth_kind( 0 ) == kind
    }

    pub fn at_ts( &self, ts: TokenSet ) -> bool {
        ts.contains( self.current() )
    }

    pub fn at_eof( &self ) -> bool {
        self.at( TokenKind::EOF )
    }

    pub fn at_eol( &self ) -> bool {
        self.at( TokenKind::EOL )
    }

    pub fn mov( &mut self ) {
        let token = self.tokens[ self.pos ];
        self.push_event( ParserEvent::Token { token } );
        self.pos += 1;
    }

    pub fn eat( &mut self, kind: TokenKind ) -> bool {
        if self.nth_kind( 0 ) == kind {
            // println!( "eaten={:?}", kind );
            if kind != TokenKind::EOF {
                self.last_eaten_token_pos = self.pos;
                self.mov();
                self.skip();
            }

            return true;
        }

        false
    }

    pub fn expect( &mut self, kind: TokenKind ) -> bool {
        if self.eat( kind ) {
            true

        } else {
            self.error( ParserErrorKind::TokenRequired( kind ) );

            false
        }
    }

    pub fn eat_any( &mut self ) -> bool {
        if self.nth( 0 ).is_some() {
            // println!( "eaten_any={:?}", self.kind() );
            self.last_eaten_token_pos = self.pos;
            self.mov();
            self.skip();

            true

        } else {
            false
        }
    }

    pub fn start( &mut self ) -> Marker {
        let pos = self.events.len();
        self.push_event( ParserEvent::empty());

        Marker::new( pos )
    }

    pub fn eol( &mut self ) {
        self.eat( TokenKind::EOL );
    }

    pub fn set_skipper( &mut self, skipper: Skipper ) {
        self.skippers.push( self.skipper );

        self.skipper = skipper;
        self.skip();
    }

    pub fn restore_skipper( &mut self ) {
        if let Some( skipper ) = self.skippers.pop() {
            self.skipper = skipper;
            self.skip();
        }
    }

    pub fn skip( &mut self ) {
        match self.skipper {
            Skipper::None => {},

            Skipper::Inline => {
                while self.at( T![ ] ) {
                    // println!( "skipped {:?}", self.kind() );
                    self.mov();
                }
            }

            Skipper::Block => {
                while self.at( T![ ] ) || self.at( TokenKind::EOL ) || self.at( TokenKind::Comment ) {
                    // println!( "skipped {:?}", self.kind() );
                    self.mov();
                }
            }
        }
    }

}

//  ---------------------------------------------------------------------------------------------------------------  //
