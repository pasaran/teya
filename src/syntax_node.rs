use std::fmt;

use crate::{ SyntaxKind, Token, TokenKind, token_set::TokenSet };

//  ---------------------------------------------------------------------------------------------------------------  //

type InputPos = ( usize, usize );
type InputRange = ( InputPos, InputPos );

struct ParseTree< 'a > {
    input: &'a str,
    root: SyntaxNode< 'a >,
}

impl < 'a > ParseTree < 'a > {

    fn get_token_range( &self, token: Token ) -> InputRange {
        (
            pos_to_coords( self.input, token.start ),
            pos_to_coords( self.input, token.end )
        )
    }

}

fn pos_to_coords( s: &str, pos: usize ) -> InputPos {
    let s = &s[ 0 .. pos ];
    let line_ends = s.match_indices( | c | c == '\n' );

    let mut y = 0;
    let mut x = 0;
    for ( l, delim ) in line_ends {
        x = l + delim.len();
        y += 1;
    }

    ( pos - x, y )
}

#[ cfg( test ) ]
mod tests {
    use super::*;

    #[ test ]
    fn test_pos_to_coords() {

        assert_eq!( pos_to_coords( "abcd\nefgh", 0 ), ( 0, 0 ) );
        assert_eq!( pos_to_coords( "abcd\nefgh", 2 ), ( 2, 0 ) );
        assert_eq!( pos_to_coords( "abcd\nefgh", 4 ), ( 4, 0 ) );
        assert_eq!( pos_to_coords( "abcd\nefgh", 5 ), ( 0, 1 ) );
        assert_eq!( pos_to_coords( "abcd\nefgh\n", 9 ), ( 4, 1 ) );
        assert_eq!( pos_to_coords( "abcd\nefgh\nijkl", 10 ), ( 0, 2 ) );
        assert_eq!( pos_to_coords( "abcd\nefgh\nijkl", 12 ), ( 2, 2 ) );

    }

}

//  ---------------------------------------------------------------------------------------------------------------  //

pub enum SyntaxElement< 'a > {
    Node( SyntaxNode< 'a > ),
    Token( Token< 'a > ),
}

pub struct SyntaxNode< 'a > {
    pub kind: SyntaxKind,
    pub children: Vec< SyntaxElement< 'a > >,
}

impl < 'a > SyntaxNode< 'a > {

    pub fn new( kind: SyntaxKind ) -> Self {
        SyntaxNode {
            kind,
            children: Vec::new(),
        }
    }

    pub fn push( &mut self, element: SyntaxElement< 'a > ) {
        self.children.push( element );
    }

    pub fn find_nodes( &'a self, kind: SyntaxKind ) -> Vec< &'a SyntaxNode< 'a > > {
        self.children
            .iter()
            .filter_map( | e | match e {
                SyntaxElement::Node( node ) if node.kind == kind => Some( node ),
                _ => None,
            } )
            .collect()
    }

    pub fn find_node_by_index( &'a self, kind: SyntaxKind, index: usize ) -> Option< &'a SyntaxNode< 'a > > {
        self.children
            .iter()
            .filter_map( | e | match e {
                SyntaxElement::Node( node ) if node.kind == kind => Some( node ),
                _ => None,
            } )
            .nth( index )
    }

    pub fn find_token( &'a self, kind: TokenKind ) -> Option< &'a Token< 'a > > {
        self.children
            .iter()
            .filter_map( | e | match e {
                SyntaxElement::Token( token ) if token.kind == kind => Some( token ),
                _ => None,
            } )
            .nth( 0 )
    }

    pub fn find_token_in_set( &'a self, ts: TokenSet ) -> Option< &'a Token< 'a > > {
        self.children
            .iter()
            .filter_map( | e | match e {
                SyntaxElement::Token( token ) if ts.contains( token.kind ) => Some( token ),
                _ => None,
            } )
            .nth( 0 )
    }

}

impl < 'a >fmt::Debug for SyntaxNode< 'a > {
    fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
        write!( f , "{}", format_node( self, 0 ) )
    }
}

fn format_element( element: &SyntaxElement, indent: usize ) -> String {
    match element {
        SyntaxElement::Node( node ) => format_node( node, indent ),
        SyntaxElement::Token( token ) => format_token( token, indent ),
    }
}

fn format_node( node: &SyntaxNode, indent: usize ) -> String {
    format!( "{}{:?}\n{}", " ".repeat( indent * 4 ), node.kind, format_children( node, indent ) )
}

fn format_token( token: &Token, indent: usize ) -> String {
    format!( "{}{:?}", " ".repeat( indent * 4 ), token )
}

fn format_children( node: &SyntaxNode, indent: usize ) -> String {
    node.children
        .iter()
        .map( | element | format_element( element, indent + 1 ) )
        .collect::< Vec< String> >()
        .join( "\n" )
}
