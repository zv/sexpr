use serde::ser::{self, Serialize};

use error::{SerdeError, SerdeResult};

use std::str::from_utf8_unchecked;

pub struct Serializer {
    // This string starts empty and JSON is appended as values are serialized.
    output: String,
}

fn escape_str(v: &str) -> String {
    let mut wr = String::new();
    wr.push_str("\"");

    let mut start = 0;

    for (i, byte) in v.bytes().enumerate() {
        let escaped = match byte {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            b'\x00' => "\\u0000",
            b'\x01' => "\\u0001",
            b'\x02' => "\\u0002",
            b'\x03' => "\\u0003",
            b'\x04' => "\\u0004",
            b'\x05' => "\\u0005",
            b'\x06' => "\\u0006",
            b'\x07' => "\\u0007",
            b'\x08' => "\\b",
            b'\t' => "\\t",
            b'\n' => "\\n",
            b'\x0b' => "\\u000b",
            b'\x0c' => "\\f",
            b'\r' => "\\r",
            b'\x0e' => "\\u000e",
            b'\x0f' => "\\u000f",
            b'\x10' => "\\u0010",
            b'\x11' => "\\u0011",
            b'\x12' => "\\u0012",
            b'\x13' => "\\u0013",
            b'\x14' => "\\u0014",
            b'\x15' => "\\u0015",
            b'\x16' => "\\u0016",
            b'\x17' => "\\u0017",
            b'\x18' => "\\u0018",
            b'\x19' => "\\u0019",
            b'\x1a' => "\\u001a",
            b'\x1b' => "\\u001b",
            b'\x1c' => "\\u001c",
            b'\x1d' => "\\u001d",
            b'\x1e' => "\\u001e",
            b'\x1f' => "\\u001f",
            b'\x7f' => "\\u007f",
            _ => { continue; }
        };

        if start < i {
            wr.push_str(&v[start..i]);
        }

        wr.push_str(escaped);

        start = i + 1;
    }

    if start != v.len() {
        wr.push_str(&v[start..]);
    }

    wr.push_str("\"");
    wr
}

fn escape_char(v: char) -> String {
    let buf = [0; 4];
    let buf = unsafe { from_utf8_unchecked(&buf[..v.len_utf8()]) };
    escape_str(buf)
}

// By convention, the public API of a Serde deserializer is one or more `to_abc`
// functions such as `to_string`, `to_bytes`, or `to_writer` depending on what
// Rust types the serializer is able to produce as output.
//
// This basic serializer supports only `to_string`.
pub fn to_string<T>(value: &T) -> SerdeResult<String>
    where T: Serialize
{
    let mut serializer = Serializer { output: String::new() };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}



impl<'a> ser::Serializer for &'a mut Serializer {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = SerdeError;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

   // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> SerdeResult<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> SerdeResult<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> SerdeResult<()> {
        self.serialize_i64(v as i64)
    }

    // Not particularly efficient but this is example code anyway. A more
    // performant approach would be to use the `itoa` crate.
    fn serialize_i64(self, v: i64) -> SerdeResult<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> SerdeResult<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> SerdeResult<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> SerdeResult<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> SerdeResult<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> SerdeResult<()> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> SerdeResult<()> {
        self.output += &v.to_string();
        Ok(())
    }


    fn serialize_bool(self, v: bool) -> SerdeResult<()> {
        // TODO: XXX
        // There are a number of S-expression boolean encoding variants, we
        // should check our configuration for the correct boolean encoding type.
        self.output += if v { "#t" } else { "#f" };
        Ok(())
    }

    fn serialize_char(self, v: char) -> SerdeResult<()> {
        self.output += &escape_char(v);
        Ok(())
    }

    fn serialize_str(self, v: &str) -> SerdeResult<()> {
        self.output += &escape_str(v);
        Ok(())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, v: &[u8]) -> SerdeResult<()> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> SerdeResult<()> {
        self.serialize_unit()
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with JSON. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> SerdeResult<()>
        where T: ?Sized + Serialize
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // JSON as `null`.
    fn serialize_unit(self) -> SerdeResult<()> {
        self.output += "()";
        Ok(())
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to JSON as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, name: &'static str) -> SerdeResult<()> {
        self.output += name;
        Ok(())
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str
    ) -> SerdeResult<()> {
        self.output += &format!("{}", variant);
        Ok(())
    }


    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> SerdeResult<()>
        where T: ?Sized + Serialize
    {
        value.serialize(self)
    }


    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to S-expression in externally tagged form as `(NAME VALUE)`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T
    ) -> SerdeResult<()>
        where T: ?Sized + Serialize
    {
        self.output += "(";
        variant.serialize(&mut *self)?;
        self.output += " ";
        value.serialize(&mut *self)?;
        self.output += ")";
        Ok(())
    }

    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in Sexp is `(`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in Sexp because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, _len: Option<usize>) -> SerdeResult<Self::SerializeSeq> {
        self.output += "(";
        Ok(self)
    }

    // Tuples look just like sequences in Sexp. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> SerdeResult<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize
    ) -> SerdeResult<Self::SerializeTupleStruct> {
        self.output += &format!("((Tvariant {}) ", name);
        self.serialize_seq(Some(len))
    }

    // Tuple variants are represented in Sexp as `( NAME  (DATA...) )`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize
    ) -> SerdeResult<Self::SerializeTupleVariant> {
        self.output += &format!("((STVvariant {}) (", variant);
        Ok(self)
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> SerdeResult<Self::SerializeMap> {
        self.output += "(";
        Ok(self)
    }

    // Structs are similar to maps in S-expressions, in particular we have to serialize
    // the field names of the struct.
    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize
    ) -> SerdeResult<Self::SerializeStruct> {
        self.output += &format!("((Svariant {}) ", name);
        self.output += "(";
        Ok(self)
    }

    // Struct variants are represented in Sexp as `(NAME ( K . V, ... ))`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize
    ) -> SerdeResult<Self::SerializeStructVariant> {
        self.output += &format!("((SVvariant {}) ", variant);
        self.output += "(";
        Ok(self)
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a> ser::SerializeSeq for &'a mut Serializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = SerdeError;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> SerdeResult<()>
        where T: ?Sized + Serialize
    {
        if !self.output.ends_with('(') {
            self.output += " ";
        }
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> SerdeResult<()> {
        self.output += ")";
        Ok(())
    }
}

// A tuple is serialized in sequence
impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = SerdeError;

    fn serialize_element<T>(&mut self, value: &T) -> SerdeResult<()>
        where T: ?Sized + Serialize
    {
        if !self.output.ends_with('(') {
            self.output += " ";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> SerdeResult<()> {
        self.output += ")";
        Ok(())
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = SerdeError;

    fn serialize_field<T>(&mut self, value: &T) -> SerdeResult<()>
        where T: ?Sized + Serialize
    {
        if !self.output.ends_with('(') {
            self.output += " ";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> SerdeResult<()> {
        self.output += "))";
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = SerdeError;

    fn serialize_field<T>(&mut self, value: &T) -> SerdeResult<()>
        where T: ?Sized + Serialize
    {
        if !self.output.ends_with('(') {
            self.output += " ";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> SerdeResult<()> {
        self.output += "))";
        Ok(())
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = SerdeError;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> SerdeResult<()>
        where T: ?Sized + Serialize
    {
        if !self.output.ends_with('(') {
            self.output += " ";
        }
        self.output += "(";
        key.serialize(&mut **self)
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> SerdeResult<()>
        where T: ?Sized + Serialize
    {
        self.output += " . ";
        value.serialize(&mut **self);
        self.output += ")";
        Ok(())
    }

    fn end(self) -> SerdeResult<()> {
        self.output += ")";
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = SerdeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> SerdeResult<()>
        where T: ?Sized + Serialize
    {
        if self.output.ends_with(')') {
            self.output += " ";
        }
        self.output += "(";
        self.output += key;
        self.output += " ";
        value.serialize(&mut **self);
        self.output += ")";
        Ok(())
    }

    fn end(self) -> SerdeResult<()> {
        self.output += ")";
        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = SerdeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> SerdeResult<()>
        where T: ?Sized + Serialize
    {
        if self.output.ends_with(')') {
            self.output += " ";
        }
        self.output += "(";
        self.output += key;
        self.output += " ";
        value.serialize(&mut **self);
        self.output += ")";
        Ok(())
    }

    fn end(self) -> SerdeResult<()> {
        self.output += ")";
        Ok(())
    }
}
