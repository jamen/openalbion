//! Reading one `.tng` statement into a typed field.
//!
//! [`crate::text`] hands over a path and an uninterpreted value; this is where
//! "`PositionX` is an `f32`" is decided. Everything a `.tng` can say about a
//! field reduces to one of a handful of shapes, and each is an implementation of
//! [`ReadField`]:
//!
//! | Written as | Read into |
//! |---|---|
//! | `Health 1.0;` | `f32`, `i32`, `u64`, `bool`, `String` |
//! | `Colour CRGBColour(100,50,10,255);` | [`Rgba`] |
//! | `CoordBase C3DCoordF(1,2,3);` | `[f32; 3]` |
//! | `LookVector.X 1.0;` | the same `[f32; 3]`, one axis at a time |
//! | `NavPosition0 C2DCoordF(1,2);` | `[f32; 2]` |
//! | `ContainerContents[0] "OBJECT_X";` | `Vec<String>` |
//! | `KeyCameras[0].Position C3DCoordF(…);` | `Vec<KeyCamera>` |
//! | absent on some things | `Option<T>` of any of the above |
//!
//! The two array forms are the same rule applied twice: `Vec<T>` consumes one
//! index segment and hands the rest to `T`, so `Shape[0].pos[3].X` reaches an
//! `f32` through `Vec<Shape>` → `Vec<[f32; 3]>` → axis, with no special case.

use crate::text::{PathSegment, Value};
use derive_more::{Display, Error};

/// An 8-bit colour, from `CRGBColour(r, g, b, a)`.
///
/// `CRGBColour` (`_core/L7.hpp:24`) stores `B,G,R,A` in memory — the D3DCOLOR
/// layout — but its constructor is inlined away everywhere, so the *argument*
/// order is not readable from the decomp.
///
// UNVERIFIED: argument order taken as (R, G, B, A) from the authored data.
// Torches are written `CRGBColour(100,50,10,255)` and cold lights
// `CRGBColour(10,50,90,255)`; read as RGB those are warm and cool respectively,
// and read as BGR they are exactly backwards. The fourth argument is 255 on
// every one of the 309 colours in the game, which is what an alpha looks like.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// A `CRightHandedSet`: the forward and up vectors that orient a thing.
///
/// The same type serves `CTCPhysicsStandard::RHSet` (written as six scalars,
/// `RHSetForwardX` … `RHSetUpZ`) and `CTCCameraPointDefinitionBase::CoordAxis`
/// (written as two `C3DCoordF` calls). Both are read into this.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct RightHandedSet {
    pub forward: [f32; 3],
    pub up: [f32; 3],
}

