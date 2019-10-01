use crate::ast::{ExprDesc, Expr};

use serde::{ser, Serialize};

use crate::error::{Error, Result};

pub struct Serializer;

// By convention, the public API of a Serde serializer is one or more `to_abc`
// functions such as `to_string`, `to_bytes`, or `to_writer` depending on what
// Rust types the serializer is able to produce as output.
//
// This basic serializer supports only `to_string`.
pub fn to_expr<T>(value: &T) -> Result<Expr>
where
    T: Serialize,
{
    value.serialize(Serializer)
}

impl ser::Serializer for Serializer {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = Expr;

    // The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = SeqTracker;
    type SerializeTuple = TupleTracker;
    type SerializeTupleStruct = TupleStructTracker;
    type SerializeTupleVariant = TupleStructTracker;
    type SerializeMap = MapTracker;
    type SerializeStruct = StructTracker;
    type SerializeStructVariant = StructTracker;

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to JSON by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(ExprDesc::Bool(v).into())
    }

    // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(v))
    }

    // Not particularly efficient but this is example code anyway. A more
    // performant approach would be to use the `itoa` crate.
    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        Ok(ExprDesc::Int(v as i32).into())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(ExprDesc::Int(v as i32).into())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(ExprDesc::Float(v as f32).into())
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        Ok(ExprDesc::Char(v).into())
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea. For example it would emit invalid JSON if the input string
    // contains a '"' character.
    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(ExprDesc::String(v.to_owned()).into())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        // use serde::ser::SerializeSeq;
        // let mut seq = self.serialize_seq(Some(v.len()))?;
        // for byte in v {
        //     seq.serialize_element(byte)?;
        // }
        // seq.end()
        unimplemented!()
    }

    // An absent optional is represented as the JSON `null`.
    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(ExprDesc::Option(Box::new(None)).into())
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with JSON. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Ok(ExprDesc::Option(Box::new(Some(value.serialize(self)?))).into())
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // JSON as `null`.
    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(ExprDesc::Unit.into())
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to JSON as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        Ok(ExprDesc::NamedTuple(name.to_owned(), vec![]).into())
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(ExprDesc::NamedTuple(variant.to_owned(), vec![]).into())
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Ok(ExprDesc::NamedTuple(
            name.to_owned(),
            vec![value.serialize(self)?],
        ).into())
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to JSON in externally tagged form as `{ NAME: VALUE }`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Ok(ExprDesc::NamedTuple(
            variant.to_owned(),
            vec![value.serialize(self)?],
        ).into())
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in JSON is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in JSON because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SeqTracker::new())
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(TupleTracker::new())
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(TupleStructTracker::new(name.to_owned()))
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(TupleStructTracker::new(variant.to_owned()))
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapTracker::new())
    }

    // Structs look just like maps in JSON. In particular, JSON requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(StructTracker::new(name.to_owned()))
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(StructTracker::new(variant.to_owned()))
    }
}

pub struct SeqTracker {
    items: Vec<Expr>,
}
impl SeqTracker {
    fn new() -> Self {
        SeqTracker { items: vec![] }
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl ser::SerializeSeq for SeqTracker {
    // Must match the `Ok` type of the serializer.
    type Ok = Expr;
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(value.serialize(Serializer)?);
        Ok(())
    }

    // Close the sequence.
    fn end(self) -> Result<Self::Ok> {
        Ok(ExprDesc::Array(self.items).into())
    }
}

pub struct TupleTracker {
    items: Vec<Expr>,
}
impl TupleTracker {
    fn new() -> Self {
        TupleTracker { items: vec![] }
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for TupleTracker {
    type Ok = Expr;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(ExprDesc::Tuple(self.items).into())
    }
}

pub struct TupleStructTracker {
    name: String,
    items: Vec<Expr>,
}
impl TupleStructTracker {
    fn new(name: String) -> Self {
        TupleStructTracker {
            name,
            items: vec![],
        }
    }
}

// Same thing but for tuple structs.
impl ser::SerializeTupleStruct for TupleStructTracker {
    type Ok = Expr;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(ExprDesc::NamedTuple(self.name, self.items).into())
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a> ser::SerializeTupleVariant for TupleStructTracker {
    type Ok = Expr;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(ExprDesc::NamedTuple(self.name, self.items).into())
    }
}

pub struct MapTracker {
    key: Option<String>,
    items: Vec<(String, Expr)>,
}
impl MapTracker {
    fn new() -> Self {
        MapTracker {
            key: None,
            items: vec![],
        }
    }
}

impl ser::SerializeMap for MapTracker {
    type Ok = Expr;
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match key.serialize(Serializer)?.desc {
            ExprDesc::String(name) | ExprDesc::Ident(name) => self.key = Some(name),
            _ => unimplemented!(),
        }
        Ok(())
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = self.key.take();
        match key {
            None => panic!("Value wthout key"),
            Some(v) => {
                self.key = None;
                self.items.push((v, value.serialize(Serializer)?));
                Ok(())
            }
        }
    }

    fn end(self) -> Result<Self::Ok> {
        if self.key.is_some() {
            panic!("Unused key");
        }
        Ok(ExprDesc::Object(self.items).into())
    }
}

pub struct StructTracker {
    name: String,
    items: Vec<(String, Expr)>,
}
impl StructTracker {
    fn new(name: String) -> Self {
        StructTracker {
            name,
            items: vec![],
        }
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for StructTracker {
    type Ok = Expr;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.items
            .push((key.to_owned(), value.serialize(Serializer)?));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(ExprDesc::Struct(self.name, self.items).into())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for StructTracker {
    type Ok = Expr;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.items
            .push((key.to_owned(), value.serialize(Serializer)?));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(ExprDesc::Struct(self.name, self.items).into())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_struct() {
    #[derive(Serialize)]
    struct Test {
        int: u32,
        seq: Vec<&'static str>,
    }

    let _test = Test {
        int: 1,
        seq: vec!["a", "b"],
    };
    let _expected = r#"{"int":1,"seq":["a","b"]}"#;
    // assert_eq!(to_string(&test).unwrap(), expected);
}

#[test]
fn test_enum() {
    #[derive(Serialize)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    // let u = E::Unit;
    // let expected = r#""Unit""#;
    // assert_eq!(to_string(&u).unwrap(), expected);

    // let n = E::Newtype(1);
    // let expected = r#"{"Newtype":1}"#;
    // assert_eq!(to_string(&n).unwrap(), expected);

    // let t = E::Tuple(1, 2);
    // let expected = r#"{"Tuple":[1,2]}"#;
    // assert_eq!(to_string(&t).unwrap(), expected);

    // let s = E::Struct { a: 1 };
    // let expected = r#"{"Struct":{"a":1}}"#;
    // assert_eq!(to_string(&s).unwrap(), expected);
}
