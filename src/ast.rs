use crate::{ SyntaxNode, SyntaxKind, SyntaxElement, Token, TokenKind, TokenSet, AstNode };
// use gen_ast::ast;

//  ---------------------------------------------------------------------------------------------------------------  //

// ast! {
//     Root {
//         source_file: SourceFile,
//     }
// }

// ast! {
//     SourceFile {
//         items: *Item,
//     }
// }

// ast! {
//     Fn {
//         name: Name,
//     }
// }

// ast! {
//     Name {
//         id: #Id,
//     }
// }

// ast! {
//     InlineBinary {
//         left: Name, // 0
//         op: #( Plus | Minus | Star ),
//         right: Name, // 1
//     }
// }

// ast! {
//     Struct {
//         name: Name,
//     }
// }

// ast! {
//     Item = Fn | Struct
// }

//  ---------------------------------------------------------------------------------------------------------------  //

pub struct Root< 'a > {
    node: &'a SyntaxNode< 'a >,
    pub source_file: Option< SourceFile< 'a > >,
}

impl< 'a > AstNode< 'a > for Root< 'a > {

    fn kind() -> SyntaxKind {
        SyntaxKind::Root
    }

    fn syntax( &'a self ) -> &SyntaxNode< 'a > { &self.node }

    fn can_cast( kind: SyntaxKind ) -> bool {
        SyntaxKind::Root == kind
    }

    fn cast( node: &'a SyntaxNode< 'a > ) -> Option< Root< 'a > > {
        if Root::can_cast( node.kind ) {
            let source_file = node.children
                .iter()
                .filter_map( | e | match e {
                    SyntaxElement::Node( node ) => SourceFile::cast( node ),
                    _ => None,
                } )
                .nth( 0 );

            Some( Self {
                node,
                source_file,
            } )

        } else {
            None
        }
    }

    fn walk< T: AstNode< 'a > >(&'a self, callback: fn(&'a T)) {
        if let Some( t ) = self.source_file.as_ref() {
            t.walk( callback );
        }
        //  if T::kind() == SyntaxKind::Root {
        if T::kind() == Self::kind() {
            let x = unsafe { std::mem::transmute::< &Self, &T >( self ) };
            callback( x );
        }
    }

}

pub struct SourceFile< 'a > {
    node: &'a SyntaxNode< 'a >,
    pub items: Vec< Item< 'a > >,
}

impl< 'a > AstNode< 'a > for SourceFile< 'a > {

    fn kind() -> SyntaxKind {
        SyntaxKind::SourceFile
    }

    fn syntax( &'a self ) -> &'a SyntaxNode< 'a > { &self.node }

    fn can_cast( kind: SyntaxKind ) -> bool {
        SyntaxKind::SourceFile == kind
    }

    fn cast(node: &'a SyntaxNode< 'a >) -> Option< SourceFile< 'a > > {
        if SourceFile::can_cast( node.kind ) {
            let items = node.children
                .iter()
                .filter_map( | e | match e {
                    SyntaxElement::Node( node ) => Item::cast( node ),
                    _ => None,
                } )
                .collect();

            Some( SourceFile {
                node,
                items,
            } )

        } else {
            None
        }
    }

    fn walk< T: AstNode< 'a > >(&'a self, callback: fn(&'a T)) {
        println!( "SourceFile::walk ");

        self.items
            .iter()
            .for_each( | item | {
                item.walk( callback );
        } );
        if T::kind() == Self::kind() {
            let x = unsafe { std::mem::transmute::< &Self, &T >( self ) };
            callback( x );
        }
    }

}

pub struct Fn< 'a > {
    node: &'a SyntaxNode< 'a >,
    pub name: Option< Name< 'a > >,
}

impl< 'a > AstNode< 'a > for Fn< 'a > {

    fn kind() -> SyntaxKind {
        SyntaxKind::Fn
    }

    fn syntax( &'a self ) -> &'a SyntaxNode< 'a > { &self.node }

    fn can_cast( kind: SyntaxKind ) -> bool {
        SyntaxKind::Fn == kind
    }

    fn cast( node: &'a SyntaxNode< 'a > ) -> Option< Self > {
        if Self::can_cast( node.kind ) {
            let name = node.children
                .iter()
                .filter_map( | e | match e {
                    SyntaxElement::Node( node ) => Name::cast( node ),
                    _ => None,
                } )
                .nth( 0 );

            Some( Self {
                node,
                name,
            } )

        } else {
            None
        }
    }

    fn walk< T: AstNode< 'a > >(&'a self, callback: fn(&'a T)) {
        println!( "Fn::walk ");

        if let Some( t ) = self.name.as_ref() {
            t.walk( callback );
        }
        if T::kind() == Self::kind() {
            let x = unsafe { std::mem::transmute::< &Self, &T >( self ) };
            callback( x );
        }
    }

}

pub struct Name< 'a > {
    node: &'a SyntaxNode< 'a >,
}

impl<'a> Name<'a> {

