#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyntaxKind {
    Root,
    None,

    SourceFile,

    TypeAlias,

    Type,
    TypeArray,
    TypeTuple,
    TypeRef,

    GenericParams,
    GenericParam,
    GenericArgs,
    GenericArg,

    Struct,
    StructItems,
    StructField,
    StructMethod,

    Enum,
    EnumVariants,
    EnumVariant,

    TupleFields,
    TupleField,

    RecordFields,
    RecordField,

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

    FnParams,
    FnParam,
    FnReturnType,

    CallArgs,
    CallArg,

    InlineCall,
    InlineField,
    InlineMethodCall,

    String,
    StringFragment,
    StringExpr,
}
