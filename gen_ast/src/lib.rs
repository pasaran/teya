extern crate proc_macro;

use std::collections::{HashMap, HashSet};

use proc_macro::TokenStream;
use proc_macro2;
use quote::{quote, format_ident};
use syn::parse::{ Parse, ParseStream, Result };
use syn::punctuated::Punctuated;
use syn::{ parse_macro_input, token, Token, braced, Ident, parenthesized };

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

struct AstRecordField {
    name: Ident,
    type_: AstRecordFieldType,
}

//  ---------------------------------------------------------------------------------------------------------------  //

enum AstRecordFieldType {
    //  #Foo
    Token( Ident ),

    //  #( Foo | Bar )
    Tokens( Punctuated< Ident, Token![ | ] > ),

    //  *Foo
    Nodes( Ident ),

    //  Foo
    Node( Ident ),
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

//  ---------------------------------------------------------------------------------------------------------------  //

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
        return Ok( AstRecordFieldType::Node( ident ) );
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
    let name = &ast.name;

    let enum_def = get_enum_def( &ast );

    let syntax_method = get_syntax_method( &ast );
    let can_cast_method = get_can_cast_method( &ast );
    let cast_method = get_cast_method( &ast );
    let walk_method = get_walk_method( &ast );

    let r = quote! {
        #enum_def

        impl < 'a > AstNode< 'a > for #name< 'a > {

            fn kind() -> SyntaxKind {
                SyntaxKind::#name
            }

            #syntax_method
            #can_cast_method
            #cast_method
            #walk_method
        }
    };

    return r.into();

    fn get_enum_def( ast: &AstUnion ) -> proc_macro2::TokenStream {
        let name = &ast.name;

        let items = ast.items
            .iter()
            .map( | item | quote! {
                #item ( #item< 'a > ),
            } );

        quote! {
            pub enum #name < 'a > {
                #( #items )*
            }
        }
    }

    fn get_syntax_method( ast: &AstUnion ) -> proc_macro2::TokenStream {
        let name = &ast.name;

        let items = ast.items
            .iter()
            .map( | item | quote! {
                #name::#item( x ) => &x.syntax(),
            } );

        quote! {
            fn syntax( &'a self ) -> &SyntaxNode< 'a > {
                match self {
                    #( #items )*
                }
            }
        }
    }

    fn get_can_cast_method( ast: &AstUnion ) -> proc_macro2::TokenStream {
        let items = ast.items
            .iter()
            .map( | item | quote! {
                SyntaxKind::#item => true,
            } );

        quote!{
            fn can_cast( kind: SyntaxKind ) -> bool {
                match kind {
                    #( #items )*
                    _ => false,
                }
            }
        }
    }

    fn get_cast_method( ast: &AstUnion ) -> proc_macro2::TokenStream {
        let name = &ast.name;

        let items = ast.items
            .iter()
            .map( | item | quote! {
                if let Some( x ) = #item::cast( node ) {
                    return Some( #name::#item( x ) );
                }
            } );

        quote!{
            fn cast( node: &'a SyntaxNode< 'a > ) -> Option< Self > {
                #( #items )*

                return None;
            }
        }
    }

    fn get_walk_method( ast: &AstUnion ) -> proc_macro2::TokenStream {
        let name = &ast.name;

        let items = ast.items
            .iter()
            .map( | item | quote! {
                #name::#item( x ) => x.walk( callback ),
            } );

        quote!{
            fn walk< T >( &'a self, callback: fn( &'a T ) ) where T: AstNode< 'a > {
                match self {
                    #( #items )*
                }
                if T::kind() == Self::kind() {
                    let x = unsafe { std::mem::transmute::< &Self, &T >( self ) };
                    callback( x );
                }
            }
        }
    }

}

//  ---------------------------------------------------------------------------------------------------------------  //

