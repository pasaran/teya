use crate::{ SyntaxKind, SyntaxNode };

pub trait AstNode< 'a > {

    fn kind() -> SyntaxKind;

    fn syntax( &'a self ) -> &'a SyntaxNode< 'a >;

    fn can_cast( kind: SyntaxKind ) -> bool
        where Self: Sized;

    fn cast( node: &'a SyntaxNode ) -> Option< Self >
        where Self: Sized;

    fn walk< T: AstNode< 'a > >( &'a self, callback: fn( &'a T ) );

}
