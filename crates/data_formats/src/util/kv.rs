use arrayvec::ArrayVec;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Kv<'a> {
    pub fields: Vec<KvField<'a>>,
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
#[error("{field_error} on line {line_num}")]
pub struct KvError {
    line_num: usize,
    field_error: KvFieldError,
}

impl<'a> Kv<'a> {
    pub fn parse(source: &'a str) -> Result<Kv, KvError> {
        let mut fields = Vec::new();

        for (mut line_num, mut line) in source.lines().enumerate() {
            line_num += 1;

            skip_spaces(&mut line);

            let field = KvField::new(line, line_num).map_err(|field_error| KvError {
                line_num,
                field_error,
            })?;

            let field = match field {
                Some(field) => field,
                None => continue,
            };

            fields.push(field);
        }

        Ok(Kv { fields })
    }
}

#[derive(Clone, Debug)]
pub struct KvField<'a> {
    pub key: KvKey<'a>,
    pub value: KvValue<'a>,
    pub line_num: usize,
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum KvFieldError {
    #[error("missing semicolon")]
    MissingSemicolon,

    #[error(transparent)]
    Key(#[from] KvKeyError),

    #[error(transparent)]
    Value(#[from] KvValueError),
}

impl<'a> KvField<'a> {
    fn new(mut line: &'a str, line_num: usize) -> Result<Option<Self>, KvFieldError> {
        skip_spaces(&mut line);

        // Split the key and value from the semicolon separator
        match line.split_once(";") {
            Some((field, _)) => {
                // Split the key and accessor from the value
                let (key, mut value) = match field.split_once(" ") {
                    Some((key, value)) => (key, value),
                    None => (field, ""),
                };

                let key = KvKey::new(key)?;

                skip_spaces(&mut value);

                let value = KvValue::new(value);

                Ok(Some(Self {
                    key,
                    value,
                    line_num,
                }))
            }
            None => {
                if line.is_empty() || line.chars().all(|c| c.is_whitespace()) {
                    Ok(None)
                } else {
                    Err(KvFieldError::MissingSemicolon)
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.with_no_value().is_ok()
    }

    pub fn with_key(&self, identifier: &str) -> Result<&Self, UnexpectedField> {
        if self.key.identifier == identifier {
            Ok(self)
        } else {
            Err(UnexpectedField {
                line_num: self.line_num,
            })
        }
    }

    pub fn with_no_path(&self) -> Result<&Self, InvalidPath> {
        if self.key.path.iter().next().is_none() {
            Ok(self)
        } else {
            Err(InvalidPath {
                line_num: self.line_num,
            })
        }
    }

    pub fn with_path(&self) -> Result<(&Self, ArrayVec<KvPathItem, MAX_PATH_ITEMS>), InvalidPath> {
        let mut path_iter = self.key.path.iter();

        let path = path_iter
            .by_ref()
            .take(MAX_PATH_ITEMS)
            .collect::<Result<_, KvPathError>>()
            .map_err(|_| InvalidPath {
                line_num: self.line_num,
            })?;

        if path_iter.next().is_some() {
            Err(InvalidPath {
                line_num: self.line_num,
            })?
        }

        Ok((self, path))
    }

    pub fn with_no_value(&self) -> Result<&Self, InvalidValue> {
        match self.value.empty() {
            Ok(()) => Ok(self),
            Err(KvValueError(expected)) => Err(InvalidValue {
                expected,
                line_num: self.line_num,
            }),
        }
    }

    pub fn with_integer_value(&self) -> Result<(&Self, i32), InvalidValue> {
        match self.value.integer() {
            Ok(integer) => Ok((self, integer)),
            Err(KvValueError(expected)) => Err(InvalidValue {
                expected,
                line_num: self.line_num,
            }),
        }
    }

    pub fn with_uid_value(&self) -> Result<(&Self, u64), InvalidValue> {
        match self.value.uid() {
            Ok(uid) => Ok((self, uid)),
            Err(KvValueError(expected)) => Err(InvalidValue {
                expected,
                line_num: self.line_num,
            }),
        }
    }

    pub fn with_float_value(&self) -> Result<(&Self, f32), InvalidValue> {
        match self.value.float() {
            Ok(float) => Ok((self, float)),
            Err(KvValueError(expected)) => Err(InvalidValue {
                expected,
                line_num: self.line_num,
            }),
        }
    }

    pub fn bool(&self) -> Result<(&Self, bool), InvalidValue> {
        match self.value.bool() {
            Ok(bool) => Ok((self, bool)),
            Err(KvValueError(expected)) => Err(InvalidValue {
                expected,
                line_num: self.line_num,
            }),
        }
    }

    pub fn with_string_value(&self) -> Result<(&Self, &str), InvalidValue> {
        match self.value.string() {
            Ok(string) => Ok((self, string)),
            Err(KvValueError(expected)) => Err(InvalidValue {
                expected,
                line_num: self.line_num,
            }),
        }
    }

    pub fn with_identifier_value(&self) -> Result<(&Self, &str), InvalidValue> {
        match self.value.identifier() {
            Ok(identifier) => Ok((self, identifier)),
            Err(KvValueError(expected)) => Err(InvalidValue {
                expected,
                line_num: self.line_num,
            }),
        }
    }

    pub fn with_c2dcoordf_value(&self) -> Result<(&Self, [f32; 2]), InvalidValue> {
        match self.value.c2dcoordf() {
            Ok(identifier) => Ok((self, identifier)),
            Err(KvValueError(expected)) => Err(InvalidValue {
                expected,
                line_num: self.line_num,
            }),
        }
    }

    pub fn with_c3dcoordf_value(&self) -> Result<(&Self, [f32; 3]), InvalidValue> {
        match self.value.c3dcoordf() {
            Ok(identifier) => Ok((self, identifier)),
            Err(KvValueError(expected)) => Err(InvalidValue {
                expected,
                line_num: self.line_num,
            }),
        }
    }

    pub fn with_crgbcolour_value(&self) -> Result<(&Self, [u8; 4]), InvalidValue> {
        match self.value.crgbcolour() {
            Ok(identifier) => Ok((self, identifier)),
            Err(KvValueError(expected)) => Err(InvalidValue {
                expected,
                line_num: self.line_num,
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct KvValue<'a> {
    source: &'a str,
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
#[error("expected {0} value but failed to parse one")]
pub struct KvValueError(KvValueKind);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KvValueKind {
    Empty,
    Integer,
    Uid,
    Float,
    Bool,
    String,
    Identifier,
    C2DCoordF,
    C3DCoordF,
    CRGBColour,
}

impl std::fmt::Display for KvValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Empty => "empty",
            Self::Integer => "integer",
            Self::Uid => "UID",
            Self::Float => "float",
            Self::Bool => "bool",
            Self::String => "string",
            Self::Identifier => "identifier",
            Self::C2DCoordF => "C2DCoordF",
            Self::C3DCoordF => "C3DCoordF",
            Self::CRGBColour => "CRGBColour",
        })
    }
}

impl<'a> KvValue<'a> {
    fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn empty(&self) -> Result<(), KvValueError> {
        if self.source.is_empty() {
            Ok(())
        } else {
            Err(KvValueError(KvValueKind::Empty))
        }
    }

    pub fn integer(&self) -> Result<i32, KvValueError> {
        self.source
            .parse::<i32>()
            .map_err(|_| KvValueError(KvValueKind::Integer))
    }

    pub fn uid(&self) -> Result<u64, KvValueError> {
        self.source
            .parse::<u64>()
            .map_err(|_| KvValueError(KvValueKind::Uid))
    }

    pub fn float(&self) -> Result<f32, KvValueError> {
        self.source
            .parse::<f32>()
            .map_err(|_| KvValueError(KvValueKind::Float))
    }

    pub fn bool(&self) -> Result<bool, KvValueError> {
        match self.source {
            "TRUE" => Ok(true),
            "FALSE" => Ok(false),
            _ => Err(KvValueError(KvValueKind::Bool)),
        }
    }

    pub fn string(&self) -> Result<&str, KvValueError> {
        let mut chars = self.source.chars();

        if chars.next() == Some('\"') && chars.last() == Some('\"') {
            Ok(&self.source[1..self.source.len() - 1])
        } else {
            Err(KvValueError(KvValueKind::String))
        }
    }

    pub fn identifier(&self) -> Result<&str, KvValueError> {
        let chars = self.source.chars();

        if chars.enumerate().all(|(i, c)| {
            if i == 0 {
                c.is_alphabetic() || c == '_'
            } else {
                c.is_alphanumeric() || c == '_' || c == ' '
            }
        }) {
            Ok(self.source)
        } else {
            Err(KvValueError(KvValueKind::Identifier))
        }
    }

    pub fn c2dcoordf(&self) -> Result<[f32; 2], KvValueError> {
        use KvValueKind::C2DCoordF;

        let (_, mut rest) = self
            .source
            .split_once("C2DCoordF(")
            .ok_or_else(|| KvValueError(C2DCoordF))?;

        skip_spaces(&mut rest);

        let (x, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError(C2DCoordF))?;

        skip_spaces(&mut rest);

        let (y, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError(C2DCoordF))?;

        skip_spaces(&mut rest);

        if rest.is_empty() {
            let y = y.parse::<f32>().map_err(|_| KvValueError(C2DCoordF))?;
            let x = x.parse::<f32>().map_err(|_| KvValueError(C2DCoordF))?;
            Ok([x, y])
        } else {
            Err(KvValueError(C2DCoordF))
        }
    }

    pub fn c3dcoordf(&self) -> Result<[f32; 3], KvValueError> {
        use KvValueKind::C3DCoordF;

        let (_, mut rest) = self
            .source
            .split_once("C3DCoordF(")
            .ok_or_else(|| KvValueError(C3DCoordF))?;

        skip_spaces(&mut rest);

        let (x, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError(C3DCoordF))?;

        skip_spaces(&mut rest);

        let (y, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError(C3DCoordF))?;

        skip_spaces(&mut rest);

        let (z, mut rest) = rest
            .split_once(")")
            .ok_or_else(|| KvValueError(C3DCoordF))?;

        skip_spaces(&mut rest);

        if rest.is_empty() {
            let y = y.parse::<f32>().map_err(|_| KvValueError(C3DCoordF))?;
            let x = x.parse::<f32>().map_err(|_| KvValueError(C3DCoordF))?;
            let z = z.parse::<f32>().map_err(|_| KvValueError(C3DCoordF))?;
            Ok([x, y, z])
        } else {
            Err(KvValueError(C3DCoordF))
        }
    }

    pub fn crgbcolour(&self) -> Result<[u8; 4], KvValueError> {
        use KvValueKind::CRGBColour;

        let (_, mut rest) = self
            .source
            .split_once("CRGBColour(")
            .ok_or_else(|| KvValueError(CRGBColour))?;

        skip_spaces(&mut rest);

        let (r, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError(CRGBColour))?;

        skip_spaces(&mut rest);

        let (g, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError(CRGBColour))?;

        skip_spaces(&mut rest);

        let (b, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError(CRGBColour))?;

        skip_spaces(&mut rest);

        let (a, mut rest) = rest
            .split_once(")")
            .ok_or_else(|| KvValueError(CRGBColour))?;

        skip_spaces(&mut rest);

        if rest.is_empty() {
            let r = r.parse::<u8>().map_err(|_| KvValueError(CRGBColour))?;
            let g = g.parse::<u8>().map_err(|_| KvValueError(CRGBColour))?;
            let b = b.parse::<u8>().map_err(|_| KvValueError(CRGBColour))?;
            let a = a.parse::<u8>().map_err(|_| KvValueError(CRGBColour))?;
            Ok([r, g, b, a])
        } else {
            Err(KvValueError(CRGBColour))
        }
    }
}

#[derive(Clone, Debug)]
pub struct KvKey<'a> {
    pub identifier: &'a str,
    pub path: KvPath<'a>,
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum KvKeyError {
    #[error("empty key")]
    EmptyKey,
    #[error("key {0}")]
    Path(#[from] KvPathError),
}

impl<'a> KvKey<'a> {
    fn new(source: &'a str) -> Result<Self, KvKeyError> {
        if source.is_empty() {
            return Err(KvKeyError::EmptyKey);
        }

        // Split the key from the accessor
        let prop_start = source.find(".").unwrap_or(source.len());
        let array_start = source.find("[").unwrap_or(source.len());
        let call_start = source.find("(").unwrap_or(source.len());
        let path_start = prop_start.min(array_start).min(call_start);

        let ident = &source[..path_start];

        let path = KvPath::new(&source[path_start..]);

        Ok(Self {
            identifier: ident,
            path,
        })
    }
}

#[derive(Clone, Debug)]
pub struct KvPath<'a> {
    source: &'a str,
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum KvPathError {
    #[error("unknown path item")]
    UnknownItem,
    #[error("invalid property")]
    InvalidProperty,
    #[error("invalid index")]
    InvalidIndex,
    #[error("invalid call")]
    InvalidCall,
}

impl<'a> KvPath<'a> {
    fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn iter(&self) -> KvPathIter<'a> {
        KvPathIter::new(self.source)
    }
}

pub struct KvPathIter<'a> {
    source: &'a str,
}

impl<'a> KvPathIter<'a> {
    fn new(source: &'a str) -> Self {
        Self { source }
    }

    fn next_property(&mut self) -> Option<Result<KvPathItem<'a>, KvPathError>> {
        let source = &self.source[1..];

        let prop_end = source
            .chars()
            .enumerate()
            .position(|(i, c)| {
                !(if i == 0 {
                    c.is_alphabetic() || c == '_'
                } else {
                    c.is_alphanumeric() || c == '_'
                })
            })
            .unwrap_or(source.len());

        let prop = &source[..prop_end];

        if prop.is_empty() {
            self.source = "";
            Some(Err(KvPathError::InvalidProperty))
        } else {
            self.source = &source[prop_end..];
            Some(Ok(KvPathItem::Property(prop)))
        }
    }

    fn next_index(&mut self) -> Option<Result<KvPathItem<'a>, KvPathError>> {
        let source = &self.source[1..];

        let (index_string, rest) = match source.split_once("]") {
            Some(x) => x,
            None => {
                self.source = "";
                return Some(Err(KvPathError::InvalidIndex));
            }
        };

        let index = match index_string.parse::<i32>() {
            Ok(x) => x,
            Err(_) => {
                self.source = "";
                return Some(Err(KvPathError::InvalidIndex));
            }
        };

        self.source = rest;

        Some(Ok(KvPathItem::Index(index)))
    }

    fn next_call(&mut self) -> Option<Result<KvPathItem<'a>, KvPathError>> {
        if self.source.get(..2) == Some("()") {
            self.source = &self.source[2..];
            Some(Ok(KvPathItem::Call))
        } else {
            self.source = "";
            Some(Err(KvPathError::InvalidCall))
        }
    }
}

impl<'a> Iterator for KvPathIter<'a> {
    type Item = Result<KvPathItem<'a>, KvPathError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source.chars().next() {
            None => None,
            Some('.') => self.next_property(),
            Some('[') => self.next_index(),
            Some('(') => self.next_call(),
            Some(_) => {
                self.source = "";
                Some(Err(KvPathError::UnknownItem))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum KvPathItem<'a> {
    Property(&'a str),
    Index(i32),
    Call,
}

// This is somewhat arbitrary. I haven't seen more than 4 path items. Doubling that to be safe.
pub const MAX_PATH_ITEMS: usize = 8;

fn skip_spaces(source: &mut &str) {
    let space_ends = source
        .chars()
        .position(|c| !c.is_whitespace() && c != '\n' && c != '\r')
        .unwrap_or(0);

    let (_spaces, rest) = source.split_at(space_ends);

    *source = rest;
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
#[error("field on line {line_num} was unexpected")]
pub struct UnexpectedField {
    pub line_num: usize,
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
#[error("field with invalid path on line {line_num}")]
pub struct InvalidPath {
    pub line_num: usize,
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
#[error("expected {expected} value on line {line_num}")]
pub struct InvalidValue {
    pub expected: KvValueKind,
    pub line_num: usize,
}
