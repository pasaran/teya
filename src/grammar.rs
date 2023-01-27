use crate::parser::{ Parser, CompletedMarker, Skipper };
use crate::token_set::TokenSet;
use crate::{ SyntaxKind, TokenKind, ParserErrorKind, T };

fn error_block( p: &mut Parser, error_kind: ParserErrorKind ) {
//     assert!(p.at(T!['{']));
//     let m = p.start();
//     p.error(message);
//     p.bump(T!['{']);
//     expressions::expr_block_contents(p);
//     p.eat(T!['}']);
//     m.complete(p, ERROR);
}

//  ---------------------------------------------------------------------------------------------------------------  //

pub fn r_source_file( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.set_skipper( Skipper::Block );

    while !p.at_eof() {
        r_item( p );
    }

    p.expect( TokenKind::EOF );

    m.complete( p, SyntaxKind::SourceFile )
}

//  ---------------------------------------------------------------------------------------------------------------  //

const ITEM_RECOVERY_SET: TokenSet = TokenSet::new( &[
    T![ fn ],
    T![ type ],
    T![ struct ],
    T![ enum ],
    T![ ; ],
] );

fn r_item( p: &mut Parser ) {
    if r_opt_item( p ) {
        return;
    }

    match p.current() {
        // Some( T![ '{' ] ) => error_block( p, ParserErrorKind::ItemRequired ),

        // Some( T![ '}' ] if !stop_on_r_curly => {
        //     let e = p.start();
        //     p.error("unmatched `}`");
        //     p.bump(T!['}']);
        //     e.complete(p, ERROR);
        // }

        TokenKind::EOF | T![ '}' ] => p.error( ParserErrorKind::ItemRequired ),

        _ => p.error_and_bump( ParserErrorKind::ItemRequired ),
    }
}

