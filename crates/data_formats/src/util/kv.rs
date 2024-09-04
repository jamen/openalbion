use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Kv<'a> {
    fields: Vec<KvField<'a>>,
}

#[derive(Copy, Clone, Debug, Error)]
#[error("{field_error} error on line {line_num}")]
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
    key: KvKey<'a>,
    value: KvValue<'a>,
    line_num: usize,
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
                let (key, value) = match field.split_once(" ") {
                    Some((key, value)) => (key, value),
                    None => (field, ""),
                };

                let key = KvKey::new(key)?;

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
}

impl<'a> KvValue<'a> {
    fn new(mut source: &'a str) -> Self {
        skip_spaces(&mut source);
        Self { source }
    }

    fn empty(&self) -> Result<(), KvValueError> {
        if self.source.is_empty() {
            Ok(())
        } else {
            Err(KvValueError::NonEmpty)
        }
    }

    fn integer(&self) -> Result<i32, KvValueError> {
        self.source
            .parse::<i32>()
            .map_err(|_| KvValueError::NonInteger)
    }

    fn uid(&self) -> Result<u64, KvValueError> {
        self.source.parse::<u64>().map_err(|_| KvValueError::NonUid)
    }

    fn float(&self) -> Result<f32, KvValueError> {
        self.source
            .parse::<f32>()
            .map_err(|_| KvValueError::NonFloat)
    }

    fn bool(&self) -> Result<bool, KvValueError> {
        match self.source {
            "TRUE" => Ok(true),
            "FALSE" => Ok(false),
            _ => Err(KvValueError::NonBool),
        }
    }

    fn string(&self) -> Result<&str, KvValueError> {
        let mut chars = self.source.chars();

        if chars.next() == Some('\"') && chars.last() == Some('\"') {
            Ok(&self.source[1..self.source.len() - 1])
        } else {
            Err(KvValueError::NonString)
        }
    }

    fn ident(&self) -> Result<&str, KvValueError> {
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
}

#[derive(Clone, Debug)]
pub struct KvKey<'a> {
    ident: &'a str,
    path: KvPath<'a>,
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

    fn iter(&self) -> KvPathIter<'a> {
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
