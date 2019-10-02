use crate::ast::{Expr, ExprDesc};

use serde::{ser, Serialize};

use crate::error::DeserializeError as Error;
pub type Result<T> = std::result::Result<T, Error>;

pub struct Serializer;

pub fn to_expr<T>(value: &T) -> Result<Expr>
where
    T: Serialize,
{
    value.serialize(Serializer)
}

impl ser::Serializer for Serializer {
    type Ok = Expr;
    type Error = Error;

    type SerializeSeq = SeqTracker;
    type SerializeTuple = TupleTracker;
    type SerializeTupleStruct = TupleStructTracker;
    type SerializeTupleVariant = TupleStructTracker;
    type SerializeMap = MapTracker;
    type SerializeStruct = StructTracker;
    type SerializeStructVariant = StructTracker;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(ExprDesc::Bool(v).into())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(v))
    }

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

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        Ok(ExprDesc::Char(v).into())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(ExprDesc::String(v.to_owned()).into())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(ExprDesc::Option(Box::new(None)).into())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Ok(ExprDesc::Option(Box::new(Some(value.serialize(self)?))).into())
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(ExprDesc::Unit.into())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        Ok(ExprDesc::NamedTuple(name.to_owned(), vec![]).into())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(ExprDesc::NamedTuple(variant.to_owned(), vec![]).into())
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Ok(ExprDesc::NamedTuple(name.to_owned(), vec![value.serialize(self)?]).into())
    }

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
        Ok(ExprDesc::NamedTuple(variant.to_owned(), vec![value.serialize(self)?]).into())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SeqTracker::new())
    }

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

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(TupleStructTracker::new(variant.to_owned()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapTracker::new())
    }

    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(StructTracker::new(name.to_owned()))
    }

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

impl ser::SerializeSeq for SeqTracker {
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
