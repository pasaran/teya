mod token_kind;
mod token_set;
mod lexer;
mod parser;
mod parser_error;
mod syntax_kind;
mod syntax_node;
mod grammar;
mod parser_event;
// mod test;

use parser_event::process;
pub use syntax_kind::{ SyntaxKind };
pub use token_kind::{ TokenKind };
pub use lexer::{ Lexer, Token };
pub use syntax_node::{ SyntaxNode, SyntaxElement };
use parser::Parser;
pub use parser_error::{ ParserError, ParserErrorKind };
use grammar::r_source_file;
pub use parser_event::{ ParserEvent };

use std::fs;

//  ---------------------------------------------------------------------------------------------------------------  //

fn main() {
    let content = fs::read_to_string( "./tests/01.teya" ).unwrap();
    let mut parser = Parser::new( &content );

    parser.parse( r_source_file );
    let events = parser.finish();
    // println!( "{:?}", events );

    let node = process( events );
    println!( "{:?}", node );
}
