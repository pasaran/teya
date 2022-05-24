use crate::{ Lexer, Token, TokenKind, T, SyntaxKind, SyntaxNode, SyntaxElement };

#[derive(Clone, Copy)]
enum Skipper {
    None,
    Inline,
    Block,
}

pub struct Node {
    start: usize,
    end: usize,
    kind: SyntaxKind,
    children: Vec< Node >,
}

impl Node {

    pub fn new( start: usize ) -> Self {
        Node {
            start,
            end: 0,
            kind: SyntaxKind::None,
            children: Vec::new(),
        }
    }

    pub fn start( &self ) -> Node {
        Node::new( self.start )
    }

    pub fn end( mut self, end: usize, kind: SyntaxKind ) -> Self {
        self.kind = kind;
        self.end = end;

        self
    }

    pub fn add( &mut self, node: Node ) {
        self.children.push( node );
    }

}

pub struct Parser {

    //  input: &'a str,
    pub tokens: Vec< Token >,

    pos: usize,

    skipper: Skipper,

    last_eaten_token_pos: usize,
}

impl Parser {

    pub fn new( input: &str ) -> Self {
        Parser {
            //  input,
            tokens: Lexer::new( input ).collect(),

            pos: 0,

            skipper: Skipper::None,

            last_eaten_token_pos: 0,
        }
    }

    pub fn parse( &mut self, kind: SyntaxKind ) -> SyntaxNode {
        let node = match kind {
            SyntaxKind::SourceFile => r_source_file( self ),
            SyntaxKind::InlineExpr => r_inline_expr( self ),
            _ => panic!( "Invalid kind" ),
        };

        self.build_tree( node )
    }

    fn build_tree( &self, node: Node ) -> SyntaxNode {
        let mut tree = SyntaxNode::new( node.kind );

        if node.children.is_empty() {
            self.push_tokens( &mut tree, node.start, node.end + 1 );

        } else {
            let mut start = node.start;
            for child in node.children {
                self.push_tokens( &mut tree, start, child.start );
                start = child.end + 1;
                tree.push( SyntaxElement::Node( self.build_tree( child ) ) );
            }
            self.push_tokens( &mut tree, start, node.end + 1 );
        }

        tree
    }

    fn push_tokens( &self, tree: &mut SyntaxNode, start: usize, end: usize ) {
        for token in &self.tokens[ start .. end ] {
            tree.push( SyntaxElement::Token( *token ) );
        }
    }

    fn get( &self ) -> Option< &Token > {
        self.tokens.get( self.pos )
    }

    fn curr( &self ) -> Option< TokenKind > {
        self.get().map( | t | t.kind )
    }

    fn is( &self, kind: TokenKind ) -> bool {
        self.get().map_or( false, | t | t.kind == kind )
    }

    fn is_eof( &self ) -> bool {
        self.is( TokenKind::EOF )
    }

    fn is_eol( &self ) -> bool {
        self.is( TokenKind::EOL )
    }

    fn eat( &mut self, kind: TokenKind ) -> Option< Token > {
        if let Some( &t ) = self.get() {
            if t.kind == kind {
                // println!( "eaten={:?}", kind );
                self.last_eaten_token_pos = self.pos;
                self.pos += 1;
                self.skip();

                return Some( t );
            }
        }

        None
    }

    fn eat_any( &mut self ) -> Option< Token > {
        if let Some( &t ) = self.get() {
            // let kind = self.curr();
            self.last_eaten_token_pos = self.pos;
            self.pos += 1;
            self.skip();

            // println!( "eaten_any={:?}", kind );

            return Some( t );
        }

        None
    }

    fn start( &self ) -> Node {
        Node::new( self.pos )
    }

    fn end( &self, node: Node, kind: SyntaxKind ) -> Node {
        node.end( self.last_eaten_token_pos, kind )
    }

    fn eol( &mut self ) -> Option< Token > {
        self.eat( TokenKind::EOL )
    }

