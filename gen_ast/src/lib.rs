extern crate proc_macro;

use proc_macro::{ TokenStream };
use quote::{ quote, format_ident };
// use syn::{ parse_macro_input, AttributeArgs, Lit, Meta, NestedMeta, LitInt, Token, token, bracketed };
use syn::parse::{ Parse, ParseStream, Result };
use syn::punctuated::Punctuated;
use syn::{ parse_macro_input, token, Token, bracketed, braced, Ident, LitInt };

//  ---------------------------------------------------------------------------------------------------------------  //

struct Ast {
    name: Ident,
    fields: Punctuated< AstField, Token![,] >,
}

impl Parse for Ast {

    fn parse( input: ParseStream ) -> Result< Self > {
        let name: Ident = input.parse()?;

        let group;
        let _: token::Brace = braced!( group in input );
        let fields: Punctuated< AstField, Token![,] > = group.parse_terminated( AstField::parse )?;

        Ok( Ast {
            name,
            fields,
        } )
    }
}

struct AstField {
    name: Ident,
    type_: AstFieldType,
}

impl Parse for AstField {

    fn parse( input: ParseStream ) -> Result< Self > {
        let name: Ident = input.parse()?;
        let _: Token![ : ] = input.parse()?;
        let type_: AstFieldType = input.parse()?;

        Ok( AstField {
            name,
            type_,
        } )
    }

}

enum AstFieldType {
    Token( Ident ),
    Nodes( Ident ),
    Node( Ident, LitInt ),
}

impl Parse for AstFieldType {

    fn parse( input: ParseStream ) -> Result< Self > {
        if input.peek( Token![ # ] ) {
            let _ = input.parse::< Token![ # ] >();
            let ident: Ident = input.parse()?;

            return Ok( AstFieldType::Token( ident ) );
        }

        let ident: Ident = input.parse()?;
        if input.peek( token::Bracket ) {
            let content;
            let _: token::Bracket = bracketed!( content in input );
            let index: LitInt = content.parse()?;

            return Ok( AstFieldType::Node( ident, index ) );

        } else {
            return Ok( AstFieldType::Nodes( ident ) );
        }
    }

}

//  ---------------------------------------------------------------------------------------------------------------  //

#[proc_macro]
pub fn ast( body: TokenStream ) -> TokenStream {
    println!( "{:?}", body );

    let ast = parse_macro_input!( body as Ast );

    let fields = ast.fields
        .iter()
        .map( | field | {
            let field_name = &field.name;

            match &field.type_ {
                AstFieldType::Token( id ) => quote! {
                    pub fn #field_name( &'a self ) -> Option< &'a Token< 'a > > {
                        self.node.find_token( TokenKind::#id )
                    }
                },
                AstFieldType::Node( id, index ) => quote! {
                    pub fn #field_name( &'a self ) -> Option< #id< 'a > >{
                        self.node.find_node( SyntaxKind::#id, #index )
                            .map( | node | #id { node } )
                    }
                },
                AstFieldType::Nodes( id ) => quote! {
                    pub fn #field_name( &'a self ) -> Vec< #id< 'a > > {
                        self.node.find_nodes( SyntaxKind::#id )
                            .iter()
                            .map( | node | #id { node } )
                            .collect()
                    }

                }
            }
        } );

    let name = ast.name;
    let r = quote! {
        pub struct #name < 'a > {
            node: &'a SyntaxNode< 'a >,
        }

        impl < 'a >#name < 'a > {
            #(#fields)*
        }

    };

    r.into()
}

// #[proc_macro]
// pub fn ast( body: TokenStream ) -> TokenStream {
//     println!( "{:?}", body );

    // println!( "{:?}", args );
    // let args = parse_macro_input!( args as AttributeArgs );
    // let input: syn::DeriveInput = syn::parse( input ).unwrap();

    // let ( impl_generics, ty_generics, where_clause ) = input.generics.split_for_impl();
    // let name = &input.ident;

    /*
    let tokens = args.iter().filter_map(
        | arg | {
            if let Some( ( name, value ) ) = arg_to_pair( arg ) {
                let token_name = format_ident!( "is_{}", name );

                let len = value.len();

                let exprs = value
                    .bytes()
                    .enumerate()
                    .map( | ( i, b ) | quote! { self.get_byte( pos + #i ) == #b } );

                return Some( quote! {
                    pub fn #token_name( &self, pos: usize ) -> usize {
                        if pos + #len <= self.len {
                            let bytes = self.bytes;
                            if #(#exprs)&&* {
                                return #len;
                            }
                        }

                        return 0;
                    }
                } );
            }

            return None;
        }
    );
    */

    //  #(#tokens)*
//     let r = quote! {
//     };

//     r.into()
// }

// fn arg_to_pair( arg: &NestedMeta ) -> Option< ( String, String ) > {
//     if let NestedMeta::Meta( Meta::NameValue( arg ) ) = arg {
//         if let Some( ident ) = arg.path.get_ident() {
//             if let Lit::Str( lit ) = &arg.lit {
//                 return Some( ( ident.to_string(), lit.value() ) );
//             }
//         }
//     }

//     return None;
// }

// fn arg_to_lit( arg: &NestedMeta ) -> Option< String > {
//     if let NestedMeta::Lit( Lit::Str( lit ) ) = arg {
//         return Some( lit.value() );
//     }

//     return None;
// }

