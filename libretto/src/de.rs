use crate::error::{DeserializeError as Error, DeserializeErrorDesc as ErrorDesc};
use serde::de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::forward_to_deserialize_any;
use serde::Deserialize;

use crate::ast::{self, Expr, ExprDesc};

pub type Result<T> = std::result::Result<T, Error>;

pub struct Deserializer<'de> {
    input: &'de ast::Expr,
}

impl<'de> Deserializer<'de> {
    // By convention, `Deserializer` constructors are named like `from_xyz`.
    // That way basic use cases are satisfied by something like
    // `serde_json::from_str(...)` while advanced use cases that require a
    // deserializer can make one with `serde_json::Deserializer::from_str(...)`.
    pub fn from_expr(input: &'de ast::Expr) -> Self {
        Deserializer { input }
    }
}

// By convention, the public API of a Serde deserializer is one or more
// `from_xyz` methods such as `from_str`, `from_bytes`, or `from_reader`
// depending on what Rust types the deserializer is able to consume as input.
//
// This basic deserializer supports only `from_str`.
pub fn from_expr<'a, T>(s: &'a ast::Expr) -> Result<T>
where
    T: Deserialize<'a>,
{
    if s.needs_evaluation() {
        Err(ErrorDesc::Unevaluated(format!("{:?}", s)).with_pos(s.pos))
    } else {
        let deserializer = Deserializer::from_expr(s);
        T::deserialize(deserializer)
    }
}

// SERDE IS NOT A PARSING LIBRARY. This impl block defines a few basic parsing
// functions from scratch. More complicated formats may wish to use a dedicated
// parsing library to help implement their Serde deserializer.
// impl<'de> Deserializer<'de> {
// }

