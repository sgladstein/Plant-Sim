//! Full-state hash (§3.14, §11.4): every number of the world's serialized state is fed
//! to a 64-bit mixer as its bit pattern, in serialization order (plants by id, organs
//! by id, soil cells by index, struct fields in declaration order). Floats are never
//! formatted. Because the hash walks the same `Serialize` impls as a snapshot, anything
//! a snapshot keeps is hashed, and a resumed run can be compared bit for bit.

/// serde's single-precision method has to exist; the sim state holds no such value.
type Single = f32; // determinism: allow (serde trait signature only)

use serde::ser::{self, Serialize};

/// The hash accumulator: splitmix64 finalization over each word folded into the state.
#[derive(Clone, Debug)]
pub struct StateHasher {
    h: u64,
    words: u64,
}

impl Default for StateHasher {
    fn default() -> Self {
        Self::new()
    }
}

fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl StateHasher {
    /// A fresh hasher.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            h: 0x9E37_79B9_7F4A_7C15,
            words: 0,
        }
    }

    /// Adds one word.
    pub fn word(&mut self, w: u64) {
        self.h = mix(self.h ^ mix(w.wrapping_add(self.words)));
        self.words = self.words.wrapping_add(1);
    }

    /// The hash so far.
    #[must_use]
    pub fn finish(&self) -> u64 {
        mix(self.h ^ self.words)
    }
}

/// Hashes any serializable value.
#[must_use]
pub fn hash_of<T: Serialize + ?Sized>(value: &T) -> u64 {
    let mut h = StateHasher::new();
    // The serializer below never fails.
    if value.serialize(&mut h).is_err() {
        unreachable!("the state hasher never fails");
    }
    h.finish()
}

/// Error type the serializer API requires; never produced.
#[derive(Debug)]
pub struct Never;

impl core::fmt::Display for Never {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("state hasher error")
    }
}

impl std::error::Error for Never {}

impl ser::Error for Never {
    fn custom<T: core::fmt::Display>(_msg: T) -> Self {
        Never
    }
}

type R = Result<(), Never>;

// Tags keep differently shaped values from colliding (None vs Some(0), and so on).
const TAG_NONE: u64 = 0xA1;
const TAG_SOME: u64 = 0xA2;
const TAG_UNIT: u64 = 0xA3;
const TAG_SEQ: u64 = 0xA4;
const TAG_MAP: u64 = 0xA5;
const TAG_VARIANT: u64 = 0xA6;
const TAG_STR: u64 = 0xA7;

impl StateHasher {
    fn bytes(&mut self, b: &[u8]) {
        self.word(TAG_STR);
        self.word(b.len() as u64);
        for chunk in b.chunks(8) {
            let mut w = [0u8; 8];
            w[..chunk.len()].copy_from_slice(chunk);
            self.word(u64::from_le_bytes(w));
        }
    }
}