    pub fn id( &'a self ) -> Option< &'a Token< 'a > > {
        self.node
            .find_token( TokenKind::Id )
    }

}

impl< 'a > AstNode< 'a > for Name< 'a > {

    fn kind() -> SyntaxKind {
        SyntaxKind::Name
    }

    fn syntax( &'a self ) -> &'a SyntaxNode< 'a > { &self.node }

    fn can_cast( kind: SyntaxKind ) -> bool {
        SyntaxKind::Name == kind
    }

    fn cast( node: &'a SyntaxNode< 'a > ) -> Option< Self > {
        if Self::can_cast( node.kind ) {
            Some( Self {
                node,
            } )

        } else {
            None
        }
    }

    fn walk<T>(&'a self, _callback: fn(&'a T)) where T: AstNode<'a> {

    }

}

pub struct InlineBinary< 'a > {
    node: &'a SyntaxNode< 'a >,
    pub left: Option< Name< 'a > >,
    pub right: Option< Name< 'a > >,
}

const TS_INLINE_BINARY_OP: TokenSet = TokenSet::new( &[
    TokenKind::Plus,
    TokenKind::Minus,
    TokenKind::Star
] );

impl<'a> InlineBinary<'a> {

    pub fn op( &'a self ) -> Option< &'a Token< 'a > > {
        self.node.find_token_in_set( TS_INLINE_BINARY_OP )
    }

}

impl< 'a > AstNode< 'a > for InlineBinary< 'a > {

    fn kind() -> SyntaxKind {
        SyntaxKind::InlineBinary
    }

    fn syntax( &'a self ) -> &'a SyntaxNode< 'a > { &self.node }

    fn can_cast( kind: SyntaxKind ) -> bool {
        SyntaxKind::InlineBinary == kind
    }

    fn cast( node: &'a SyntaxNode< 'a > ) -> Option< Self > {
        if Self::can_cast( node.kind ) {
            let mut names = node.children
                .iter()
                .filter_map( | e | match e {
                    SyntaxElement::Node( node ) => Name::cast( node ),
                    _ => None,
                } );

            let left = names.nth( 0 );
            let right = names.nth( 1 );

            Some( Self {
                node,
                left,
                right,
            } )

        } else {
            None
        }
    }

    fn walk< T: AstNode< 'a > >( &'a self, callback: fn( &'a T ) ) {
        if let Some( t ) = self.left.as_ref() {
            t.walk( callback );
        }
        if let Some( t ) = self.right.as_ref() {
            t.walk( callback );
        }
        if T::kind() == Self::kind() {
            let x = unsafe { std::mem::transmute::< &Self, &T >( self ) };
            callback( x );
        }
    }

}

pub struct Struct< 'a > {
    node: &'a SyntaxNode< 'a >,
    pub name: Option< Name< 'a > >,
}

impl< 'a > AstNode< 'a > for Struct< 'a > {

    fn kind() -> SyntaxKind {
        SyntaxKind::Struct
    }

    fn syntax( &'a self ) -> &'a SyntaxNode< 'a > { &self.node }

    fn can_cast( kind: SyntaxKind ) -> bool {
        SyntaxKind::Struct == kind
    }

    fn cast( node: &'a SyntaxNode< 'a > ) -> Option< Self > {
        if Self::can_cast( node.kind ) {
            let name = node.children
                .iter()
                .filter_map( | e | match e {
                    SyntaxElement::Node( node ) => Name::cast( node ),
                    _ => None,
                } )
                .nth( 0 );


            Some( Self {
                node,
                name,
            } )

        } else {
            None
        }
    }

    fn walk<T>(&'a self, _callback: fn(&'a T)) where T: AstNode<'a> {

    }

}

pub enum Item< 'a > {
    Fn( Fn< 'a > ),
    Struct( Struct< 'a > ),
}

impl<'a> AstNode<'a> for Item<'a> {

    fn kind() -> SyntaxKind {
        SyntaxKind::Item
    }

    fn syntax( &'a self) -> &'a SyntaxNode< 'a > {
        match self {
            Item::Fn(x) => x.syntax(),
            Item::Struct(x) => x.syntax(),
        }
    }

    fn can_cast( kind: SyntaxKind ) -> bool {
        match kind {
            SyntaxKind::Fn => true,
            SyntaxKind::Struct => true,
            _ => false,
        }
    }

    fn cast( node: &'a SyntaxNode< 'a > ) -> Option< Self > {
        if let Some( x ) = Fn::cast( node ) {
            return Some( Item::Fn( x ) );
        }
        if let Some( x ) = Struct::cast( node ) {
            return Some( Item::Struct( x ) );
        }
        return None;
    }

    fn walk<T>( &'a self, callback: fn( &'a T ) ) where T: AstNode< 'a > {
        println!( "Item::walk" );

        match self {
            Item::Fn(x) => x.walk(callback),
            Item::Struct(x) => x.walk(callback),
        }
        if T::kind() == Self::kind() {
            let x = unsafe { std::mem::transmute::< &Self, &T >( self ) };
            callback( x );
        }
    }

}