    fn indent( &mut self ) {

    }

    fn dedent( &mut self ) {

    }

    fn set_skipper( &mut self, skipper: Skipper ) -> Skipper {
        let old_skipper = self.skipper;

        self.skipper = skipper;
        self.skip();

        old_skipper
    }

    fn skip( &mut self ) {
        match self.skipper {
            Skipper::Inline => {
                while self.is( T![ ] ) {
                    self.pos += 1;
                }
            },

            Skipper::Block => {
                while self.is( T![ ] ) || self.is( TokenKind::EOL ) || self.is( TokenKind::Comment ) {
                    self.pos += 1;
                }
            },

            Skipper::None => (),
        }

    }

}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_source_file( p: &mut Parser ) -> Node {
    let mut n = p.start();
    let skipper = p.set_skipper( Skipper::Block );

    while !p.is_eof() {
        n.add( r_statement( p ) );
    }

    p.set_skipper( skipper );

    p.end( n, SyntaxKind::SourceFile )
}

fn r_statement( p: &mut Parser ) -> Node {
    match p.curr() {
        Some( T![ fn ] ) => r_fn( p ),
        _ => unreachable!()
    }
}

fn r_fn( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.eat( T![ fn ] );
    p.eat( TokenKind::Id );
    p.eat( TokenKind::EOL );
    n.add( r_block( p ) );

    p.end( n, SyntaxKind::Fn )
}

fn r_if( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.eat( T![ if ] );
    n.add( r_inline_expr( p ) );
    p.eol();
    n.add( r_block( p ) );

    p.end( n, SyntaxKind::If )
}

fn r_while( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.eat( T![ while ] );
    n.add( r_inline_expr( p ) );
    p.eol();
    n.add( r_block( p ) );

    p.end( n, SyntaxKind::While )
}

fn r_inline_expr( p: &mut Parser ) -> Node {
    let skipper = p.set_skipper( Skipper::Inline );

    let expr = r_inline_binary( p, 0 );

    p.set_skipper( skipper );

    expr
}

fn infix_binding_power( kind: Option< TokenKind > ) -> Option< ( i8, i8 ) > {
    match kind {
        Some( T![ || ] ) => Some( ( 1, 2 ) ),
        Some( T![ && ] ) => Some( ( 3, 4 ) ),
        Some( T![ == ] | T![ != ] ) => Some( ( 5, 6 ) ),
        Some( T![ < ] | T![ <= ] | T![ > ] | T![ >= ] ) => Some( ( 7, 8 ) ),
        Some( T![ + ] | T![ - ] ) => Some( ( 9, 10 ) ),
        Some( T![ * ] | T![ / ] | T![ % ] ) => Some( ( 11, 12 ) ),
        _ => None,
    }
}

fn r_inline_binary( p: &mut Parser, min_bp: i8 ) -> Node {
    let mut left = r_inline_unary( p );

    while let Some ( ( left_bp, right_bp ) ) = infix_binding_power( p.curr() ) {
        if left_bp < min_bp {
            break;

        } else {
            let mut n = left.start();

            n.add( left );
            p.eat_any();
            n.add( r_inline_binary( p, right_bp ) );
            left = p.end( n, SyntaxKind::InlineBinary );
        }

    }

    left
}

fn r_inline_unary( p: &mut Parser ) -> Node {
    if p.curr() == Some( T![ + ] ) || p.curr() == Some( T![ - ] ) || p.curr() == Some( T![ ! ] ) {
        let mut n = p.start();
        p.eat_any();
        n.add( r_inline_unary( p ) );
        p.end( n, SyntaxKind::InlineUnary )

    } else {
        r_inline_primary( p )
    }
}

