// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementations of serialization for structures found in libcollections

use std::hash::Hash;


use serialize::Encodable;
use serialize::cap_capacity;
use serialize::Encoder;

use std::collections::{LinkedList, VecDeque, BTreeMap, BTreeSet, HashMap, HashSet};

impl<
    T: Encodable
> Encodable for LinkedList<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            for (i, e) in self.iter().enumerate() {
                try!(s.emit_seq_elt(i, |s| e.encode(s)));
            }
            Ok(())
        })
    }
}

impl<T: Encodable> Encodable for VecDeque<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            for (i, e) in self.iter().enumerate() {
                try!(s.emit_seq_elt(i, |s| e.encode(s)));
            }
            Ok(())
        })
    }
}

impl<
    K: Encodable + Ord,
    V: Encodable
> Encodable for BTreeMap<K, V> {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_map(self.len(), |e| {
            let mut i = 0;
            for (key, val) in self.iter() {
                try!(e.emit_map_elt_key(i, |e| key.encode(e)));
                try!(e.emit_map_elt_val(i, |e| val.encode(e)));
                i += 1;
            }
            Ok(())
        })
    }
}

impl<
    T: Encodable + Ord
> Encodable for BTreeSet<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            let mut i = 0;
            for e in self.iter() {
                try!(s.emit_seq_elt(i, |s| e.encode(s)));
                i += 1;
            }
            Ok(())
        })
    }
}

impl<K, V> Encodable for HashMap<K, V>
    where K: Encodable + Hash + Eq,
          V: Encodable,
{
    fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
        e.emit_map(self.len(), |e| {
            let mut i = 0;
            for (key, val) in self.iter() {
                try!(e.emit_map_elt_key(i, |e| key.encode(e)));
                try!(e.emit_map_elt_val(i, |e| val.encode(e)));
                i += 1;
            }
            Ok(())
        })
    }
}

impl<T> Encodable for HashSet<T> where T: Encodable + Hash + Eq {
    fn encode<E: Encoder>(&self, s: &mut E) -> Result<(), E::Error> {
        s.emit_seq(self.len(), |s| {
            let mut i = 0;
            for e in self.iter() {
                try!(s.emit_seq_elt(i, |s| e.encode(s)));
                i += 1;
            }
            Ok(())
        })
    }
}