impl ser::Serializer for &mut StateHasher {
    type Ok = ();
    type Error = Never;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> R {
        self.word(u64::from(v));
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> R {
        self.serialize_i64(i64::from(v))
    }
    fn serialize_i16(self, v: i16) -> R {
        self.serialize_i64(i64::from(v))
    }
    fn serialize_i32(self, v: i32) -> R {
        self.serialize_i64(i64::from(v))
    }
    fn serialize_i64(self, v: i64) -> R {
        self.word(u64::from_le_bytes(v.to_le_bytes()));
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> R {
        self.serialize_u64(u64::from(v))
    }
    fn serialize_u16(self, v: u16) -> R {
        self.serialize_u64(u64::from(v))
    }
    fn serialize_u32(self, v: u32) -> R {
        self.serialize_u64(u64::from(v))
    }
    fn serialize_u64(self, v: u64) -> R {
        self.word(v);
        Ok(())
    }
    fn serialize_f32(self, v: Single) -> R {
        self.word(u64::from(v.to_bits()));
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> R {
        self.word(v.to_bits());
        Ok(())
    }
    fn serialize_char(self, v: char) -> R {
        self.word(u64::from(v));
        Ok(())
    }
    fn serialize_str(self, v: &str) -> R {
        self.bytes(v.as_bytes());
        Ok(())
    }
    fn serialize_bytes(self, v: &[u8]) -> R {
        self.bytes(v);
        Ok(())
    }
    fn serialize_none(self) -> R {
        self.word(TAG_NONE);
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> R {
        self.word(TAG_SOME);
        value.serialize(self)
    }
    fn serialize_unit(self) -> R {
        self.word(TAG_UNIT);
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> R {
        self.serialize_unit()
    }
    fn serialize_unit_variant(self, _name: &'static str, index: u32, _v: &'static str) -> R {
        self.word(TAG_VARIANT);
        self.word(u64::from(index));
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _name: &'static str, value: &T) -> R {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        index: u32,
        _v: &'static str,
        value: &T,
    ) -> R {
        self.word(TAG_VARIANT);
        self.word(u64::from(index));
        value.serialize(self)
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self, Never> {
        self.word(TAG_SEQ);
        self.word(len.map_or(u64::MAX, |n| n as u64));
        Ok(self)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self, Never> {
        Ok(self)
    }
    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self, Never> {
        Ok(self)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        index: u32,
        _v: &'static str,
        _len: usize,
    ) -> Result<Self, Never> {
        self.word(TAG_VARIANT);
        self.word(u64::from(index));
        Ok(self)
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self, Never> {
        self.word(TAG_MAP);
        self.word(len.map_or(u64::MAX, |n| n as u64));
        Ok(self)
    }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self, Never> {
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        index: u32,
        _v: &'static str,
        _len: usize,
    ) -> Result<Self, Never> {
        self.word(TAG_VARIANT);
        self.word(u64::from(index));
        Ok(self)
    }
}

impl ser::SerializeSeq for &mut StateHasher {
    type Ok = ();
    type Error = Never;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> R {
        v.serialize(&mut **self)
    }
    fn end(self) -> R {
        Ok(())
    }
}

impl ser::SerializeTuple for &mut StateHasher {
    type Ok = ();
    type Error = Never;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> R {
        v.serialize(&mut **self)
    }
    fn end(self) -> R {
        Ok(())
    }
}

impl ser::SerializeTupleStruct for &mut StateHasher {
    type Ok = ();
    type Error = Never;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> R {
        v.serialize(&mut **self)
    }
    fn end(self) -> R {
        Ok(())
    }
}

impl ser::SerializeTupleVariant for &mut StateHasher {
    type Ok = ();
    type Error = Never;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> R {
        v.serialize(&mut **self)
    }
    fn end(self) -> R {
        Ok(())
    }
}

impl ser::SerializeMap for &mut StateHasher {
    type Ok = ();
    type Error = Never;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, k: &T) -> R {
        k.serialize(&mut **self)
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, v: &T) -> R {
        v.serialize(&mut **self)
    }
    fn end(self) -> R {
        Ok(())
    }
}

impl ser::SerializeStruct for &mut StateHasher {
    type Ok = ();
    type Error = Never;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, _k: &'static str, v: &T) -> R {
        v.serialize(&mut **self)
    }
    fn end(self) -> R {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut StateHasher {
    type Ok = ();
    type Error = Never;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, _k: &'static str, v: &T) -> R {
        v.serialize(&mut **self)
    }
    fn end(self) -> R {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::hash_of;

    #[test]
    fn floats_hash_by_bits() {
        assert_ne!(hash_of(&0.0_f64), hash_of(&-0.0_f64));
        assert_eq!(hash_of(&1.5_f64), hash_of(&1.5_f64));
        assert_ne!(hash_of(&(1.0_f64, 2.0_f64)), hash_of(&(2.0_f64, 1.0_f64)));
    }

    #[test]
    fn shapes_do_not_collide() {
        assert_ne!(hash_of(&None::<u64>), hash_of(&Some(0_u64)));
        assert_ne!(hash_of(&vec![1_u64]), hash_of(&vec![1_u64, 0]));
        assert_ne!(hash_of("ab"), hash_of("ba"));
    }
}
