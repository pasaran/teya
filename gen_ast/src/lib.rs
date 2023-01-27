extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{ Parse, ParseStream, Result };
use syn::punctuated::Punctuated;
use syn::{ parse_macro_input, token, Token, bracketed, braced, Ident, LitInt, parenthesized };

//  ---------------------------------------------------------------------------------------------------------------  //

enum Ast {
    Union( AstUnion ),
    Record( AstRecord ),
}

//  ---------------------------------------------------------------------------------------------------------------  //

struct AstUnion {
    name: Ident,
    items: Punctuated< Ident, Token![ | ] >,
}

//  ---------------------------------------------------------------------------------------------------------------  //

struct AstRecord {
    name: Ident,
    fields: Punctuated< AstRecordField, Token![ , ] >,
}

//  ---------------------------------------------------------------------------------------------------------------  //

impl Parse for Ast {

    fn parse( input: ParseStream ) -> Result< Self > {
        let name: Ident = input.parse()?;

        if input.peek( Token![ = ] ) {
            let _ = input.parse::< Token![ = ] >();

            let items: Punctuated< Ident, Token![ | ] > = input.parse_terminated( Ident::parse )?;

            Ok( Ast::Union( AstUnion {
                name,
                items,
            } ) )

        } else {
            let group;
            let _: token::Brace = braced!( group in input );
            let fields: Punctuated< AstRecordField, Token![ , ] > = group.parse_terminated( AstRecordField::parse )?;

            Ok( Ast::Record( AstRecord {
                name,
                fields,
            } ) )
        }
    }
}

//  ---------------------------------------------------------------------------------------------------------------  //

struct AstRecordField {
    name: Ident,
    type_: AstRecordFieldType,
}

impl Parse for AstRecordField {

    fn parse( input: ParseStream ) -> Result< Self > {
        let name: Ident = input.parse()?;
        let _: Token![ : ] = input.parse()?;
        let type_: AstRecordFieldType = input.parse()?;

        Ok( AstRecordField {
            name,
            type_,
        } )
    }

}

enum AstRecordFieldType {
    //  #Foo
    Token( Ident ),
    //  #( Foo | Bar )
    Tokens( Punctuated< Ident, Token![ | ] > ),
    //  *Foo
    Nodes( Ident ),
    //  Foo
    Node( Ident ),
    //  Foo[ 0 ]
    NodeIndex( Ident, LitInt ),
}

impl Parse for AstRecordFieldType {

    fn parse( input: ParseStream ) -> Result< Self > {
        if input.peek( Token![ # ] ) {
            let _ = input.parse::< Token![ # ] >();
            if input.peek( token::Paren ) {
                let content;
                let _: token::Paren = parenthesized!( content in input );
                let tokens: Punctuated< Ident, Token![ | ] > = content.parse_terminated( Ident::parse )?;

                return Ok( AstRecordFieldType::Tokens( tokens ) );

            } else {
                let name: Ident = input.parse()?;

                return Ok( AstRecordFieldType::Token( name ) );
            }
        }

        if input.peek( Token![ * ] ) {
            let _ = input.parse::< Token![ * ] >();
            let ident: Ident = input.parse()?;

            return Ok( AstRecordFieldType::Nodes( ident ) );
        }

        let ident: Ident = input.parse()?;
        if input.peek( token::Bracket ) {
            let content;
            let _: token::Bracket = bracketed!( content in input );
            let index: LitInt = content.parse()?;

            return Ok( AstRecordFieldType::NodeIndex( ident, index ) );

        } else {
            return Ok( AstRecordFieldType::Node( ident ) );
        }
    }

}

//  ---------------------------------------------------------------------------------------------------------------  //

#[proc_macro]
pub fn ast( body: TokenStream ) -> TokenStream {
    // println!( "{:?}", body );

    let ast = parse_macro_input!( body as Ast );

    match ast {
        Ast::Union( union_ ) => ast_union( union_ ),
        Ast::Record( record ) => ast_record( record ),
    }
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn ast_union( ast: AstUnion ) -> TokenStream {
    let name = ast.name;

    let enum_items = ast.items
        .iter()
        .map( | item | quote! {
            #item ( #item< 'a > )
        } );

    let can_cast_items = ast.items
        .iter()
        .map( | item | quote! {
            SyntaxKind::#item => true,
        } );

    let cast_items = ast.items
        .iter()
        .map( | item | quote! {
            SyntaxKind::#item => #name::#item( #item { node } ),
        } );

    let walk_items = ast.items
        .iter()
        .map( | item | quote! {
            #name::#item( x ) => x.walk( callback ),
        } );

    let syntax_items = ast.items
        .iter()
        .map( | item | quote! {
            #name::#item( x ) => &x.syntax(),
        } );

    let r = quote! {
        pub enum #name < 'a > {
            #( #enum_items ),*
        }

        impl < 'a > AstNode< 'a > for #name< 'a > {

            fn syntax( &self ) -> &SyntaxNode< 'a > {
                match self {
                    #( #syntax_items )*
                }
            }

            fn can_cast( kind: SyntaxKind ) -> bool {
                match kind {
                    #( #can_cast_items )*
                    _ => false,
                }
            }

            fn cast( node: &'a SyntaxNode< 'a > ) -> Option< Self > {
                let r = match node.kind {
                    #( #cast_items )*
                    _ => return None,
                };

                Some( r )
            }

            fn walk< T >( &'a self, callback: fn( &'a T ) ) where T: AstNode< 'a > {
                match self {
                    #( #walk_items )*
                }
            }

        }
    };

    r.into()
}

