use logos::Logos;

#[derive(Debug, Logos, PartialEq, Clone)]
#[logos(skip r"[ \t\f\v]+")]
pub enum Token {
    // Types
    #[token("void")]
    Void,
    #[token("char")]
    CharKeyword,
    #[token("short")]
    Short,
    #[token("int")]
    Int,
    #[token("long")]
    Long,
    #[token("float")]
    FloatKeyword,
    #[token("double")]
    Double,
    #[token("signed")]
    Signed,
    #[token("unsigned")]
    Unsigned,
    #[token("struct")]
    Struct,
    #[token("union")]
    Union,
    #[token("enum")]
    Enum,
    #[token("typedef")]
    Typedef,

    // Storage class specifiers
    #[token("auto")]
    Auto,
    #[token("register")]
    Register,
    #[token("static")]
    Static,
    #[token("extern")]
    Extern,
    #[token("const")]
    Const,
    #[token("volatile")]
    Volatile,

    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("switch")]
    Switch,
    #[token("case")]
    Case,
    #[token("default")]
    Default,
    #[token("while")]
    While,
    #[token("do")]
    Do,
    #[token("for")]
    For,
    #[token("goto")]
    Goto,
    #[token("continue")]
    Continue,
    #[token("break")]
    Break,
    #[token("return")]
    Return,

    // buildin operators
    #[token("sizeof")]
    Sizeof,

    // Operators
    #[token("=")]
    Assign,
    #[token("+=")]
    PlusAssign,
    #[token("-=")]
    MinusAssign,
    #[token("*=")]
    StarAssign,
    #[token("/=")]
    SlashAssign,
    #[token("%=")]
    PercentAssign,
    #[token("<<=")]
    LeftShiftAssign,
    #[token(">>=")]
    RightShiftAssign,
    #[token("&=")]
    BitAndAssign,
    #[token("^=")]
    BitXorAssign,
    #[token("|=")]
    BitOrAssign,

    // Arithmetic
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("++")]
    Increment,
    #[token("--")]
    Decrement,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    LessThan,
    #[token(">")]
    GreaterThan,
    #[token("&&")]
    LogicalAnd,
    #[token("||")]
    LogicalOr,
    #[token("!")]
    LogicalNot,

    #[token("<=")]
    LessEqual,

    #[token(">=")]
    GreaterEqual,

    // bitwise
    #[token("&")]
    BitAnd,
    #[token("|")]
    BitOr,
    #[token("^")]
    BitXor,
    #[token("~")]
    BitNot,
    #[token("<<")]
    LeftShift,
    #[token(">>")]
    RightShift,

    #[token(".")]
    Dot,
    #[token("->")]
    Arrow,
    #[token("?")]
    Question,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,

    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,

    #[regex(r"#[^\n]*", allow_greedy = true)]
    Preprocessor,

    #[regex(r"\r?\n")]
    Newline,

    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLiteral,

    #[regex(r"'([^'\\]|\\.)*'")]
    CharLiteral,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r"[0-9]+")]
    Integer,

    // Floating-point literals (e.g., 3.14, .5, 1e-10f)
    #[regex(r"([0-9]+\.[0-9]*|\.[0-9]+)([eE][+-]?[0-9]+)?[fFL]?")]
    FloatLiteral,

    #[regex(r"//[^\n]*", allow_greedy = true)]
    SingleLineComment,

    #[regex(r"/\*([^*]|\*+[^/*])*\*+/")]
    BlockComment,
}
