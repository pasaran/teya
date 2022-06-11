use crate::{ Lexer, Token, TokenKind, T, SyntaxKind, ParserEvent, ParserError, ParserErrorKind, token_set::TokenSet };

#[derive(Clone, Copy)]
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

    // pub fn cancel( self, p: &mut Parser ) {

    // }

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

pub struct Parser {

    //  input: &'a str,
    pub tokens: Vec< Token >,

    events: Vec< ParserEvent >,
    errors: Vec< ParserError >,

    pos: usize,

    skipper: Skipper,
    skippers: Vec< Skipper >,

    last_eaten_token_pos: usize,
}

impl Parser {

    pub fn new( input: &str ) -> Self {
        Parser {
            //  input,
            tokens: Lexer::new( input ).collect(),

            events: vec![],
            errors: vec![],

            pos: 0,

            skipper: Skipper::None,
            skippers: vec![],

            last_eaten_token_pos: 0,
        }
    }

    pub fn parse( &mut self, rule: fn ( parser: &mut Parser ) -> CompletedMarker ) {
        let _node = rule( self );

        for error in self.errors.iter() {
            println!( "{:?}", error );
        }

        //  node
        // self.build_tree( node )
    }

    pub fn finish( self ) -> Vec< ParserEvent >  {
        self.events
    }

    // fn build_tree( &mut self, node: CompletedMarker ) -> SyntaxNode {
    //     let mut tree = SyntaxNode::new( node.kind );

    //     if node.children.is_empty() {
    //         self.push_tokens( &mut tree, node.start, node.end + 1 );

    //     } else {
    //         let mut start = node.start;
    //         for child in node.children {
    //             self.push_tokens( &mut tree, start, child.start );
    //             start = child.end + 1;
    //             tree.push( SyntaxElement::Node( self.build_tree( child ) ) );
    //         }
    //         self.push_tokens( &mut tree, start, node.end + 1 );
    //     }

    //     tree
    // }

    // fn push_tokens( &self, tree: &mut SyntaxNode, start: usize, end: usize ) {
    //     for token in &self.tokens[ start .. end ] {
    //         tree.push( SyntaxElement::Token( *token ) );
    //     }
    // }

    pub fn push_event( &mut self, event: ParserEvent ) {
        self.events.push( event );
    }

    pub fn error( &mut self, kind: ParserErrorKind ) {
        let error = ParserEvent::Error{
            error: ParserError::new( self.pos, kind ),
        };
        self.push_event( error );
    }

    // pub fn error_and_mov( &mut self, kind: ParserErrorKind ) {
    //     self.error( kind );
    //     self.eat_any();
    // }

    pub fn nth( &self, n: usize ) -> Option< &Token > {
        self.tokens.get( self.pos + n )
    }

    pub fn nth_kind( &self, n: usize ) -> Option< TokenKind > {
        self.nth( n ).map( | t | t.kind )
    }

    pub fn kind( &self ) -> Option< TokenKind > {
        self.nth_kind( 0 )
    }

    pub fn at_ts( &self, ts: TokenSet ) -> bool {
        if let Some( kind ) = self.kind() {
            ts.contains( kind )

        } else {
            false
        }
    }

    pub fn is_kind( &self, kind: TokenKind ) -> bool {
        self.nth( 0 ).map_or( false, | t | t.kind == kind )
    }

    pub fn is_eof( &self ) -> bool {
        self.is_kind( TokenKind::EOF )
    }

    pub fn is_eol( &self ) -> bool {
        self.is_kind( TokenKind::EOL )
    }

    pub fn mov( &mut self ) {
        let token = self.tokens[ self.pos ];
        self.push_event( ParserEvent::Token { token } );
        self.pos += 1;
    }

    pub fn eat( &mut self, kind: TokenKind ) -> bool {
        if let Some( &t ) = self.nth( 0 ) {
            if t.kind == kind {
                // println!( "eaten={:?}", kind );
                self.last_eaten_token_pos = self.pos;
                self.mov();
                self.skip();

                return true;
            }
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

    // pub fn end( &self, node: CompletedMarker, kind: SyntaxKind ) -> CompletedMarker {
    //     node.end( self.last_eaten_token_pos, kind )
    // }

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
                while self.is_kind( T![ ] ) {
                    // println!( "skipped {:?}", self.kind() );
                    self.mov();
                }
            }

            Skipper::Block => {
                while self.is_kind( T![ ] ) || self.is_kind( TokenKind::EOL ) || self.is_kind( TokenKind::Comment ) {
                    // println!( "skipped {:?}", self.kind() );
                    self.mov();
                }
            }
        }
    }

}

//  ---------------------------------------------------------------------------------------------------------------  //

