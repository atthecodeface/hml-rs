//a Result
//tp Result
/// The [Result] type is a result with an error type of [crate::Error]
pub type Result<T> = std::result::Result<T, Error>;

//tp Error
/// Error for all HML
#[derive(Debug)]
pub enum Error {
    /// An I/O error occured in the underlying 'Read' or 'Write'
    Io(std::io::Error),
    /// A message
    Msg(String),
}

//ip Error
impl Error {
    //fp empty_name
    /// An error message indicating an empty name was provided, which
    /// is illegal
    pub fn empty_name<T>() -> Result<T> {
        Err(Self::Msg("empty name".to_string()))
    }

    //fp unmapped_prefix
    /// Create an error from the use of an unmapped prefix / namespace
    pub fn unmapped_prefix<T>(p: &str) -> Result<T> {
        Err(Self::Msg(format!("unmapped_prefix {}", p)))
    }

    //fp bad_name
    /// Create an error indicating a bad name (such as a:b:c)
    pub fn bad_name<T>(s: &str) -> Result<T> {
        Err(Self::Msg(format!("bad_name {}", s)))
    }
}

//ip From<std::io::Error> for Error
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

//ip Display for Error
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "HML: IO error: {}", e),
            Error::Msg(s) => write!(f, "HML: {}", s),
        }
    }
}

//ip std::error::Error for Error
impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(_) => "I/O error",
            Error::Msg(_) => "msg error",
        }
    }
}