fn r_inline_primary( p: &mut Parser ) -> Node {
    let mut expr = match p.curr() {
        Some( T![ '(' ] ) => r_inline_subexpr( p ),
        Some( TokenKind::Number ) => r_inline_number( p ),
        Some( TokenKind::Id ) => r_inline_var( p ),
        Some( T![ '"' ] ) => r_inline_string( p ),
        _ => unreachable!(),
    };

    loop {
        if p.is( T![ . ] ) {
            let mut n = expr.start();
            n.add( expr );
            p.eat( T![ . ] );
            p.eat( TokenKind::Id );

            if p.is( T![ '(' ] ) {
                n.add( r_inline_args( p ) );

                expr = p.end( n, SyntaxKind::InlineMethodCall );

            } else {
                expr = p.end( n, SyntaxKind::InlineField );
            }

        } else if p.is( T![ '(' ] ) {
            let mut n = expr.start();

            n.add( r_inline_args( p ) );

            expr = p.end( n, SyntaxKind::InlineCall );

        } else {
            break;
        }
    }

    expr

}

fn r_inline_subexpr( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.eat( T![ '(' ] );
    n.add( r_inline_expr( p ) );
    p.eat( T![ ')' ] );

    p.end( n, SyntaxKind::InlineSubexpr )
}

fn r_inline_number( p: &mut Parser ) -> Node {
    let n = p.start();

    p.eat( TokenKind::Number );

    p.end( n, SyntaxKind::InlineNumber )
}

fn r_inline_var( p: &mut Parser ) -> Node {
    let n = p.start();

    p.eat( TokenKind::Id );

    p.end( n, SyntaxKind::InlineVar )
}

fn r_inline_args( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.eat( T![ '(' ] );
    while !p.is( T![')'] ) && !p.is_eol() {
        n.add( r_inline_arg( p ) );
        p.eat( T![ , ] );
    }
    p.eat( T![ ')' ] );

    p.end( n, SyntaxKind::InlineArgs )
}

fn r_inline_arg( p: &mut Parser ) -> Node {
    let mut n = p.start();

    n.add( r_inline_expr( p ) );

    p.end( n, SyntaxKind::InlineArg )
}

fn r_inline_string( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.eat( T![ '"' ] );

    while !p.is_eol() {
        match p.curr() {
            Some( T![ '"' ] ) => {
                p.eat( T![ '"' ] );
                break;
            }
            Some( TokenKind::StringFragment ) => {
                n.add( r_inline_string_fragment( p ) );
            }
            Some( TokenKind::DollarOpenBrace ) => {
                n.add( r_inline_string_expr( p ) );
            }
            _ => (),
        }
    }

    p.end( n, SyntaxKind::InlineString )
}

fn r_inline_string_fragment( p: &mut Parser ) -> Node {
    let n = p.start();

    p.eat( TokenKind::StringFragment );

    p.end( n, SyntaxKind::InlineStringFragment )
}

fn r_inline_string_expr( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.eat( TokenKind::DollarOpenBrace );
    n.add( r_inline_expr( p ) );
    p.eat( T![ '}' ] );

    p.end( n, SyntaxKind::InlineStringExpr )
}

fn r_block( p: &mut Parser ) -> Node {
    let mut n = p.start();

    let skipper = p.set_skipper( Skipper::Block );

    p.indent();
    while !p.is_eof() {
        n.add( r_block_expr( p ) );
    }
    p.dedent();

    p.set_skipper( skipper );

    p.end( n, SyntaxKind::Block )
}

fn r_expr( p: &mut Parser ) -> Node {
    let mut n = p.start();

    let skipper = p.set_skipper( Skipper::Inline );

    n.add( r_inline_expr( p ) );
    p.eat( TokenKind::EOL );

    p.set_skipper( skipper );

    p.end( n, SyntaxKind::Expr )
}

fn r_block_expr( p: &mut Parser ) -> Node {
    match p.curr() {
        Some( T![ if ] ) => r_if( p ),
        Some( T![ while ] ) => r_while( p ),
        _ => r_expr( p ),
    }
}
