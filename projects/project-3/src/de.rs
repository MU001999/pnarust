use serde::de::{
    self, DeserializeSeed, EnumAccess, Error as _, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use serde::Deserialize;

use crate::{Error, Result};

pub struct Deserializer<'de> {
    input: &'de str,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer { input }
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;

    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::SerdeError(String::from("Trailing Characters")))
    }
}

impl<'de> Deserializer<'de> {
    fn peek_char(&mut self) -> Result<char> {
        self.input
            .chars()
            .next()
            .ok_or(Error::SerdeError(String::from("EOF")))
    }

    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    fn parse_term(&mut self) -> Result<&'de str> {
        match self.input.find(',') {
            Some(len) => {
                let s = &self.input[..len];
                self.input = &self.input[len + 1..];
                Ok(s)
            }
            None => Err(Error::SerdeError(String::from("EOF Meet"))),
        }
    }

    fn parse_integer(&mut self) -> Result<usize> {
        self.parse_term()?
            .parse::<usize>()
            .map_err(|_| Error::SerdeError(String::from("Expected integer")))
    }

    fn parse_string(&mut self) -> Result<&'de str> {
        if self.next_char()? == '\'' {
            let len = self.parse_integer()?;
            let s = &self.input[..len];

            if self.input.as_bytes()[len] == b',' {
                self.input = &self.input[len + 1..];
                Ok(s)
            } else {
                Err(Error::SerdeError(String::from("Expected ,")))
            }
        } else {
            Err(Error::SerdeError(String::from("Expected '")))
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input.starts_with("'4,true,") {
            self.input = &self.input["'4,true,".len()..];
            visitor.visit_bool(true)
        } else if self.input.starts_with("'5,false,") {
            self.input = &self.input["'5,false,".len()..];
            visitor.visit_bool(false)
        } else {
            Err(Error::SerdeError(String::from("Expected bool")))
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_string()?.parse::<i8>()
            .map_err(|_| Error::SerdeError(String::from("Expected i8")))?
        )
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_string()?.parse::<i16>()
            .map_err(|_| Error::SerdeError(String::from("Expected i16")))?
        )
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_string()?.parse::<i32>()
            .map_err(|_| Error::SerdeError(String::from("Expected i32")))?
        )
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_string()?.parse::<i64>()
            .map_err(|_| Error::SerdeError(String::from("Expected i64")))?
        )
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_string()?.parse::<u8>()
            .map_err(|_| Error::SerdeError(String::from("Expected u8")))?
        )
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_string()?.parse::<u16>()
            .map_err(|_| Error::SerdeError(String::from("Expected u16")))?
        )
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_string()?.parse::<u32>()
            .map_err(|_| Error::SerdeError(String::from("Expected u32")))?
        )
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_string()?.parse::<u64>()
            .map_err(|_| Error::SerdeError(String::from("Expected u64")))?
        )
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse_string()?.parse::<f32>()
            .map_err(|_| Error::SerdeError(String::from("Expected f32")))?
        )
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_string()?.parse::<f64>()
            .map_err(|_| Error::SerdeError(String::from("Expected f64")))?
        )
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(String::from(self.parse_string()?))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input.starts_with("=2,'4,Some,") {
            self.input = &self.input["=2,'4,Some,".len()..];
            visitor.visit_some(self)
        } else if self.input.starts_with("=1,'4,None,") {
            self.input = &self.input["=1,'4,None,".len()..];
            visitor.visit_none()
        } else {
            Err(Error::SerdeError(String::from("Expected option")))
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input.starts_with("=0,") {
            self.input = &self.input["=0,".len()..];
            visitor.visit_unit()
        } else {
            Err(Error::SerdeError(String::from("Expected =0,")))
        }
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.next_char()? == '=' {
            let len = self.parse_integer()?;
            Ok(visitor.visit_seq(CommaSeparated::new(self, len))?)
        } else {
            Err(Error::SerdeError(String::from("Expected =N,")))
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
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
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.next_char()? == '=' {
            let len = self.parse_integer()?;
            Ok(visitor.visit_map(CommaSeparated::new(self, len))?)
        } else {
            Err(Error::SerdeError(String::from("Expected =N,")))
        }
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.next_char()? == '=' {
            let len = self.parse_integer()?;
            visitor.visit_enum(Enum::new(self, len - 1))
        } else {
            Err(Error::SerdeError(String::from("Expected enum")))
        }
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
        unimplemented!()
    }
}

struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: usize,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        CommaSeparated { de, len }
    }
}

impl<'de, 'a> SeqAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        // Check if there are no more elements.
        if self.len == 0 {
            Ok(None)
        } else {
            self.len -= 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }
}

impl<'de, 'a> MapAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.len == 0 {
            Ok(None)
        } else {
            self.len -= 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        self.len -= 1;
        seed.deserialize(&mut *self.de)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: usize,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        Enum { de, len }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        Ok((seed.deserialize(&mut *self.de)?, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        if self.de.input.starts_with("=0,") {
            self.de.input = &self.de.input["=0,".len()..];
            Ok(())
        } else {
            Err(Error::SerdeError(String::from("Expected =0,")))
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(CommaSeparated::new(self.de, self.len))
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Ok(visitor.visit_map(CommaSeparated::new(self.de, self.len))?)
    }
}
