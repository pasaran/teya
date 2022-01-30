mod token_stream;

use token_stream::TokenStream;
use token_stream::TokenKind;

//  ---------------------------------------------------------------------------------------------------------------  //

fn main() {
    let ts = TokenStream::new( "\"Foo ${ { { \"foo-${ x }\" } } } Bar $ } Boo\"" );

    for token in ts {
        println!( "{:?}", token );
    }

    println!( "{:?}", T![ ] );
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