/// Why a statement could not be read into its field.
#[derive(Clone, Debug, Display, Error, PartialEq, Eq)]
pub enum FieldErrorKind {
    #[display("unknown field")]
    UnknownField,
    #[display("unknown component class")]
    UnknownComponent,
    #[display("expected {expected}")]
    WrongType { expected: &'static str },
    /// A bare `Name;` where a field was expected. No component in the game
    /// writes one.
    #[display("expected a value")]
    MissingValue,
    /// A `StartCTC…` inside a component block. Components never nest.
    #[display("unexpected nested block")]
    UnexpectedNestedBlock,
    /// Path segments the field has no meaning for — `PositionX[0]`, or
    /// `CoordBase.W`.
    #[display("this field takes no {_0}")]
    UnexpectedPath(#[error(not(source))] &'static str),
    /// A guard against a malformed index allocating unboundedly. The largest in
    /// the game is 23.
    #[display("index {index} is beyond the {MAX_INDEX} this format allows")]
    IndexTooLarge { index: usize },
}

/// The largest array index a `.tng` may use. The corpus reaches 23
/// (`Shape[0].pos[23]`); this only exists so a corrupt file cannot ask for a
/// four-billion-element `Vec`.
const MAX_INDEX: usize = 4096;

/// A field that a `.tng` statement can be read into.
///
/// `rest` is whatever is left of the statement's path after the segment that
/// selected this field, so a `Vec` sees `[Index(3), Field("X")]` and passes
/// `[Field("X")]` down to its element.
pub trait ReadField {
    fn read_field(
        &mut self,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind>;
}

/// The scalar leaves. Each takes the whole value and no further path.
macro_rules! scalar {
    ($ty:ty, $expected:literal, |$value:ident| $get:expr) => {
        impl ReadField for $ty {
            fn read_field(
                &mut self,
                rest: &[PathSegment<'_>],
                $value: &Value<'_>,
            ) -> Result<(), FieldErrorKind> {
                if !rest.is_empty() {
                    return Err(FieldErrorKind::UnexpectedPath("index or sub-field"));
                }
                *self = $get.ok_or(FieldErrorKind::WrongType {
                    expected: $expected,
                })?;
                Ok(())
            }
        }
    };
}

scalar!(f32, "a number", |value| value.as_f32());
scalar!(i32, "a whole number", |value| value.as_i32());
// UIDs are `unsigned long long` and run to within a few thousand of `u64::MAX`.
scalar!(u64, "a UID", |value| value.as_u64());
scalar!(bool, "TRUE or FALSE", |value| value.as_bool());
scalar!(String, "a name or string", |value| value
    .as_str()
    .map(str::to_string));

/// A field that is read and thrown away because it duplicates something the
/// model already holds — an array length that is also the array's length.
///
/// Declaring it keeps each struct a *complete* inventory of what the file
/// writes, so "not in the struct" always means "not in the format" and the
/// unknown-field error stays meaningful. Each use documents what makes it
/// redundant, and the corpus test proves the two agree everywhere.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Redundant;

impl ReadField for Redundant {
    fn read_field(
        &mut self,
        rest: &[PathSegment<'_>],
        _value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        if rest.is_empty() {
            Ok(())
        } else {
            Err(FieldErrorKind::UnexpectedPath("index or sub-field"))
        }
    }
}

impl<T: ReadField + Default> ReadField for Option<T> {
    fn read_field(
        &mut self,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        self.get_or_insert_with(Default::default)
            .read_field(rest, value)
    }
}

impl<T: ReadField + Default> ReadField for Vec<T> {
    fn read_field(
        &mut self,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        let [PathSegment::Index(index), tail @ ..] = rest else {
            return Err(FieldErrorKind::UnexpectedPath("value without an index"));
        };
        let index = index
            .value
            .as_i32()
            .filter(|i| *i >= 0)
            .ok_or(FieldErrorKind::WrongType {
                expected: "a non-negative index",
            })? as usize;
        if index >= MAX_INDEX {
            return Err(FieldErrorKind::IndexTooLarge { index });
        }
        // The game writes indices densely from 0, so growth never leaves a
        // default element behind in practice — `arrays_are_dense` pins that.
        if index >= self.len() {
            self.resize_with(index + 1, Default::default);
        }
        self[index].read_field(tail, value)
    }
}

/// A vector written either whole (`C3DCoordF(x, y, z)`) or one axis at a time
/// (`LookVector.X`). Both occur, sometimes for different fields of the same
/// component, so both are the same type here.
impl ReadField for [f32; 3] {
    fn read_field(
        &mut self,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match rest {
            [] => read_coord(self, value, "C3DCoordF", "C3DCoordF(x, y, z)"),
            [PathSegment::Field(axis)] => {
                let index = axis_index(axis, 3)?;
                self[index] = value.as_f32().ok_or(FieldErrorKind::WrongType {
                    expected: "a number",
                })?;
                Ok(())
            }
            _ => Err(FieldErrorKind::UnexpectedPath("index")),
        }
    }
}

impl ReadField for [f32; 2] {
    fn read_field(
        &mut self,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match rest {
            [] => read_coord(self, value, "C2DCoordF", "C2DCoordF(x, y)"),
            [PathSegment::Field(axis)] => {
                let index = axis_index(axis, 2)?;
                self[index] = value.as_f32().ok_or(FieldErrorKind::WrongType {
                    expected: "a number",
                })?;
                Ok(())
            }
            _ => Err(FieldErrorKind::UnexpectedPath("index")),
        }
    }
}

impl ReadField for RightHandedSet {
    fn read_field(
        &mut self,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match rest {
            [PathSegment::Field(half), tail @ ..] => match *half {
                "Forward" => self.forward.read_field(tail, value),
                "Up" => self.up.read_field(tail, value),
                _ => Err(FieldErrorKind::UnknownField),
            },
            _ => Err(FieldErrorKind::UnexpectedPath("value without a half")),
        }
    }
}

impl ReadField for Rgba {
    fn read_field(
        &mut self,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        if !rest.is_empty() {
            return Err(FieldErrorKind::UnexpectedPath("index or sub-field"));
        }
        let expected = "CRGBColour(r, g, b, a)";
        let call = value
            .as_call()
            .filter(|call| call.name == "CRGBColour" && call.args.len() == 4)
            .ok_or(FieldErrorKind::WrongType { expected })?;
        let mut channels = [0u8; 4];
        for (channel, arg) in channels.iter_mut().zip(&call.args) {
            let n = arg
                .value
                .as_f32()
                .ok_or(FieldErrorKind::WrongType { expected })?;
            *channel = n.round().clamp(0.0, 255.0) as u8;
        }
        *self = Rgba {
            r: channels[0],
            g: channels[1],
            b: channels[2],
            a: channels[3],
        };
        Ok(())
    }
}

fn axis_index(axis: &str, len: usize) -> Result<usize, FieldErrorKind> {
    let index = match axis {
        "X" => 0,
        "Y" => 1,
        "Z" => 2,
        _ => return Err(FieldErrorKind::UnknownField),
    };
    (index < len)
        .then_some(index)
        .ok_or(FieldErrorKind::UnknownField)
}

fn read_coord<const N: usize>(
    out: &mut [f32; N],
    value: &Value<'_>,
    ctor: &str,
    expected: &'static str,
) -> Result<(), FieldErrorKind> {
    let call = value
        .as_call()
        .filter(|call| call.name == ctor && call.args.len() == N)
        .ok_or(FieldErrorKind::WrongType { expected })?;
    for (slot, arg) in out.iter_mut().zip(&call.args) {
        *slot = arg
            .value
            .as_f32()
            .ok_or(FieldErrorKind::WrongType { expected })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text;

    /// Parse `input` as one statement and read it into `target`.
    fn read<T: ReadField>(target: &mut T, input: &str) -> Result<(), FieldErrorKind> {
        let body = text::parse(input).expect("parse");
        let field = body.fields().next().expect("a field");
        let [_, rest @ ..] = field.path.segments.as_slice() else {
            unreachable!()
        };
        target.read_field(rest, &field.value.value)
    }

    #[test]
    fn scalars() {
        let mut f = 0.0f32;
        read(&mut f, "X -0.5;").unwrap();
        assert_eq!(f, -0.5);

        let mut uid = 0u64;
        read(&mut uid, "X 18446741874686301490;").unwrap();
        assert_eq!(uid, 18446741874686301490);

        let mut b = false;
        read(&mut b, "X TRUE;").unwrap();
        assert!(b);

        // A bare identifier and a quoted string are the same thing to the format.
        let mut s = String::new();
        read(&mut s, "X GuardTrack;").unwrap();
        assert_eq!(s, "GuardTrack");
        read(&mut s, "X \"NULL\";").unwrap();
        assert_eq!(s, "NULL");
    }

    #[test]
    fn wrong_type_is_an_error_not_a_default() {
        let mut f = 1.0f32;
        assert_eq!(
            read(&mut f, "X TRUE;"),
            Err(FieldErrorKind::WrongType {
                expected: "a number"
            })
        );
        assert_eq!(f, 1.0, "the field must be left alone");
    }

    #[test]
    fn option_is_set_on_first_write() {
        let mut o: Option<f32> = None;
        read(&mut o, "X 2.5;").unwrap();
        assert_eq!(o, Some(2.5));
    }

    #[test]
    fn vector_whole_or_by_axis() {
        let mut v = [0.0f32; 3];
        read(&mut v, "X C3DCoordF(1.0,2.0,3.0);").unwrap();
        assert_eq!(v, [1.0, 2.0, 3.0]);

        let mut v = [0.0f32; 3];
        read(&mut v, "X.X 1.0;").unwrap();
        read(&mut v, "X.Z 3.0;").unwrap();
        assert_eq!(v, [1.0, 0.0, 3.0]);

        let mut v = [0.0f32; 2];
        read(&mut v, "X C2DCoordF(4.0,5.0);").unwrap();
        assert_eq!(v, [4.0, 5.0]);
        assert_eq!(read(&mut v, "X.Z 1.0;"), Err(FieldErrorKind::UnknownField));
    }

    #[test]
    fn colour() {
        let mut c = Rgba::default();
        read(&mut c, "X CRGBColour(100,50,10,255);").unwrap();
        assert_eq!(
            c,
            Rgba {
                r: 100,
                g: 50,
                b: 10,
                a: 255
            }
        );
        assert!(read(&mut c, "X C3DCoordF(1.0,2.0,3.0);").is_err());
    }

    #[test]
    fn indexed_array_grows() {
        let mut v: Vec<String> = Vec::new();
        read(&mut v, "X[2] \"C\";").unwrap();
        read(&mut v, "X[0] \"A\";").unwrap();
        assert_eq!(v, vec!["A".to_string(), String::new(), "C".to_string()]);
        assert_eq!(
            read(&mut v, "X[999999] \"Z\";"),
            Err(FieldErrorKind::IndexTooLarge { index: 999999 })
        );
        assert_eq!(
            read(&mut v, "X \"no index\";"),
            Err(FieldErrorKind::UnexpectedPath("value without an index"))
        );
    }

    #[derive(Debug, Default, PartialEq)]
    struct Point {
        position: [f32; 3],
        name: String,
    }

    impl ReadField for Point {
        fn read_field(
            &mut self,
            rest: &[PathSegment<'_>],
            value: &Value<'_>,
        ) -> Result<(), FieldErrorKind> {
            let [PathSegment::Field(name), tail @ ..] = rest else {
                return Err(FieldErrorKind::UnexpectedPath("value without a field name"));
            };
            match *name {
                "Position" => self.position.read_field(tail, value),
                "Name" => self.name.read_field(tail, value),
                _ => Err(FieldErrorKind::UnknownField),
            }
        }
    }

    #[test]
    fn nested_struct_arrays() {
        // The shape `KeyCameras[0].Position` and `Shape[0].pos[3].X` share.
        let mut v: Vec<Point> = Vec::new();
        read(&mut v, "X[1].Position C3DCoordF(1.0,2.0,3.0);").unwrap();
        read(&mut v, "X[1].Name \"second\";").unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!(v[1].position, [1.0, 2.0, 3.0]);
        assert_eq!(v[1].name, "second");
        assert_eq!(v[0], Point::default());
        assert_eq!(
            read(&mut v, "X[0].Nope 1.0;"),
            Err(FieldErrorKind::UnknownField)
        );
    }

    #[test]
    fn array_of_arrays() {
        let mut v: Vec<Vec<[f32; 3]>> = Vec::new();
        read(&mut v, "X[0][2].Y 7.0;").unwrap();
        assert_eq!(v[0][2], [0.0, 7.0, 0.0]);
    }
}
