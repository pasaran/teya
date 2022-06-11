use std::mem;

use crate::{ SyntaxKind, Token, ParserError, SyntaxNode, SyntaxElement };

#[ derive( Debug ) ]
pub enum ParserEvent< 'a > {

    Start{
        kind: SyntaxKind,
        forward_parent: Option< usize >,
    },

    Finish,

    Token{
        token: Token< 'a >,
    },

    Error{
        error: ParserError,
    },

}

impl < 'a > ParserEvent< 'a > {

    pub fn empty() -> Self {
        ParserEvent::Start {
            kind: SyntaxKind::None,
            forward_parent: None,
        }
    }
}

pub fn process( mut events: Vec< ParserEvent > ) -> SyntaxNode {
    let mut forward_parents = Vec::new();

    let mut root = SyntaxNode::new( SyntaxKind::Root );
    let mut nodes: Vec< SyntaxNode > = vec![];

    for i in 0 .. events.len() {
        match mem::replace( &mut events[ i ], ParserEvent::empty() ) {
            ParserEvent::Start { kind, forward_parent } => {
                // For events[A, B, C], B is A's forward_parent, C is B's forward_parent,
                // in the normal control flow, the parent-child relation: `A -> B -> C`,
                // while with the magic forward_parent, it writes: `C <- B <- A`.

                // append `A` into parents.
                forward_parents.push( kind );
                let mut index = i;
                let mut fp = forward_parent;
                while let Some( fwd ) = fp {
                    index += fwd;
                    // append `A`'s forward_parent `B`
                    fp = match mem::replace( &mut events[ index ], ParserEvent::empty() ) {
                        ParserEvent::Start { kind, forward_parent } => {
                            forward_parents.push( kind );

                            forward_parent
                        }
                        _ => unreachable!(),
                    };
                    // append `B`'s forward_parent `C` in the next stage.
                }

                for kind in forward_parents.drain( .. ).rev() {
                    if kind != SyntaxKind::None {
                        nodes.push( root );
                        root = SyntaxNode::new( kind );
                    }
                }
            }

            ParserEvent::Finish => {
                let old_root = root;
                root = nodes.pop().unwrap();
                root.push( SyntaxElement::Node( old_root ) );
            }

            ParserEvent::Token { token } => {
                root.push( SyntaxElement::Token( token ) );
            }

            ParserEvent::Error { error } => (),//res.error( kind ),
        }
    }

    root
}
