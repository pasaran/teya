#[derive(Copy,Clone,Debug,PartialEq)]
pub enum TokenKind {
    None,

    Space,
    Bang,
    Quote,
    Pound,
    Dollar,
    Percent,
    Amp,
    Apos,
    OpenParen,
    CloseParen,
    Star,
    Plus,
    Comma,
    Minus,
    Dot,
    Slash,
    Colon,
    Semicolon,
    Lt,
    Eq,
    Gt,
    Question,
    At,
    OpenBracket,
    Backslash,
    CloseBracket,
    Caret,
    //  Underscore,
    Backtick,
    OpenBrace,
    Pipe,
    CloseBrace,
    Tilde,

    BangEq,
    PercentEq,
    AmpAmp,
    AmpAmpEq,
    StarEq,
    PlusEq,
    MinusEq,
    MinusGt,
    DotDot,
    DotDotDot,
    SlashEq,
    LtEq,
    EqEq,
    GtEq,
    PipePipe,
    PipePipeEq,

    EOL,
    EOF,
    Indent,
    Dedent,
    Comment,
    Id,
    Number,
    StringFragment,
    DollarOpenBrace,

    Type,
    Struct,
    Enum,
    Let,
    Const,
    Fn,
    If,
    For,
    While,

    Unknown,
}


#[macro_export]
macro_rules !T {
    [ ] => { TokenKind::Space };
    [!] => { TokenKind::Bang };
    ['"'] => { TokenKind::Quote };
    [#] => { TokenKind::Pound };
    [$] => { TokenKind::Dollar };
    [%] => { TokenKind::Percent };
    [&] => { TokenKind::Amp };
    ['\''] => { TokenKind::Apos };
    ['('] => { TokenKind::OpenParen };
    [')'] => { TokenKind::CloseParen };
    [*] => { TokenKind::Star };
    [+] => { TokenKind::Plus };
    [,] => { TokenKind::Comma };
    [-] => { TokenKind::Minus };
    [.] => { TokenKind::Dot };
    [/] => { TokenKind::Slash };
    [:] => { TokenKind::Colon };
    [;] => { TokenKind::Semicolon };
    [<] => { TokenKind::Lt };
    [=] => { TokenKind::Eq };
    [>] => { TokenKind::Gt };
    [?] => { TokenKind::Question };
    [@] => { TokenKind::At };
    ['['] => { TokenKind::OpenBracket };
    ['\\'] => { TokenKind::Backslash };
    [']'] => { TokenKind::CloseBracket };
    [^] => { TokenKind::Caret };
    ['`'] => { TokenKind::Backtick };
    ['{'] => { TokenKind::OpenBrace };
    [|] => { TokenKind::Pipe };
    ['}'] => { TokenKind::CloseBrace };
    [~] => { TokenKind::Tilde };

    [!=] => { TokenKind::BangEq };
    [%=] => { TokenKind::PercentEq };
    [&&] => { TokenKind::AmpAmp };
    [&&=] => { TokenKind::AmpAmpEq };
    [*=] => { TokenKind::StarEq };
    [+=] => { TokenKind::PlusEq };
    [-=] => { TokenKind::MinusEq };
    [->] => { TokenKind::MinusGt };
    [..] => { TokenKind::DotDot };
    [...] => { TokenKind::DotDotDot };
    [/=] => { TokenKind::SlashEq };
    [<=] => { TokenKind::LtEq };
    [==] => { TokenKind::EqEq };
    [>=] => { TokenKind::GtEq };
    [||] => { TokenKind::PipePipe };
    [||=] => { TokenKind::PipePipeEq };

    ["${"] => { TokenKind::DollarOpenBrace };

    [type] => { TokenKind::Type };
    [struct] => { TokenKind::Struct };
    [enum] => { TokenKind::Enum };
    [const] => { TokenKind::Const };
    [let] => { TokenKind::Let };
    [if] => { TokenKind::If };
    [for] => { TokenKind::For };
    [while] => { TokenKind::While };
    [fn] => { TokenKind::Fn };

}
pub use T;