//  ---------------------------------------------------------------------------------------------------------------  //

fn ast_record( ast: AstRecord ) -> TokenStream {
    let fields = ast.fields
        .iter()
        .map( | field | {
            let field_name = &field.name;

            match &field.type_ {

                AstRecordFieldType::Tokens( tokens ) => {
                    let tokens = tokens.iter();

                    quote! {
                        pub fn #field_name( &'a self ) -> Option< &'a Token< 'a > > {
                            let ts = TokenSet::new( &[ #( TokenKind::#tokens ),* ] );
                                self.node.find_token_in_set( ts )
                        }
                    }
                },

                AstRecordFieldType::Token( token ) => {
                    quote! {
                        pub fn #field_name( &'a self ) -> Option< &'a Token< 'a > > {
                            self.node.find_token( TokenKind::#token )
                        }
                    }
                },

                AstRecordFieldType::Node( id ) => quote! {
                    pub fn #field_name( &'a self ) -> Option< #id< 'a > > {
                        self.node.find_node_by_index( SyntaxKind::#id, 0 )
                            .map( | node | #id { node } )
                    }
                },

                AstRecordFieldType::NodeIndex( id, index ) => quote! {
                    pub fn #field_name( &'a self ) -> Option< #id< 'a > > {
                        self.node.find_node_by_index( SyntaxKind::#id, #index )
                            .map( | node | #id { node } )
                    }
                },

                AstRecordFieldType::Nodes( id ) => quote! {
                    pub fn #field_name( &'a self ) -> Vec< #id< 'a > > {
                        self.node.children
                            .iter()
                            .filter_map( | e | {
                                match e {
                                    SyntaxElement::Node( node ) => #id::cast( node ),
                                    _ => None,
                                }
                            } )
                            .collect()

                        // self.node.find_nodes( SyntaxKind::#id )
                        //     .iter()
                        //     .map( | node | #id { node } )
                        //     .collect()
                    }
                },

            }
        } );

    let walk_items = ast.fields
        .iter()
        .map( | field | {
            let field_name = &field.name;

            match &field.type_ {

                AstRecordFieldType::Node( _ ) |
                AstRecordFieldType::NodeIndex( _, _ ) => quote! {
                    if let Some( t ) = self.#field_name() {
                        let node = t.syntax();
                        if let Some( ast ) = T::cast( node ) {
                            ast.walk( callback );
                            callback( &ast );
                        }
                    }
                },

                AstRecordFieldType::Nodes( _ ) => quote! {
                    self.#field_name()
                        .iter()
                        .for_each( | x | {
                            let node = x.syntax();
                            if let Some( ast ) = T::cast( node ) {
                                ast.walk( callback );
                                callback( &ast );
                            }
                        } );
                },

                _ => quote! {},

            }
        } );

    let name = ast.name;
    let r = quote! {
        pub struct #name < 'a > {
            node: &'a SyntaxNode< 'a >,
        }

        impl < 'a > #name< 'a > {
            #( #fields )*
        }

        impl < 'a > AstNode< 'a > for #name< 'a > {

            fn syntax( &self ) -> &SyntaxNode< 'a > {
                &self.node
            }

            fn can_cast( kind: SyntaxKind ) -> bool {
                SyntaxKind::#name == kind
            }

            fn cast( node: &'a SyntaxNode< 'a > ) -> Option< Self > {
                if Self::can_cast( node.kind ) {
                    Some( Self { node } )
                } else {
                    None
                }
            }

            fn walk< T >( &'a self, callback: fn( &'a T ) ) where T: AstNode< 'a > {
                #( #walk_items )*
            }

        }
    };

    r.into()
}
