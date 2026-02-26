use crate::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    UnknownBlock,
    UnknownInline,
    MissingRequiredArg,
    UnknownArg,
    InvalidType,
    InvalidStaticEnumValue,
    InvalidDynamicEnumValue,
    BlockNotClosed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: Level,
    pub code: ErrorCode,
    pub message: String,
    pub span: Span,
    pub suggestion: Option<String>,
}
