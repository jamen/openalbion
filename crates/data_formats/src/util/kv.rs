use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Kv<'a> {
    pub fields: Vec<KvField<'a>>,
}

#[derive(Copy, Clone, Debug, Error)]
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

#[derive(Copy, Clone, Debug, Error)]
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
}

#[derive(Clone, Debug)]
pub struct KvValue<'a> {
    source: &'a str,
}

#[derive(Copy, Clone, Debug, Error)]
pub enum KvValueError {
    #[error("non-empty value")]
    NonEmpty,
    #[error("non-integer value")]
    NonInteger,
    #[error("non-UID value")]
    NonUid,
    #[error("non-float value")]
    NonFloat,
    #[error("non-boolean value")]
    NonBool,
    #[error("non-string value")]
    NonString,
    #[error("non-identifier value")]
    NonIdent,
    #[error("non-C2DCoordF value")]
    NonC2DCoordF,
    #[error("non-C3DCoordF value")]
    NonC3DCoordF,
    #[error("non-CRGBColour value")]
    NonCRGBColour,
}

impl<'a> KvValue<'a> {
    fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn empty(&self) -> Result<(), KvValueError> {
        if self.source.is_empty() {
            Ok(())
        } else {
            Err(KvValueError::NonEmpty)
        }
    }

    pub fn integer(&self) -> Result<i32, KvValueError> {
        self.source
            .parse::<i32>()
            .map_err(|_| KvValueError::NonInteger)
    }

    pub fn uid(&self) -> Result<u64, KvValueError> {
        self.source.parse::<u64>().map_err(|_| KvValueError::NonUid)
    }

    pub fn float(&self) -> Result<f32, KvValueError> {
        self.source
            .parse::<f32>()
            .map_err(|_| KvValueError::NonFloat)
    }

    pub fn bool(&self) -> Result<bool, KvValueError> {
        match self.source {
            "TRUE" => Ok(true),
            "FALSE" => Ok(false),
            _ => Err(KvValueError::NonBool),
        }
    }

    pub fn string(&self) -> Result<&str, KvValueError> {
        let mut chars = self.source.chars();

        if chars.next() == Some('\"') && chars.last() == Some('\"') {
            Ok(&self.source[1..self.source.len() - 1])
        } else {
            Err(KvValueError::NonString)
        }
    }

    pub fn ident(&self) -> Result<&str, KvValueError> {
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
            Err(KvValueError::NonIdent)
        }
    }

    pub fn c2dcoordf(&self) -> Result<[f32; 2], KvValueError> {
        let (_, mut rest) = match self.source.split_once("C2DCoordF(") {
            Some(x) => x,
            None => return Err(KvValueError::NonC2DCoordF),
        };

        skip_spaces(&mut rest);

        let (x, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError::NonC2DCoordF)?;

        skip_spaces(&mut rest);

        let (y, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError::NonC2DCoordF)?;

        skip_spaces(&mut rest);

        if rest.is_empty() {
            let y = y.parse::<f32>().map_err(|_| KvValueError::NonC2DCoordF)?;
            let x = x.parse::<f32>().map_err(|_| KvValueError::NonC2DCoordF)?;
            Ok([x, y])
        } else {
            Err(KvValueError::NonC2DCoordF)
        }
    }

    pub fn c3dcoordf(&self) -> Result<[f32; 3], KvValueError> {
        let (_, mut rest) = match self.source.split_once("C3DCoordF(") {
            Some(x) => x,
            None => return Err(KvValueError::NonC3DCoordF),
        };

        skip_spaces(&mut rest);

        let (x, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError::NonC3DCoordF)?;

        skip_spaces(&mut rest);

        let (y, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError::NonC3DCoordF)?;

        skip_spaces(&mut rest);

        let (z, mut rest) = rest
            .split_once(")")
            .ok_or_else(|| KvValueError::NonC3DCoordF)?;

        skip_spaces(&mut rest);

        if rest.is_empty() {
            let y = y.parse::<f32>().map_err(|_| KvValueError::NonC3DCoordF)?;
            let x = x.parse::<f32>().map_err(|_| KvValueError::NonC3DCoordF)?;
            let z = z.parse::<f32>().map_err(|_| KvValueError::NonC3DCoordF)?;
            Ok([x, y, z])
        } else {
            Err(KvValueError::NonC3DCoordF)
        }
    }

    pub fn crgbcolour(&self) -> Result<[u8; 4], KvValueError> {
        let (_, mut rest) = match self.source.split_once("CRGBColour(") {
            Some(x) => x,
            None => return Err(KvValueError::NonCRGBColour),
        };

        skip_spaces(&mut rest);

        let (r, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError::NonCRGBColour)?;

        skip_spaces(&mut rest);

        let (g, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError::NonCRGBColour)?;

        skip_spaces(&mut rest);

        let (b, mut rest) = rest
            .split_once(",")
            .ok_or_else(|| KvValueError::NonCRGBColour)?;

        skip_spaces(&mut rest);

        let (a, mut rest) = rest
            .split_once(")")
            .ok_or_else(|| KvValueError::NonCRGBColour)?;

        skip_spaces(&mut rest);

        if rest.is_empty() {
            let r = r.parse::<u8>().map_err(|_| KvValueError::NonCRGBColour)?;
            let g = g.parse::<u8>().map_err(|_| KvValueError::NonCRGBColour)?;
            let b = b.parse::<u8>().map_err(|_| KvValueError::NonCRGBColour)?;
            let a = a.parse::<u8>().map_err(|_| KvValueError::NonCRGBColour)?;
            Ok([r, g, b, a])
        } else {
            Err(KvValueError::NonCRGBColour)
        }
    }
}

#[derive(Clone, Debug)]
pub struct KvKey<'a> {
    pub ident: &'a str,
    pub path: KvPath<'a>,
}

#[derive(Copy, Clone, Debug, Error)]
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

        Ok(Self { ident, path })
    }
}

#[derive(Clone, Debug)]
pub struct KvPath<'a> {
    source: &'a str,
}

#[derive(Copy, Clone, Debug, Error)]
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
        let chars = self.source.chars();

        let prop_end = chars
            .enumerate()
            .position(|(i, c)| {
                !(if i == 0 {
                    c.is_alphabetic() || c == '_'
                } else {
                    c.is_alphanumeric() || c == '_'
                })
            })
            .unwrap_or(self.source.len());

        let prop = &self.source[..prop_end];

        if prop.is_empty() {
            self.source = "";
            Some(Err(KvPathError::InvalidProperty))
        } else {
            self.source = &self.source[prop_end..];
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

fn skip_spaces(source: &mut &str) {
    let space_ends = source
        .chars()
        .position(|c| !c.is_whitespace() && c != '\n' && c != '\r')
        .unwrap_or(0);

    let (_spaces, rest) = source.split_at(space_ends);

    *source = rest;
}
