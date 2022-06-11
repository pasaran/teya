use crate::parser::{ Parser, CompletedMarker, Skipper };
use crate::token_set::TokenSet;
use crate::{ SyntaxKind, TokenKind, ParserErrorKind, T };

//  ---------------------------------------------------------------------------------------------------------------  //

pub fn r_source_file( p: &mut Parser ) -> CompletedMarker {
    p.set_skipper( Skipper::Block );
    let m = p.start();

    while !p.is_eof() {
        r_statement( p );
    }

    p.expect( TokenKind::EOF );

    m.complete( p, SyntaxKind::SourceFile )
}

fn r_statement( p: &mut Parser ) -> CompletedMarker {
    match p.kind() {
        Some( T![ fn ] ) => r_fn( p ),
        Some( T![ type ] ) => r_type_alias( p ),
        Some( T![ struct ] ) => r_struct( p ),
        Some( T![ enum ] ) => r_enum( p ),
        _ => unreachable!( "{:?}", p.kind() )
    }
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_generic_params( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( T![ < ] ) );
    loop {
        match p.kind() {
            Some( TokenKind::Id ) => r_generic_param( p ),
            _ => break,
        };
        p.eat( T![ , ] );
    }
    p.expect( T![ > ] );

    m.complete( p, SyntaxKind::GenericParams )
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

fn r_type_alias( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( T![ type ] ) );

    r_name( p );
    p.expect( T![ = ] );
    r_type( p );

    m.complete( p, SyntaxKind::TypeAlias)
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_type( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    match p.kind() {
        Some( TokenKind::Id ) => r_typeref( p ),
        Some( T![ '[' ] ) => r_type_array( p ),
        Some( T![ '(' ] ) => r_type_tuple( p ),
        _ => unreachable!(),
    };

    m.complete( p, SyntaxKind::Type )
}

fn r_typeref( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.eat( TokenKind::Id );
    if p.is_kind( T![ < ] ) {
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

fn r_struct( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( T![ struct ] ) );
    r_struct_items( p );
    if p.is_kind( T![ < ] ) {
        r_generic_params( p );
    }

    m.complete( p, SyntaxKind::Struct )
}

fn r_struct_items( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.expect( T![ '{'] );
    loop {
        match p.kind() {
            Some( TokenKind::Id ) => r_struct_field( p ),
            Some( TokenKind::Fn ) => r_fn( p ),
            _ => break,
        };
        p.eat( T![ , ] );
    }
    p.expect( T![ '}'] );

    m.complete( p, SyntaxKind::StructItems )
}

fn r_struct_field( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( TokenKind::Id ) );
    p.expect( T![ : ] );
    r_type( p );
    if p.is_kind( T![ = ] ) {
        p.eat( T![ = ] );
        r_expr( p );
    }

    m.complete( p, SyntaxKind::StructField )
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn r_enum( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( T![ enum ] ) );
    r_name( p );
    if p.is_kind( T![ < ] ) {
        r_generic_params( p );
    }
    r_enum_variants( p );

    m.complete( p, SyntaxKind::Enum )
}

fn r_enum_variants( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.expect( T![ '{' ] );
    loop {
        match p.kind() {
            Some( TokenKind::Id ) => {
                let m = p.start();

                r_name( p );
                match p.nth_kind( 1 ) {
                    Some( T![ '(' ] ) => { r_tuple_fields( p ); },
                    Some( T![ '{' ] ) => { r_record_fields( p ); },
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
        match p.kind() {
            Some( TokenKind::Id ) => r_tuple_field( p ),
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
        match p.kind() {
            Some( TokenKind::Id ) => r_record_field( p ),
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

fn r_fn( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( T![ fn ] ) );
    r_name( p );
    r_fn_params( p );
    if p.is_kind( T![ -> ] ) {
        r_fn_return_type( p );
    }
    r_block( p );

    m.complete( p, SyntaxKind::Fn )
}

fn r_name( p: &mut Parser ) {
    if p.is_kind( TokenKind::Id ) {
        p.eat( TokenKind::Id );

    } else {
        p.error( ParserErrorKind::TokenRequired( TokenKind::Id ) );
        p.eat_any();
    }
}

fn r_fn_params( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.expect( T![ '(' ] );
    while !p.is_eol() && !p.is_kind( T![ ')' ] ) {
        r_fn_param( p );
        p.eat( T![ , ] );
    }
    p.expect( T![ ')' ] );

    m.complete( p, SyntaxKind::FnParams )
}

fn r_fn_param( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.eat( TokenKind::Id );
    p.eat( T![ : ] );
    r_type( p );

    m.complete( p, SyntaxKind::FnParam )
}

fn r_fn_return_type( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    p.eat( T![ -> ] );
    r_type( p );

    m.complete( p, SyntaxKind::FnReturnType )
}


fn r_let( p: &mut Parser ) -> CompletedMarker {
    let m = p.start();

    assert!( p.eat( T![ let ] ) );
    p.eat( TokenKind::Id );
    if p.is_kind( T![ : ] ) {
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

fn r_inline_binary( p: &mut Parser, min_bp: i8 ) -> CompletedMarker {
    let mut left = r_inline_unary( p );

    while let Some ( ( left_bp, right_bp ) ) = infix_binding_power( p.kind() ) {
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
    match p.kind() {
        Some( T![ + ] | T![ - ] |  T![ ! ] ) => {
            let m = p.start();
            p.eat_any();
            r_inline_unary( p );
            m.complete( p, SyntaxKind::InlineUnary )
        }
        _ => r_inline_primary( p )
    }
}

fn r_inline_primary( p: &mut Parser ) -> CompletedMarker {
    let mut expr = match p.kind() {
        Some( T![ '(' ] ) => r_inline_subexpr( p ),
        Some( TokenKind::Number ) => r_inline_number( p ),
        Some( TokenKind::Id ) => r_inline_var( p ),
        Some( T![ '"' ] ) => r_string( p ),
        _ => unreachable!(),
    };

    loop {
        if p.is_kind( T![ . ] ) {
            let m = expr.precede( p );
            p.eat( T![ . ] );
            p.eat( TokenKind::Id );

            if p.is_kind( T![ '(' ] ) {
                r_inline_args( p );

                expr = m.complete( p, SyntaxKind::InlineMethodCall );

            } else {
                expr = m.complete( p, SyntaxKind::InlineField );
            }

        } else if p.is_kind( T![ '(' ] ) {
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
    while !p.is_kind( T![ ')' ] ) && !p.is_eol() {
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

    while !p.is_eol() && !p.is_kind( T![ '"' ] ){
        match p.kind() {
            Some( TokenKind::StringFragment ) => {
                r_string_fragment( p );
            }

            Some( TokenKind::DollarOpenBrace ) => {
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
    while !p.is_eof() && !p.is_kind( T![ '}' ] ) {
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
    match p.kind() {
        Some( T![ if ] ) => r_if( p ),
        Some( T![ while ] ) => r_while( p ),
        Some( T![ let ] ) => r_let( p ),
        _ => r_expr( p ),
    }
}
