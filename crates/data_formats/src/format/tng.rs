use crate::{util::text::WithSpan, Location, Span};

#[derive(Clone, Debug)]
pub struct Tng {
    // raw_tng: RawTng<'a>,
}

#[derive(Clone, Debug)]
pub struct TngParseError {
    location: Option<Location>,
    kind: TngParseErrorKind,
}

#[derive(Clone, Debug)]
pub enum TngParseErrorKind {
    MissingSemicolon,
    ExpectedEmptyValue,
    ParseIntError,
    NoValue,
    ExpectedString,
    ExpectedIdent,
    ParseFloatError,
    ExpectedBool,
    ExpectedPath,
    ExpectedPathItem,
}

impl Tng {
    pub fn parse(source: &str) -> Result<Self, TngParseError> {
        let raw_tng = RawTng::parse(source)?;
        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
struct RawTng<'a> {
    fields: Vec<RawTngField<'a>>,
}

impl<'a> RawTng<'a> {
    fn parse(source: &'a str) -> Result<RawTng, TngParseError> {
        let mut fields = Vec::new();

        for (line_count, mut line) in source.lines().enumerate() {
            let mut line_span = Span::new(
                Location::new(line_count, 0),
                Location::new(line_count, line.len().saturating_sub(1)),
            );

            skip_spaces(&mut line);

            line_span.start.column = line.len().saturating_sub(line_span.end.column);

            // Split the key and value from the semicolon separator
            let (field, _) = match line.split_once(";") {
                Some(x) => x,
                None => {
                    if line.is_empty() || line.chars().all(|c| c.is_whitespace()) {
                        continue;
                    } else {
                        return Err(TngParseError {
                            location: Some(Location::new(line_count, 0)),
                            kind: TngParseErrorKind::MissingSemicolon,
                        });
                    }
                }
            };

            // Split the key and accessor from the value
            let (key_accessor, mut value) = match field.split_once(" ") {
                Some((key, value)) => (key, value),
                None => (field, ""),
            };

            skip_spaces(&mut value);

            let value_with_span = if value.is_empty() {
                None
            } else {
                let mut span = line_span.clone();
                span.start.column = key_accessor.len() + 1;
                span.end.column = line.len() - 1;
                Some(WithSpan::new(value, span))
            };

            // Split the key from the accessor
            let property_index = key_accessor.find(".").unwrap_or(key_accessor.len());
            let array_index = key_accessor.find("[").unwrap_or(key_accessor.len());
            let call_index = key_accessor.find("(").unwrap_or(key_accessor.len());
            let first_accessor_index = property_index.min(array_index).min(call_index);

            let key = &key_accessor[..first_accessor_index];

            let key_with_span = {
                let mut span = line_span.clone();
                span.end.column = key.len();
                WithSpan::new(key, span)
            };

            let accessor = &key_accessor[first_accessor_index..];

            let accessor_with_span = if accessor.is_empty() {
                None
            } else {
                let mut span = line_span.clone();
                span.start.column = key.len();
                span.end.column = key.len() + accessor.len();
                Some(WithSpan::new(accessor, span))
            };

            fields.push(RawTngField {
                key: key_with_span,
                path: accessor_with_span,
                value: value_with_span,
            });
        }

        Ok(RawTng { fields })
    }
}

#[derive(Clone, Debug)]
struct RawTngField<'a> {
    key: WithSpan<&'a str>,
    path: Option<WithSpan<&'a str>>,
    value: Option<WithSpan<&'a str>>,
}

impl<'a> RawTngField<'a> {
    fn key(&self) -> &str {
        self.key.inner
    }