fn ast_record( ast: AstRecord ) -> TokenStream {
    let name = &ast.name;

    let struct_def = get_struct_def( &ast );

    let cast_method = get_cast_method( &ast );
    let walk_method = get_walk_method( &ast );

    let r = quote! {
        #struct_def

        impl < 'a > AstNode< 'a > for #name< 'a > {

            fn kind() -> SyntaxKind {
                SyntaxKind::#name
            }

            fn syntax( &'a self ) -> &SyntaxNode< 'a > {
                &self.node
            }

            fn can_cast( kind: SyntaxKind ) -> bool {
                SyntaxKind::#name == kind
            }

            #cast_method
            #walk_method
        }
    };

    return r.into();

    fn get_struct_def( ast: &AstRecord ) -> proc_macro2::TokenStream {
        let struct_items = ast.fields
            .iter()
            .map( | field | {
                let field_name = &field.name;

                match &field.type_ {
                    AstRecordFieldType::Nodes( id ) => {
                        quote! {
                            pub #field_name: Vec< #id< 'a > >,
                        }
                    }
                    AstRecordFieldType::Node( id ) => {
                        quote! {
                            pub #field_name: Option< #id< 'a > >,
                        }
                    },
                    _ => quote! {},
                }
            } );

        let name = &ast.name;

        quote! {
            pub struct #name < 'a > {
                node: &'a SyntaxNode< 'a >,
                #( #struct_items )*
            }
        }
    }

    fn get_cast_method( ast: &AstRecord ) -> proc_macro2::TokenStream {
        let field_ids: HashSet< _ > = ast.fields
            .iter()
            .filter_map( | field | {
                match &field.type_ {
                    AstRecordFieldType::Nodes( id ) |
                    AstRecordFieldType::Node( id ) => {
                        Some( id )
                    },
                    _ => None,
                }
            } )
            .collect();

        let cast_vars = field_ids
            .iter()
            .map( | &field_id | {
                let var_name = get_var_name( field_id );

                quote! {
                    let mut #var_name = node.children
                    .iter()
                    .filter_map( | e | match e {
                        SyntaxElement::Node( node ) => #field_id::cast( node ),
                        _ => None,
                    } );
                }
            } );

        let mut field_indexes: HashMap< String, usize > = HashMap::new();

        let cast_record_items = ast.fields
            .iter()
            .filter_map( | field | {
                let field_name = &field.name;

                match &field.type_ {
                    AstRecordFieldType::Node( field_id ) => {
                        let var_name = get_var_name( field_id );

                        let index = *field_indexes.entry( field_id.to_string() )
                            .and_modify( | e | { *e += 1 } )
                            .or_insert( 0 );

                        Some( quote! {
                            #field_name: #var_name.nth( #index ),
                        } )
                    },

                    AstRecordFieldType::Nodes( field_id ) => {
                        let var_name = get_var_name( field_id );

                        Some( quote! {
                            #field_name: #var_name.collect(),
                        } )
                    },

                    _ => None,
                }
            } );

        quote! {
            fn cast( node: &'a SyntaxNode< 'a > ) -> Option< Self > {
                if Self::can_cast( node.kind ) {
                    #( #cast_vars )*

                    Some( Self {
                        node,
                        #( #cast_record_items )*
                    } )

                } else {
                    None
                }
            }
        }
    }

    fn get_walk_method( ast: &AstRecord ) -> proc_macro2::TokenStream {
        let walk_items = ast.fields
            .iter()
            .filter_map( | field | {
                let field_name = &field.name;

                match &field.type_ {

                    AstRecordFieldType::Node( _ ) => Some( quote! {
                        if let Some( x ) = self.#field_name.as_ref() {
                            x.walk( callback );
                        }
                    } ),

                    AstRecordFieldType::Nodes( _ ) => Some( quote! {
                        self.#field_name
                            .iter()
                            .for_each( | x | {
                                x.walk( callback );
                            } );
                    } ),

                    _ => None,

                }
            } );

        quote! {
            fn walk< T >( &'a self, callback: fn( &'a T ) ) where T: AstNode< 'a > {
                #( #walk_items )*

                if T::kind() == Self::kind() {
                    let x = unsafe { std::mem::transmute::< &Self, &T >( self ) };
                    callback( x );
                }
            }
        }
    }

    fn get_var_name( id: &Ident ) -> Ident {
        format_ident!( "{}", id.to_string().to_lowercase() )
    }

}