impl<'de, 'a> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option
        // unit unit_struct newtype_struct seq tuple
        // tuple_struct map struct enum identifier ignored_any
    }

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.input.desc {
            ExprDesc::Float(f) => visitor.visit_f32(*f),
            ExprDesc::Int(i) => visitor.visit_i32(*i),
            ExprDesc::Bool(b) => visitor.visit_bool(*b),
            ExprDesc::Char(c) => visitor.visit_char(*c),
            ExprDesc::String(s) => visitor.visit_borrowed_str(s),
            ExprDesc::Option(inner) => match &**inner {
                None => visitor.visit_none(),
                Some(s) => visitor.visit_some(Deserializer::from_expr(&s)),
            },
            s => Err(ErrorDesc::Unevaluated(format!("{:?}", s)).with_pos(self.input.pos)),
        }
    }

    // // An absent optional is represented as the JSON `null` and a present
    // // optional is represented as just the contained value.
    // //
    // // As commented in `Serializer` implementation, this is a lossy
    // // representation. For example the values `Some(())` and `None` both
    // // serialize as just `null`. Unfortunately this is typically what people
    // // expect when working with JSON. Other formats are encouraged to behave
    // // more intelligently if possible.
    // fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    // where
    //     V: Visitor<'de>,
    // {
    //     if self.input.starts_with("null") {
    //         self.input = &self.input["null".len()..];
    //         visitor.visit_none()
    //     } else {
    //         visitor.visit_some(self)
    //     }
    // }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let ExprDesc::Unit = self.input.desc {
            visitor.visit_unit()
        } else {
            Err(ErrorDesc::ExpectedUnit.with_pos(self.input.pos))
        }
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let ExprDesc::NamedTuple(tname, contents) = &self.input.desc {
            if name != tname {
                Err(ErrorDesc::WrongName(name.to_owned(), tname.to_owned())
                    .with_pos(self.input.pos))
            } else if !contents.is_empty() {
                Err(ErrorDesc::WrongTupleLength(0, contents.len()).with_pos(self.input.pos))
            } else {
                visitor.visit_unit()
            }
        } else {
            Err(ErrorDesc::ExpectedNamedTuple.with_pos(self.input.pos))
        }
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let ExprDesc::Struct(sname, _items) = &self.input.desc {
            if sname != name {
                Err(ErrorDesc::WrongName(name.to_owned(), sname.to_owned())
                    .with_pos(self.input.pos))
            } else {
                visitor.visit_newtype_struct(self)
                // TODO?
                // visitor.visit_newtype_struct(Deserializer::from_expr(ExprDesc::Object(self.items)))
            }
        } else {
            Err(ErrorDesc::ExpectedStruct.with_pos(self.input.pos))
        }
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.input.desc {
            ExprDesc::Array(contents) | ExprDesc::NamedTuple(_, contents) => {
                visitor.visit_seq(Items::new(contents))
            }
            _ => Err(ErrorDesc::ExpectedSequence.with_pos(self.input.pos)),
        }
    }

    // TODO check _len
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.input.desc {
            ExprDesc::Tuple(contents) => visitor.visit_seq(Items::new(contents)),
            _ => Err(ErrorDesc::ExpectedSequence.with_pos(self.input.pos)),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let ExprDesc::NamedTuple(tname, contents) = &self.input.desc {
            if name != tname {
                Err(ErrorDesc::WrongName(name.to_owned(), tname.to_owned())
                    .with_pos(self.input.pos))
            } else if contents.len() != len {
                Err(ErrorDesc::WrongTupleLength(len, contents.len()).with_pos(self.input.pos))
            } else {
                visitor.visit_unit()
            }
        } else {
            Err(ErrorDesc::ExpectedNamedTuple.with_pos(self.input.pos))
        }
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let ExprDesc::Object(items) = &self.input.desc {
            visitor.visit_map(Pairs::new(items))
        } else {
            Err(ErrorDesc::ExpectedMap.with_pos(self.input.pos))
        }
        // // Parse the opening brace of the map.
        // if self.next_char()? == '{' {
        //     // Give the visitor access to each entry of the map.
        //     let value = visitor.visit_map(CommaSeparated::new(&mut self))?;
        //     // Parse the closing brace of the map.
        //     if self.next_char()? == '}' {
        //         Ok(value)
        //     } else {
        //         Err(ErrorDesc::ExpectedMapEnd)
        //     }
        // } else {
        //     Err(ErrorDesc::ExpectedMap)
        // }
    }

    fn deserialize_struct<V>(
        self,
        ename: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // self.deserialize_map(visitor)
        if let ExprDesc::Struct(name, items) = &self.input.desc {
            if name == ename {
                visitor
                    .visit_map(Pairs::new(items))
                    .map_err(|e| e.with_pos(self.input.pos))
            } else {
                Err(ErrorDesc::WrongName(ename.to_string(), name.to_string())
                    .with_pos(self.input.pos))
            }
        } else {
            Err(ErrorDesc::ExpectedMap.with_pos(self.input.pos))
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // if let ExprDesc::
        // if variants.contains(x: &T)
        visitor.visit_enum(Enum::new(self.input))
        // if self.peek_char()? == '"' {
        //     // Visit a unit variant.
        //     visitor.visit_enum(self.parse_string()?.into_deserializer())
        // } else if self.next_char()? == '{' {
        //     // Visit a newtype variant, tuple variant, or struct variant.
        //     let value = visitor.visit_enum(Enum::new(self))?;
        //     // Parse the matching close brace.
        //     if self.next_char()? == '}' {
        //         Ok(value)
        //     } else {
        //         Err(ErrorDesc::ExpectedMapEnd)
        //     }
        // } else {
        //     Err(ErrorDesc::ExpectedEnum)
        // }
    }

    // An identifier in Serde is the type that identifies a field of a struct or
    // the variant of an enum. In JSON, struct fields and enum variants are
    // represented as strings. In other formats they may be represented as
    // numeric indices.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // Like `deserialize_any` but indicates to the `Deserializer` that it makes
    // no difference which `Visitor` method is called because the data is
    // ignored.
    //
    // Some deserializers are able to implement this more efficiently than
    // `deserialize_any`, for example by rapidly skipping over matched
    // delimiters without paying close attention to the data in between.
    //
    // Some formats are not able to implement this at all. Formats that can
    // implement `deserialize_any` and `deserialize_ignored_any` are known as
    // self-describing.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct Items<'a> {
    contents: &'a [Expr],
    index: usize,
}

impl<'a, 'de> Items<'a> {
    fn new(contents: &'a [Expr]) -> Self {
        Items { contents, index: 0 }
    }
}

struct Pairs<'a> {
    contents: &'a [(String, Expr)],
    index: usize,
}

impl<'a, 'de> Pairs<'a> {
    fn new(contents: &'a [(String, Expr)]) -> Self {
        Pairs { contents, index: 0 }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'a> SeqAccess<'a> for Items<'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'a>,
    {
        if self.index == self.contents.len() {
            return Ok(None);
        }
        self.index += 1;
        // Deserialize an array element.
        seed.deserialize(Deserializer::from_expr(&self.contents[self.index - 1]))
            .map(Some)
    }
}

pub struct KeyDeserializer<'de> {
    input: &'de str,
}

impl<'de> KeyDeserializer<'de> {
    // By convention, `Deserializer` constructors are named like `from_xyz`.
    // That way basic use cases are satisfied by something like
    // `serde_json::from_str(...)` while advanced use cases that require a
    // deserializer can make one with `serde_json::Deserializer::from_str(...)`.
    pub fn from_str(input: &'de str) -> Self {
        KeyDeserializer { input }
    }
}

