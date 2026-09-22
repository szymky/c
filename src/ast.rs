#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Float,
    Char,
    Void,

    Struct(String),
    Pointer(Box<Type>),

    Array {
        element_type: Box<Type>,
        size: Option<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    AddrOf,
    Deref,
    Neg,
    LogicalNot,
    BitNot,
    PreInc,
    PreDec,
    PostInc,
    PostDec,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    Variable(String),
    ArrayInit(Vec<Expr>),
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    FunctionCall {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    MemberAccess {
        target: Box<Expr>,
        field: String,
    },
    PointerMemberAccess {
        target: Box<Expr>,
        field: String,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Assignment {
        op: AssignmentOp,
        target: Box<Expr>,
        value: Box<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expr(Expr),

    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    For {
        init: Option<Box<Stmt>>,
        condition: Option<Expr>,
        post: Option<Expr>,
        body: Box<Stmt>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    VarDecl {
        ty: Type,
        name: String,
        initializer: Option<Expr>,
    },
    Return(Option<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Declaration {
    Function(FunctionDecl),
    GlobalVar {
        ty: Type,
        name: String,
        initializer: Option<Expr>,
    },

    StructDef {
        name: Option<String>,
        fields: Vec<StructField>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDecl {
    pub return_type: Type,
    pub name: String,
    pub params: Vec<Param>,
    pub body: Option<Vec<Stmt>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub ty: Type,
    pub name: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    pub ty: Type,
    pub name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignmentOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
}
