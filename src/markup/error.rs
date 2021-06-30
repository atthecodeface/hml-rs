//a Result
//tp Result
/// The [Result] type is a result with an error type of [crate::Error]
pub type MarkupResult<T> = std::result::Result<T, MarkupError>;


/// Error for all HML
#[derive(Debug)]
pub enum MarkupError {
    /// An I/O error occured in the underlying 'Read' or 'Write'
    Io(std::io::Error),
    Msg(String),
}

impl MarkupError {
    pub fn empty_name<T>() -> Result<T,Self> {
        Err(Self::Msg(format!("empty name")))
    }
    pub fn unmapped_prefix<T>(p:&str) -> Result<T,Self> {
        Err(Self::Msg(format!("unmapped_prefix {}",p)))
    }
    pub fn bad_name<T>(s:&str) -> Result<T,Self> {
        Err(Self::Msg(format!("bad_name {}",s)))
    }
}

impl From<std::io::Error> for MarkupError {
    fn from(err: std::io::Error) -> MarkupError {
        MarkupError::Io(err)
    }
}

impl std::fmt::Display for MarkupError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MarkupError::Io(e)  => write!(f, "HML: IO error: {}", e),
            MarkupError::Msg(s) => write!(f, "HML: {}", s),
        }
    }
}

impl std::error::Error for MarkupError {
    fn description(&self) -> &str {
        match *self {
            MarkupError::Io(_) => "I/O error",
            MarkupError::Msg(_) => "msg error",
        }
    }
}