fn r_opt_item( p: &mut Parser ) -> bool {
    match p.current() {
        T![ fn ] => r_fn( p ),
        T![ type ] => r_type_alias( p ),
        T![ struct ] => r_struct( p ),
        T![ enum ] => r_enum( p ),
        _ => return false,
    };

    true
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_name( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.expect( TokenKind::Id );

    m.complete( p, SyntaxKind::Name )
}

fn r_name_rec( p: &mut Parser, recovery: TokenSet ) {
    if p.at( TokenKind::Id ) {
        let m = p.start();

        p.expect( TokenKind::Id );

        m.complete(p, SyntaxKind::Name );

    } else {
        p.error_recover( ParserErrorKind::NameRequired, recovery );
    }
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_generic_params( p: &mut Parser ) {
    let m = p.start();

    assert!( p.eat( T![ < ] ) );
    loop {
        match p.current() {
            TokenKind::Id => r_generic_param( p ),
            _ => break,
        };
        p.eat( T![ , ] );
    }
    p.expect( T![ > ] );

    m.complete( p, SyntaxKind::GenericParams );
}

fn r_generic_param( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    r_name( p );

    m.complete( p, SyntaxKind::GenericParam )
}

const GENERIC_TYPE_FIRST: TokenSet = TokenSet::new( &[
    TokenKind::Id,
    T![ '(' ],
    T![ '[' ] ],
);

fn r_generic_args( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( T![ < ] ) );
    while p.at_ts( GENERIC_TYPE_FIRST ) {
        r_generic_arg( p );
        p.eat( T![ , ] );
    }
    p.expect( T![ > ] );

    m.complete( p, SyntaxKind::GenericArgs )
}

fn r_generic_arg( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    r_type( p );

    m.complete( p, SyntaxKind::GenericArg )
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_type_alias( p: &mut Parser ) {
    let m = p.start();

    assert!( p.eat( T![ type ] ) );

    r_name( p );
    p.expect( T![ = ] );
    r_type( p );

    m.complete( p, SyntaxKind::TypeAlias);
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_type( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    match p.current() {
        TokenKind::Id => r_typeref( p ),
        T![ '[' ] => r_type_array( p ),
        T![ '(' ] => r_type_tuple( p ),
        _ => unreachable!(),
    };

    m.complete( p, SyntaxKind::Type )
}

fn r_typeref( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.eat( TokenKind::Id );
    if p.at( T![ < ] ) {
        r_generic_args( p );
    }

    m.complete( p, SyntaxKind::TypeRef )
}

fn r_type_array( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( T![ '[' ] ) );
    r_type( p );
    p.expect( T![ ; ] );
    p.eat( TokenKind::Number );
    p.expect( T![ ']' ] );

    m.complete( p, SyntaxKind::TypeArray )
}

fn r_type_tuple( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    r_tuple_field( p );

    m.complete( p, SyntaxKind::TypeTuple )
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_struct( p: &mut Parser ) {
    let m = p.start();

    assert!( p.eat( T![ struct ] ) );
    //  r_name( p );
    r_name_rec( p, ITEM_RECOVERY_SET );
    r_struct_items( p );
    if p.at( T![ < ] ) {
        r_generic_params( p );
    }

    m.complete( p, SyntaxKind::Struct );
}

fn r_struct_items( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.expect( T![ '{'] );
    loop {
        match p.current() {
            TokenKind::Id => r_struct_field( p ),
            TokenKind::Fn => r_fn( p ),
            _ => break,
        };
        p.eat( T![ , ] );
    }
    p.expect( T![ '}'] );

    m.complete( p, SyntaxKind::StructItems )
}

fn r_struct_field( p: &mut Parser ) {
    let m = p.start();

    assert!( p.eat( TokenKind::Id ) );
    p.expect( T![ : ] );
    r_type( p );
    if p.at( T![ = ] ) {
        p.eat( T![ = ] );
        r_expr( p );
    }

    m.complete( p, SyntaxKind::StructField );
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_enum( p: &mut Parser ) {
    let m = p.start();

    assert!( p.eat( T![ enum ] ) );
    // . r_name( p );
    r_name_rec( p, ITEM_RECOVERY_SET );
    if p.at( T![ < ] ) {
        r_generic_params( p );
    }
    r_enum_variants( p );

    m.complete( p, SyntaxKind::Enum );
}

fn r_enum_variants( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.expect( T![ '{' ] );
    loop {
        match p.current() {
            TokenKind::Id => {
                let m = p.start();

                r_name( p );
                match p.current() {
                    T![ '(' ] => { r_tuple_fields( p ); },
                    T![ '{' ] => { r_record_fields( p ); },
                    _ => {},
                };

                m.complete( p, SyntaxKind::EnumVariant );
            },

            _ => break,
        };
        p.eat( T![ , ] );

    }
    p.expect( T![ '}' ] );

    m.complete( p, SyntaxKind::EnumVariants )
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_tuple_fields( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.expect( T![ '(' ] );
    loop {
        match p.current() {
            TokenKind::Id => r_tuple_field( p ),
            _ => break,
        };
        p.eat( T![ , ] );
    }
    p.expect( T![ ')' ] );

    m.complete( p, SyntaxKind::TupleFields )
}

fn r_tuple_field( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    r_type( p );

    m.complete( p, SyntaxKind::TupleField )
}


fn r_record_fields( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.expect( T![ '{' ] );
    loop {
        match p.current() {
            TokenKind::Id => r_record_field( p ),
            _ => break,
        };
        p.eat( T![ , ] );
    }
    p.expect( T![ '}' ] );

    m.complete( p, SyntaxKind::RecordFields )
}

fn r_record_field( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    r_name( p );
    p.expect( T![ : ] );
    r_type( p );

    m.complete( p, SyntaxKind::RecordField )
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_fn( p: &mut Parser ) {
    let m = p.start();

    assert!( p.eat( T![ fn ] ) );
    r_name_rec( p, ITEM_RECOVERY_SET );
    r_fn_params( p );
    r_opt_fn_return_type( p );
    r_block( p );

    m.complete( p, SyntaxKind::Fn );
}

fn r_fn_params( p: &mut Parser ) {
    if p.at( T![ '(' ] ) {
        let m = p.start();

        p.eat( T![ '(' ] );
        while !p.at_eof() && !p.at( T![ ')' ] ) {
            r_fn_param( p );

            p.eat( T![ , ] );
            // if !p.at_eof() {
            //     p.expect( T![ , ] );
            // }
        }
        p.expect( T![ ')' ] );

        m.complete( p, SyntaxKind::FnParams );

    } else {
        p.error( ParserErrorKind::FunctionArgumentsExpected );
    }

}

fn r_fn_param( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.eat( TokenKind::Id );
    p.eat( T![ : ] );
    r_type( p );

    m.complete( p, SyntaxKind::FnParam )
}

fn r_opt_fn_return_type( p: &mut Parser ) {
    if p.at( T![ -> ] ) {
        let m = p.start();

        p.eat( T![ -> ] );
        r_type( p );

        m.complete( p, SyntaxKind::FnReturnType );
    }
}


fn r_let( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( T![ let ] ) );
    r_name( p );
    //  p.eat( TokenKind::Id );
    if p.at( T![ : ] ) {
        p.eat( T![ : ] );
        r_type( p );
    }
    p.eat( T![ = ] );
    r_expr( p );

    m.complete( p, SyntaxKind::Let )
}

fn r_if( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( T![ if ] ) );
    r_inline_expr( p );
    r_block( p );

    m.complete( p, SyntaxKind::If )
}

fn r_while( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( T![ while ] ) );
    r_inline_expr( p );
    r_block( p );

    m.complete( p, SyntaxKind::While )
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_inline_expr( p: &mut Parser ) -> CompletedMarker {
    p.set_skipper( Skipper::Inline );

    let m = r_inline_binary( p, 0 );

    p.restore_skipper();

    m
}

fn infix_binding_power( kind: TokenKind ) -> Option< ( i8, i8 ) > {
    match kind {
        T![ || ] => Some( ( 1, 2 ) ),
        T![ && ] => Some( ( 3, 4 ) ),
        T![ == ] | T![ != ] => Some( ( 5, 6 ) ),
        T![ < ] | T![ <= ] | T![ > ] | T![ >= ] => Some( ( 7, 8 ) ),
        T![ + ] | T![ - ] => Some( ( 9, 10 ) ),
        T![ * ] | T![ / ] | T![ % ] => Some( ( 11, 12 ) ),
        _ => None,
    }
}

fn r_inline_binary( p: &mut Parser, min_bp: i8 ) -> CompletedMarker {
    let mut left = r_inline_unary( p );

    while let Some ( ( left_bp, right_bp ) ) = infix_binding_power( p.current() ) {
        if left_bp < min_bp {
            break;

        } else {
            let m = left.precede( p );

            p.eat_any();
            r_inline_binary( p, right_bp );
            left = m.complete( p, SyntaxKind::InlineBinary );
        }

    }

    left
}

fn r_inline_unary( p: &mut Parser ) -> CompletedMarker {
    match p.current() {
        T![ + ] | T![ - ] |  T![ ! ] => {
            let m = p.start();
            p.eat_any();
            r_inline_unary( p );
            m.complete( p, SyntaxKind::InlineUnary )
        }
        _ => r_inline_primary( p )
    }
}

fn r_inline_primary( p: &mut Parser ) -> CompletedMarker {
    let mut expr = match p.current() {
        T![ '(' ] => r_inline_subexpr( p ),
        TokenKind::Number => r_inline_number( p ),
        TokenKind::Id => r_inline_var( p ),
        T![ '"' ] => r_string( p ),
        _ => unreachable!(),
    };

    loop {
        if p.at( T![ . ] ) {
            let m = expr.precede( p );
            p.eat( T![ . ] );
            p.eat( TokenKind::Id );

            if p.at( T![ '(' ] ) {
                r_inline_args( p );

                expr = m.complete( p, SyntaxKind::InlineMethodCall );

            } else {
                expr = m.complete( p, SyntaxKind::InlineField );
            }

        } else if p.at( T![ '(' ] ) {
            let m = expr.precede( p );

            r_inline_args( p );

            m.complete( p, SyntaxKind::InlineCall );

        } else {
            break expr;
        }
    }
}

fn r_inline_subexpr( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( T![ '(' ] ) );
    r_inline_expr( p );
    p.expect( T![ ')' ] );

    m.complete( p, SyntaxKind::InlineSubexpr )
}

fn r_inline_number( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.eat( TokenKind::Number );

    m.complete( p, SyntaxKind::InlineNumber )
}

fn r_inline_var( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.eat( TokenKind::Id );

    m.complete( p, SyntaxKind::InlineVar )
}

fn r_inline_args( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( T![ '(' ] ) );
    while !p.at( T![ ')' ] ) && !p.at_eol() {
        r_inline_arg( p );
        p.eat( T![ , ] );
    }
    p.expect( T![ ')' ] );

    m.complete( p, SyntaxKind::CallArgs )
}

fn r_inline_arg( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    r_inline_expr( p );

    m.complete( p, SyntaxKind::CallArg )
}

fn r_string( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.set_skipper( Skipper::None );

    assert!( p.eat( T![ '"' ] ) );

    while !p.at_eol() && !p.at( T![ '"' ] ){
        match p.current() {
            TokenKind::StringFragment => {
                r_string_fragment( p );
            }

            TokenKind::DollarOpenBrace => {
                r_string_expr( p );
            }

            _ => {
                break;
            },
        }
    }

    p.expect( T![ '"' ] );

    p.restore_skipper();

    m.complete( p, SyntaxKind::String )
}

fn r_string_fragment( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.eat( TokenKind::StringFragment );

    m.complete( p, SyntaxKind::StringFragment )
}

fn r_string_expr( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.set_skipper( Skipper::Inline );

    assert!( p.eat( TokenKind::DollarOpenBrace ) );
    r_inline_expr( p );
    p.eat( T![ '}' ] );

    p.restore_skipper();

    m.complete( p, SyntaxKind::StringExpr )
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_block( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.set_skipper( Skipper::Block );

    p.expect( T![ '{' ] );
    while !p.at_eof() && !p.at( T![ '}' ] ) {
        r_block_statement( p );
    }
    p.eat( T![ '}' ] );

    p.restore_skipper();

    m.complete( p, SyntaxKind::Block )
}

fn r_expr( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    r_inline_expr( p );

    m.complete( p, SyntaxKind::Expr )
}

fn r_block_statement( p: &mut Parser ) -> CompletedMarker {
    match p.current() {
        T![ if ] => r_if( p ),
        T![ while ] => r_while( p ),
        T![ let ] => r_let( p ),
        _ => r_expr( p ),
    }
}
