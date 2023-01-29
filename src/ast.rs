use crate::{ SyntaxNode, SyntaxKind, SyntaxElement, Token, TokenKind, TokenSet, AstNode };
use gen_ast::ast;

//  ---------------------------------------------------------------------------------------------------------------  //

ast! {
    Root {
        source_file: SourceFile,
    }
}

ast! {
    SourceFile {
        items: *Item,
    }
}

ast! {
    Fn {
        name: Name,
    }
}

ast! {
    Name {
        id: #Id,
    }
}

ast! {
    InlineBinary {
        left: Name,
        op: #( Plus | Minus | Star ),
        right: Name,
    }
}

ast! {
    Struct {
        name: Name,
    }
}

ast! {
    Item = Fn | Struct
}
