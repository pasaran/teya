use crate::{ SyntaxNode, SyntaxKind, Token, TokenKind };

//  ---------------------------------------------------------------------------------------------------------------  //

pub struct Root< 'a > {
    node: &'a SyntaxNode< 'a >,
}

impl < 'a > Root < 'a > {

    pub fn new( node: &'a SyntaxNode< 'a > ) -> Self {
        Root { node }
    }

    pub fn source_file( &'a self ) -> Option< SourceFile< 'a > > {
        self.node.find_node( SyntaxKind::SourceFile )
            .map( | node | SourceFile { node } )
    }

}

//  ---------------------------------------------------------------------------------------------------------------  //

pub struct SourceFile< 'a > {
    node: &'a SyntaxNode< 'a >,
}

impl < 'a > SourceFile< 'a > {

    pub fn fns( &'a self ) -> Vec< Fn< 'a > > {
        self.node.find_nodes( SyntaxKind::Fn )
            .iter()
            .map( | node | Fn { node } )
            .collect()
    }

    pub fn fn_( &'a self, index: usize ) -> Option< Fn< 'a > > {
        self.node.find_nodes( SyntaxKind::Fn )
            .iter()
            .map( | node | Fn { node } )
            .nth( index )
    }

}

//  ---------------------------------------------------------------------------------------------------------------  //

pub struct Fn< 'a > {
    node: &'a SyntaxNode< 'a >,
}

impl < 'a > Fn < 'a > {

    pub fn name( &'a self ) -> Option< Name< 'a > > {
        self.node.find_node( SyntaxKind::Name )
            .map( | node | Name { node } )
    }

}

//  ---------------------------------------------------------------------------------------------------------------  //

pub struct Name< 'a > {
    node: &'a SyntaxNode< 'a >,
}

impl < 'a > Name < 'a > {

    pub fn id( &'a self ) -> Option< &'a Token< 'a > > {
        self.node.find_token( TokenKind::Id )
    }

}