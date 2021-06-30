/// Error for all HML
#[derive(Debug)]
pub enum HmlError {
    /// An I/O error occured in the underlying 'Read' or 'Write'
    Io(std::io::Error),
    Msg(String),
}

impl HmlError {
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

impl From<std::io::Error> for HmlError {
    fn from(err: std::io::Error) -> HmlError {
        HmlError::Io(err)
    }
}

impl std::fmt::Display for HmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HmlError::Io(e)  => write!(f, "HML: IO error: {}", e),
            HmlError::Msg(s) => write!(f, "HML: {}", s),
        }
    }
}

impl std::error::Error for HmlError {
    fn description(&self) -> &str {
        match *self {
            HmlError::Io(_) => "I/O error",
            HmlError::Msg(_) => "msg error",
        }
    }
}

