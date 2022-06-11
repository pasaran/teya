use std::fmt;

use crate::{ SyntaxKind, Token };

pub enum SyntaxElement< 'a > {
    Node( SyntaxNode< 'a > ),
    Token( Token< 'a > ),
}

pub struct SyntaxNode< 'a > {
    kind: SyntaxKind,
    children: Vec< SyntaxElement< 'a > >,
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

}