impl<'de, 'a> de::Deserializer<'de> for KeyDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.input)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option
        unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'a> MapAccess<'a> for Pairs<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'a>,
    {
        if self.index == self.contents.len() {
            return Ok(None);
        }
        let (key, _v) = &self.contents[self.index];
        // self.index += 1;
        // Deserialize a map key.
        seed.deserialize(KeyDeserializer::from_str(key)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'a>,
    {
        // It doesn't make a difference whether the colon is parsed at the end
        // of `next_key_seed` or at the beginning of `next_value_seed`. In this
        // case the code is a bit simpler having it here.
        // if self.de.next_char()? != ':' {
        //     return Err(ErrorDesc::ExpectedMapColon);
        // }
        // Deserialize a map value.
        let (_key, v) = &self.contents[self.index];
        self.index += 1;
        seed.deserialize(Deserializer::from_expr(&v))
    }
}

struct Enum<'a> {
    // name: &'a str,
    expr: &'a Expr,
}

impl<'a> Enum<'a> {
    fn new(expr: &'a Expr) -> Self {
        Enum { expr }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'a> EnumAccess<'a> for Enum<'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'a>,
    {
        // The `deserialize_enum` method parsed a `{` character so we are
        // currently inside of a map. The seed will be deserializing itself from
        // the key of the map.
        match &self.expr.desc {
            ExprDesc::NamedTuple(name, _) | ExprDesc::Struct(name, _) => {
                let val = seed.deserialize(KeyDeserializer::from_str(name))?;
                Ok((val, self))
            }
            _ => Err(ErrorDesc::ExpectedEnum.with_pos(self.expr.pos)),
        }
        // Parse the colon separating map key from value.
        // if self.de.next_char()? == ':' {
        //     Ok((val, self))
        // } else {
        //     Err(ErrorDesc::ExpectedMapColon)
        // }
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'a> VariantAccess<'a> for Enum<'a> {
    type Error = Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<()> {
        match &self.expr.desc {
            ExprDesc::NamedTuple(_, v) if v.is_empty() => Ok(()),
            _ => Err(ErrorDesc::ExpectedEnum.with_pos(self.expr.pos)),
        }
    }

    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'a>,
    {
        if let ExprDesc::NamedTuple(_, v) = &self.expr.desc {
            if v.len() == 1 {
                seed.deserialize(Deserializer::from_expr(&v[0]))
            } else {
                Err(ErrorDesc::WrongTupleLength(0, v.len()).with_pos(self.expr.pos))
            }
        } else {
            Err(ErrorDesc::ExpectedNamedTuple.with_pos(self.expr.pos))
        }
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'a>,
    {
        if let ExprDesc::NamedTuple(_, v) = &self.expr.desc {
            if len == v.len() {
                de::Deserializer::deserialize_seq(Deserializer::from_expr(self.expr), visitor)
            } else {
                Err(ErrorDesc::WrongTupleLength(len, v.len()).with_pos(self.expr.pos))
            }
        } else {
            Err(ErrorDesc::ExpectedNamedTuple.with_pos(self.expr.pos))
        }
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'a>,
    {
        if let ExprDesc::Struct(_name, items) = &self.expr.desc {
            visitor.visit_map(Pairs::new(items))
        } else {
            Err(ErrorDesc::ExpectedStruct.with_pos(self.expr.pos))
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

// #[test]
// fn test_struct() {
//     #[derive(Deserialize, PartialEq, Debug)]
//     struct Test {
//         int: u32,
//         seq: Vec<String>,
//     }

//     let j = r#"{"int":1,"seq":["a","b"]}"#;
//     let expected = Test {
//         int: 1,
//         seq: vec!["a".to_owned(), "b".to_owned()],
//     };
//     assert_eq!(expected, from_str(j).unwrap());
// }

// #[test]
// fn test_enum() {
//     #[derive(Deserialize, PartialEq, Debug)]
//     enum E {
//         Unit,
//         Newtype(u32),
//         Tuple(u32, u32),
//         Struct { a: u32 },
//     }

//     let j = r#""Unit""#;
//     let expected = E::Unit;
//     assert_eq!(expected, from_str(j).unwrap());

//     let j = r#"{"Newtype":1}"#;
//     let expected = E::Newtype(1);
//     assert_eq!(expected, from_str(j).unwrap());

//     let j = r#"{"Tuple":[1,2]}"#;
//     let expected = E::Tuple(1, 2);
//     assert_eq!(expected, from_str(j).unwrap());

//     let j = r#"{"Struct":{"a":1}}"#;
//     let expected = E::Struct { a: 1 };
//     assert_eq!(expected, from_str(j).unwrap());
// }
