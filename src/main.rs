mod token_kind;
mod lexer;
mod parser;
mod syntax_kind;
mod syntax_node;

pub use syntax_kind::{ SyntaxKind };
pub use token_kind::{ TokenKind };
pub use lexer::{ Lexer, Token };
pub use syntax_node::{ SyntaxNode, SyntaxElement };

use std::fs;

use crate::parser::Parser;

//  ---------------------------------------------------------------------------------------------------------------  //

fn main() {
    let content = fs::read_to_string( "./tests/01.teya" ).unwrap();
    let mut parser = Parser::new( &content );

    let node = parser.parse( SyntaxKind::SourceFile );
    println!( "{:?}", node );

    //  let ts = TokenStream::new( content.as_str() );
    // for token in ts {
    //     println!( "{:?}", token );
    // }

    // println!( "{:?}", T![ ] );
}

/*

//  "Foo ${ { { "foo-bar-${ x }-zad" } } } Baz $ Boo"

'"' Normal -- [].clear() -- open string
'Foo ' StringFragment
'${' StringFragment -- open expr, n_curlies = 1
' ' StringExpr
'{' StringExpr -- n_curlies = 2, []
' ' StringExpr
'{' StringExpr -- n_curlies = 3, []
' ' StringExpr
'"' StringExpr -- open nested string, n_curlies = 0, [ 3 ]
'foo-bar-' StringFragment
'${' StringFragment -- open expr, n_curlies = 1, [ 3 ]
' ' StringExpr
'x' StringExpr
' ' StringExpr
'}' StringExpr -- close expr, n_curlies = 0, [ 3 ]
'-zad' StringFragment
'"' StringFragment -- close nested string, n_curlies = 3, []
' ' StringExpr
'}' StringExpr -- n_curlies = 2, []
' ' StringExpr
'}' StringExpr -- n_curlies = 1, []
' ' StringExpr
'}' StringExpr -- close expr, n_curlies = 0, []
' Baz $ Boo' StringFragment
'"' StringFragment -- close string, [].is_empty()

*/