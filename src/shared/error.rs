use nom::error::ParseError;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Nom(nom::error::ErrorKind),
    NomIncomplete(nom::Needed), // TODO: Remove?
    NomStacked(Vec<Error>),
    Utf8Error,
    Fable(ErrorKind),
}

#[derive(Debug)]
pub enum ErrorKind {
    InvalidInstruction,
    InvalidTagName,
    InvalidScriptProperty,
    InvalidScriptValue,
    InvalidScriptBinaryOp,
    NotEnoughSpaceForParser,
    CannotInferFileType,
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IO(error)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_: std::str::Utf8Error) -> Self {
        Error::Utf8Error
    }
}

impl From<nom::Err<(&[u8], nom::error::ErrorKind)>> for Error {
    fn from(error: nom::Err<(&[u8], nom::error::ErrorKind)>) -> Self {
        match error {
            nom::Err::Incomplete(needed) => Error::NomIncomplete(needed), // TODO: Remove?
            nom::Err::Error((_rest, error)) => Error::Nom(error),
            nom::Err::Failure((_rest, error)) => Error::Nom(error)
        }
    }
}

impl From<nom::Err<Error>> for Error {
    fn from(error: nom::Err<Error>) -> Self {
        match error {
            nom::Err::Incomplete(needed) => Error::NomIncomplete(needed), // TODO: Remove?
            nom::Err::Error(error) => error,
            nom::Err::Failure(error) => error,
        }
    }
}

impl<I> ParseError<I> for Error {
    fn from_error_kind(_: I, kind: nom::error::ErrorKind) -> Self {
        Error::Nom(kind)
    }

    fn append(_: I, kind: nom::error::ErrorKind, prev: Self) -> Self {
        match prev {
            Error::NomStacked(mut errors) => {
                errors.push(Error::Nom(kind));
                Error::NomStacked(errors)
            },
            other => {
                let mut errors = Vec::new();
                errors.push(other);
                errors.push(Error::Nom(kind));
                Error::NomStacked(errors)
            }
        }
    }
}