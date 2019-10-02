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
    pub fn from_expr(input: &'de ast::Expr) -> Self {
        Deserializer { input }
    }
}

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

impl<'de, 'a> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option
    }

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
        let pos = self.input.pos;
        if let ExprDesc::Struct(sname, _items) = &self.input.desc {
            if sname != name {
                Err(ErrorDesc::WrongName(name.to_owned(), sname.to_owned())
                    .with_pos(self.input.pos))
            } else {
                visitor.visit_newtype_struct(self)
                    .map_err(|e| e.with_pos(pos))
            }
        } else {
            Err(ErrorDesc::ExpectedStruct.with_pos(self.input.pos))
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.input.desc {
            ExprDesc::Array(contents) | ExprDesc::NamedTuple(_, contents) => {
                visitor.visit_seq(Items::new(contents))
                    .map_err(|e| e.with_pos(self.input.pos))
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
            ExprDesc::Tuple(contents) => visitor.visit_seq(Items::new(contents))
                    .map_err(|e| e.with_pos(self.input.pos))
            ,
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

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let ExprDesc::Object(items) = &self.input.desc {
            visitor.visit_map(Pairs::new(items))
                    .map_err(|e| e.with_pos(self.input.pos))
        } else {
            Err(ErrorDesc::ExpectedMap.with_pos(self.input.pos))
        }
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
        visitor.visit_enum(Enum::new(self.input))
                    .map_err(|e| e.with_pos(self.input.pos))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

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
        seed.deserialize(KeyDeserializer::from_str(key)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'a>,
    {
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

impl<'a> EnumAccess<'a> for Enum<'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'a>,
    {
        match &self.expr.desc {
            ExprDesc::NamedTuple(name, _) | ExprDesc::Struct(name, _) => {
                let val = seed.deserialize(KeyDeserializer::from_str(name))?;
                Ok((val, self))
            }
            _ => Err(ErrorDesc::ExpectedEnum.with_pos(self.expr.pos)),
        }
    }
}

impl<'a> VariantAccess<'a> for Enum<'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        match &self.expr.desc {
            ExprDesc::NamedTuple(_, v) if v.is_empty() => Ok(()),
            _ => Err(ErrorDesc::ExpectedEnum.with_pos(self.expr.pos)),
        }
    }

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
