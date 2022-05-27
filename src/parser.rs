use crate::{ Lexer, Token, TokenKind, T, SyntaxKind, SyntaxNode, SyntaxElement };

// #[derive(Clone, Copy)]
// enum Skipper {
//     None,
//     Inline,
//     Block,
// }

struct Node {
    start: usize,
    end: usize,
    kind: SyntaxKind,
    children: Vec< Node >,
}

impl Node {

    fn new( start: usize ) -> Self {
        Node {
            start,
            end: 0,
            kind: SyntaxKind::None,
            children: Vec::new(),
        }
    }

    fn start( &self ) -> Node {
        Node::new( self.start )
    }

    fn end( mut self, end: usize, kind: SyntaxKind ) -> Self {
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

    // skipper: Skipper,

    last_eaten_token_pos: usize,

    current_indent: usize,
    indents: Vec< usize >,
    n_indent: usize,
    n_dedent: usize,
}

impl Parser {

    pub fn new( input: &str ) -> Self {
        Parser {
            //  input,
            tokens: Lexer::new( input ).collect(),

            pos: 0,

            // skipper: Skipper::None,

            last_eaten_token_pos: 0,

            current_indent: 0,
            indents: vec![],
            n_indent: 0,
            n_dedent: 0,
        }
    }

    pub fn parse( &mut self, kind: SyntaxKind ) -> SyntaxNode {
        let node = match kind {
            SyntaxKind::SourceFile => r_source_file( self ),
            SyntaxKind::InlineExpr => r_inline_expr( self ),
            _ => panic!( "Invalid kind {:?}", kind ),
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

    fn nth( &self, n: usize ) -> Option< &Token > {
        self.tokens.get( self.pos + n )
    }

    fn nth_kind( &self, n: usize ) -> Option< TokenKind > {
        self.nth( n ).map( | t | t.kind )
    }

    fn kind( &self ) -> Option< TokenKind > {
        self.nth_kind( 0 )
    }

    fn is_kind( &self, kind: TokenKind ) -> bool {
        self.nth( 0 ).map_or( false, | t | t.kind == kind )
    }

    fn is_eof( &self ) -> bool {
        self.is_kind( TokenKind::EOF )
    }

    fn is_eol( &self ) -> bool {
        self.is_kind( TokenKind::EOL )
    }

    fn mov( &mut self, n: usize ) {
        self.pos += n;
    }

    fn eat( &mut self, kind: TokenKind ) -> Option< Token > {
        if let Some( &t ) = self.nth( 0 ) {
            if t.kind == kind {
                // println!( "eaten={:?}", kind );
                self.last_eaten_token_pos = self.pos;
                self.mov( 1 );
                self.skip_spaces();

                return Some( t );
            }
        }

        None
    }

    fn eat_any( &mut self ) -> Option< Token > {
        if let Some( &t ) = self.nth( 0 ) {
            // println!( "eaten_any={:?}", self.kind() );
            self.last_eaten_token_pos = self.pos;
            self.mov( 1 );
            self.skip_spaces();


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

    fn eol( &mut self ) {
        self.last_eaten_token_pos = self.pos;
        // println!( "EOL!!!" );
        // println!( "{:?} {:?} {:?}", self.nth_kind( 1 ), self.nth_kind( 2 ), self.indents );
        while self.is_eol() {
            self.mov( 1 );

            let mut indent = 0;
            let token = self.nth( 0 );
            if let Some( token ) = token {
                if token.kind == T![ ] {
                    indent = token.end - token.start;

                    self.mov( 1 );

                }
            }

            match self.kind() {
                Some( TokenKind::EOL ) => {
                    continue;
                }

                Some( TokenKind::Comment ) => {
                    self.mov( 1 );
                    continue;
                }

                _ => {
                    self.n_indent = 0;
                    self.n_dedent = 0;

                    // println!( "indent={} current_indent={} indents={:?}", indent, self.current_indent, self.indents );

                    if indent > self.current_indent {
                        self.indents.push( self.current_indent );
                        self.current_indent = indent;

                        self.n_indent = 1;

                    } else if indent < self.current_indent {
                        while let Some( last ) = self.indents.pop() {
                            self.current_indent = last;
                            self.n_dedent += 1;

                            if last == indent {
                                break;
                            }
                        }
                    }

                }
            }

        }

        // println!( "n_indent={} n_dedent={}", self.n_indent, self.n_dedent );

    }

    fn is_indent( &self ) -> bool {
        self.n_indent > 0
    }

    fn is_dedent( &self ) -> bool {
        self.n_dedent > 0
    }

    fn indent( &mut self ) {
        if self.is_indent() {
            self.n_indent -= 1;

        } else {
            panic!( "No indent ");
        }
    }

    fn dedent( &mut self ) {
        if self.is_dedent() {
            self.n_dedent -= 1;

        } else {
            panic!( "No dedent" );
        }
    }

    // fn set_skipper( &mut self, skipper: Skipper ) -> Skipper {
    //     let old_skipper = self.skipper;

    //     self.skipper = skipper;
    //     self.skip_spaces();

    //     old_skipper
    // }

    fn skip_spaces( &mut self ) {
        if self.is_kind( T![ ] ) {
            self.pos += 1;
        }
    }

}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_source_file( p: &mut Parser ) -> Node {
    let mut n = p.start();

    while !p.is_eof() {
        n.add( r_statement( p ) );
    }

    n.end( p.pos, SyntaxKind::SourceFile )
    // p.end( n, SyntaxKind::SourceFile )
}

fn r_statement( p: &mut Parser ) -> Node {
    match p.kind() {
        Some( T![ fn ] ) => r_fn( p ),
        _ => unreachable!( "{:?}", p.kind() )
    }
}

fn r_fn( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.eat( T![ fn ] );
    p.eat( TokenKind::Id );
    n.add( r_fn_args( p ) );
    if p.is_kind( T![ -> ] ) {
        n.add( r_fn_return_type( p ) );
    }
    p.eol();
    n.add( r_block( p ) );

    p.end( n, SyntaxKind::Fn )
}

fn r_fn_args( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.eat( T![ '(' ] );
    while !p.is_eol() && !p.is_kind( T![ ')' ] ) {
        n.add( r_fn_arg( p ) );
        p.eat( T![ , ] );
    }
    p.eat( T![ ')' ] );

    p.end( n, SyntaxKind::FnArgs )
}

fn r_fn_arg( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.eat( TokenKind::Id );
    p.eat( T![ : ] );
    n.add( r_typeref( p ) );

    p.end( n, SyntaxKind::FnArg )
}

fn r_fn_return_type( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.eat( T![ -> ] );
    n.add( r_typeref( p ) );

    p.end( n, SyntaxKind::FnReturnType )
}

fn r_typeref( p: &mut Parser ) -> Node {
    let n = p.start();

    p.eat( TokenKind::Id );

    p.end( n, SyntaxKind::TypeRef )
}

fn r_let( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.eat( T![ let ] );
    p.eat( TokenKind::Id );
    if p.is_kind( T![ : ] ) {
        p.eat( T![ : ] );
        n.add( r_typeref( p ) );
    }
    p.eat( T![ = ] );
    n.add( r_expr( p ) );

    p.end( n, SyntaxKind::Let )
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

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_inline_expr( p: &mut Parser ) -> Node {
    r_inline_binary( p, 0 )
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

    while let Some ( ( left_bp, right_bp ) ) = infix_binding_power( p.kind() ) {
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
    match p.kind() {
        Some( T![ + ] | T![ - ] |  T![ ! ] ) => {
            let mut n = p.start();
            p.eat_any();
            n.add( r_inline_unary( p ) );
            p.end( n, SyntaxKind::InlineUnary )
        }
        _ => r_inline_primary( p )
    }
}

fn r_inline_primary( p: &mut Parser ) -> Node {
    let mut expr = match p.kind() {
        Some( T![ '(' ] ) => r_inline_subexpr( p ),
        Some( TokenKind::Number ) => r_inline_number( p ),
        Some( TokenKind::Id ) => r_inline_var( p ),
        Some( T![ '"' ] ) => r_inline_string( p ),
        _ => unreachable!(),
    };

    loop {
        if p.is_kind( T![ . ] ) {
            let mut n = expr.start();
            n.add( expr );
            p.eat( T![ . ] );
            p.eat( TokenKind::Id );

            if p.is_kind( T![ '(' ] ) {
                n.add( r_inline_args( p ) );

                expr = p.end( n, SyntaxKind::InlineMethodCall );

            } else {
                expr = p.end( n, SyntaxKind::InlineField );
            }

        } else if p.is_kind( T![ '(' ] ) {
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
    while !p.is_kind( T![ ')' ] ) && !p.is_eol() {
        n.add( r_inline_arg( p ) );
        p.eat( T![ , ] );
    }
    p.eat( T![ ')' ] );

    p.end( n, SyntaxKind::CallArgs )
}

fn r_inline_arg( p: &mut Parser ) -> Node {
    let mut n = p.start();

    n.add( r_inline_expr( p ) );

    p.end( n, SyntaxKind::CallArg )
}

fn r_inline_string( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.eat( T![ '"' ] );

    while !p.is_eol() {
        match p.kind() {
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

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_block( p: &mut Parser ) -> Node {
    let mut n = p.start();

    p.indent();
    while !p.is_eof() && !p.is_dedent() {
        n.add( r_block_statement( p ) );
    }
    p.dedent();

    p.end( n, SyntaxKind::Block )
}

fn r_expr( p: &mut Parser ) -> Node {
    let mut n = p.start();

    n.add( r_inline_expr( p ) );
    p.eol();

    p.end( n, SyntaxKind::Expr )
}

fn r_block_statement( p: &mut Parser ) -> Node {
    match p.kind() {
        Some( T![ if ] ) => r_if( p ),
        Some( T![ while ] ) => r_while( p ),
        Some( T![ let ] ) => r_let( p ),
        _ => r_expr( p ),
    }
}
