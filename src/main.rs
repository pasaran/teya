mod token_kind;
mod token_set;
mod lexer;
mod parser;
mod parser_error;
mod syntax_kind;
mod syntax_node;
mod grammar;
mod parser_event;
mod ast;
// mod test;

pub use syntax_kind::{ SyntaxKind };
pub use token_kind::{ TokenKind };
pub use lexer::{ Lexer, Token };
pub use syntax_node::{ SyntaxNode, SyntaxElement };
use parser::Parser;
pub use parser_error::{ ParserError, ParserErrorKind };
use grammar::r_source_file;
pub use parser_event::{ ParserEvent };

use ast::Root;

use std::fs;

//  ---------------------------------------------------------------------------------------------------------------  //

fn main() {
    let content = fs::read_to_string( "./tests/01.teya" ).unwrap();
    let parser = Parser::new( &content );

    let node = parser.parse( r_source_file );

    let root = Root::new( &node );

    let source_file = root.SourceFile().unwrap();
    let fn0 = source_file.Fn( 0 ).unwrap();
    let name = fn0.Name().unwrap();
    let id = name.Id().unwrap();

    println!( "{:?}", node );
    println!( "{:?}", id );
}