    fn path(&self) -> Result<Path<'a>, TngParseError> {
        match self.path {
            Some(path) => Ok(Path::new(path)),
            None => Err(TngParseError {
                location: Some(self.key.span.end),
                kind: TngParseErrorKind::ExpectedPath,
            }),
        }
    }

    fn empty_value(&self) -> Result<(), TngParseError> {
        match &self.value {
            None => Ok(()),
            Some(value) => Err(TngParseError {
                location: Some(value.span.start),
                kind: TngParseErrorKind::ExpectedEmptyValue,
            }),
        }
    }

    fn integer_value(&self) -> Result<i32, TngParseError> {
        match &self.value {
            Some(value) => match value.inner.parse::<i32>() {
                Ok(value) => Ok(value),
                Err(_) => Err(TngParseError {
                    location: Some(value.span.start),
                    kind: TngParseErrorKind::ParseIntError,
                }),
            },
            None => Err(TngParseError {
                location: Some(self.key.span.end),
                kind: TngParseErrorKind::NoValue,
            }),
        }
    }

    fn uid_value(&self) -> Result<u64, TngParseError> {
        match &self.value {
            Some(value) => match value.inner.parse::<u64>() {
                Ok(value) => Ok(value),
                Err(_) => Err(TngParseError {
                    location: Some(value.span.start),
                    kind: TngParseErrorKind::ParseIntError,
                }),
            },
            None => Err(TngParseError {
                location: Some(self.key.span.end),
                kind: TngParseErrorKind::NoValue,
            }),
        }
    }

    fn float_value(&self) -> Result<f32, TngParseError> {
        match &self.value {
            Some(value) => match value.inner.parse::<f32>() {
                Ok(value) => Ok(value),
                Err(_) => Err(TngParseError {
                    location: Some(value.span.start),
                    kind: TngParseErrorKind::ParseFloatError,
                }),
            },
            None => Err(TngParseError {
                location: Some(self.key.span.end),
                kind: TngParseErrorKind::NoValue,
            }),
        }
    }

    fn bool_value(&self) -> Result<bool, TngParseError> {
        match &self.value {
            Some(value) => {
                let bool_string = value.inner;
                match bool_string {
                    "TRUE" | "true" => Ok(true),
                    "FALSE" | "false" => Ok(false),
                    _ => Err(TngParseError {
                        location: Some(value.span.start),
                        kind: TngParseErrorKind::ExpectedBool,
                    }),
                }
            }
            None => Err(TngParseError {
                location: Some(self.key.span.end),
                kind: TngParseErrorKind::NoValue,
            }),
        }
    }

    fn string_value(&self) -> Result<&str, TngParseError> {
        match &self.value {
            Some(value) => {
                let string = value.inner;
                let mut chars = string.chars();
                if chars.next() == Some('\"') && chars.last() == Some('\"') {
                    Ok(&string[1..string.len() - 1])
                } else {
                    Err(TngParseError {
                        location: Some(value.span.start),
                        kind: TngParseErrorKind::ExpectedString,
                    })
                }
            }
            None => Err(TngParseError {
                location: Some(self.key.span.end),
                kind: TngParseErrorKind::NoValue,
            }),
        }
    }

    fn ident_value(&self) -> Result<&str, TngParseError> {
        match &self.value {
            Some(value) => {
                let ident_string = value.inner;
                let chars = ident_string.chars();
                if chars.enumerate().all(is_ident_char) {
                    Ok(ident_string)
                } else {
                    Err(TngParseError {
                        location: Some(value.span.start),
                        kind: TngParseErrorKind::ExpectedIdent,
                    })
                }
            }
            None => Err(TngParseError {
                location: Some(self.key.span.end),
                kind: TngParseErrorKind::NoValue,
            }),
        }
    }
}

struct Path<'a> {
    inner: WithSpan<&'a str>,
}

impl<'a> Path<'a> {
    fn new(inner: WithSpan<&'a str>) -> Self {
        Self { inner }
    }

    fn next(&mut self) -> Result<Option<PathItem<'a>>, TngParseError> {
        let path_string = self.inner.inner;

        let property_index = path_string.find(".").unwrap_or(path_string.len());
        let array_index = path_string.find("[").unwrap_or(path_string.len());
        let call_index = path_string.find("(").unwrap_or(path_string.len());
        let first_item_index = property_index.min(array_index).min(call_index);

        if property_index == first_item_index {
            let path_chars = path_string.chars();

            let property_end = match path_chars.enumerate().position(|x| !is_ident_char(x)) {
                Some(x) => x,
                None => path_string.len(),
            };

            let property = &path_string[..property_end];

            self.inner.inner = &path_string[property_end..];
            self.inner.span.start.column += property.len();

            Ok(Some(PathItem::Property(property)))
        } else if array_index == first_item_index {
        } else if call_index == first_item_index {
        } else if path_string.is_empty() {
            Ok(None)
        } else {
            Err(TngParseError {
                location: Some(self.inner.span.start),
                kind: TngParseErrorKind::ExpectedPathItem,
            })
        }
    }
}

fn is_ident_char((i, c): (usize, char)) -> bool {
    if i == 0 {
        c.is_ascii_alphabetic() || c == '_'
    } else {
        c.is_ascii_alphanumeric() || c == '_'
    }
}

#[derive(Clone, Debug)]
enum PathItem<'a> {
    Index(i32),
    Property(&'a str),
    Call,
}

fn skip_spaces(source: &mut &str) {
    let space_ends = source
        .chars()
        .position(|c| !c.is_whitespace() && c != '\n' && c != '\r')
        .unwrap_or(0);

    let (_spaces, rest) = source.split_at(space_ends);

    *source = rest;
}
