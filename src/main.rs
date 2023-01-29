mod token_kind;
mod token_set;
mod lexer;
mod parser;
mod parser_error;
mod syntax_kind;
mod syntax_node;
mod grammar;
mod parser_event;
mod ast_node;
mod ast;
//  mod test;
//  mod types;

pub use syntax_kind::{ SyntaxKind };
pub use token_kind::{ TokenKind };
pub use lexer::{ Lexer, Token };
pub use syntax_node::{ SyntaxNode, SyntaxElement };
use parser::Parser;
pub use parser_error::{ ParserError, ParserErrorKind };
use grammar::r_source_file;
pub use parser_event::{ ParserEvent };
pub use ast_node::AstNode;
pub use token_set::TokenSet;

use std::fs;

// . use crate::ast;

//  ---------------------------------------------------------------------------------------------------------------  //

fn main() {
    let content = fs::read_to_string( "./tests/01.teya" ).unwrap();
    let parser = Parser::new( &content );

    let node = parser.parse( r_source_file );

    let root = ast::Root::cast( &node ).unwrap();

    root.walk( | x: &ast::Fn | {
        // println!( "{:?}", x.name.as_ref().unwrap().id().unwrap() );
    } );

    // let source_file = root.source_file.unwrap();
    // let item0 = source_file.items.get( 0 ).unwrap();
    // if let ast::Item::Fn( fn0 ) = item0 {
    //     let name = fn0.name.as_ref().unwrap();
    //     let id = name.id().unwrap();

        // println!( "{:?}", node );
    //     println!( "{}", id.text );
    // }

    // let fn0 = ast::Fn::cast( item0 ).unwrap();
    // let name = fn0.name().unwrap();
    // let id = name.id().unwrap();

    // println!( "{:?}", node );
    // println!( "{}", id.text );
}
