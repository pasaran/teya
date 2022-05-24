#[derive(Debug, Clone, Copy)]
pub enum SyntaxKind {
    None,

    SourceFile,
    Struct,
    Fn,
    Block,
    If,
    For,
    While,
    Expr,

    InlineExpr,
    InlineBinary,
    InlineUnary,

    InlineSubexpr,
    InlineNumber,
    InlineVar,

    InlineArgs,
    InlineArg,

    InlineCall,
    InlineField,
    InlineMethodCall,

    InlineString,
    InlineStringFragment,
    InlineStringExpr,
}
