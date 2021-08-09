/*a Copyright

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

@file    error.rs
@brief   Errors for markup
 */

//a Result
//tp Result
/// The [Result] type is a result with an error type of [crate::Error]
pub type Result<T> = std::result::Result<T, Error>;

/// Error for all HML
#[derive(Debug)]
pub enum Error {
    /// An I/O error occured in the underlying 'Read' or 'Write'
    Io(std::io::Error),
    /// A message
    Msg(String),
}

impl Error {
    //fp empty_name
    /// An error message indicating an empty name was provided, which
    /// is illegal
    pub fn empty_name<T>() -> Result<T> {
        Err(Self::Msg(format!("empty name")))
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
