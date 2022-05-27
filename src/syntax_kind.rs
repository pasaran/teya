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
    Let,

    InlineExpr,
    InlineBinary,
    InlineUnary,

    InlineSubexpr,
    InlineNumber,
    InlineVar,

    FnArgs,
    FnArg,
    FnReturnType,
    TypeRef,

    CallArgs,
    CallArg,

    InlineCall,
    InlineField,
    InlineMethodCall,

    InlineString,
    InlineStringFragment,
    InlineStringExpr,
}
