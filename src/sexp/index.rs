use std::fmt;
use std::ops;

use super::Sexp;

/// A type that can be used to index into a `sexpr::Sexp`. See the `get`
/// and `get_mut` methods of `Sexp`.
///
/// This trait is sealed and cannot be implemented for types outside of
/// `sexpr`.
pub trait Index: private::Sealed {
    /// Return None if the key is not already in the array or object.
    #[doc(hidden)]
    fn index_into<'v>(&self, v: &'v Sexp) -> Option<&'v Sexp>;

    /// Return None if the key is not already in the array or object.
    #[doc(hidden)]
    fn index_into_mut<'v>(&self, v: &'v mut Sexp) -> Option<&'v mut Sexp>;

    /// Panic if array index out of bounds. If key is not already in the object,
    /// insert it with a value of null. Panic if Sexp is a type that cannot be
    /// indexed into, except if Sexp is null then it can be treated as an empty
    /// object.
    #[doc(hidden)]
    fn index_or_insert<'v>(&self, v: &'v mut Sexp) -> &'v mut Sexp;
}

impl Index for usize {
    fn index_into<'v>(&self, v: &'v Sexp) -> Option<&'v Sexp> {
        match *v {
            Sexp::List(ref vec) => vec.get(*self),
            _ => None,
        }
    }
    fn index_into_mut<'v>(&self, v: &'v mut Sexp) -> Option<&'v mut Sexp> {
        match *v {
            Sexp::List(ref mut vec) => vec.get_mut(*self),
            _ => None,
        }
    }
    fn index_or_insert<'v>(&self, v: &'v mut Sexp) -> &'v mut Sexp {
        match *v {
            Sexp::List(ref mut vec) => {
                let len = vec.len();
                vec.get_mut(*self)
                    .unwrap_or_else(
                        || {
                            panic!(
                                "cannot access index {} of JSON array of length {}",
                                self,
                                len
                            )
                        },
                    )
            }
            _ => panic!("cannot access index {} of JSON {}", self, Type(v)),
        }
    }
}

impl Index for str {
    fn index_into<'v>(&self, v: &'v Sexp) -> Option<&'v Sexp> {
        match v {
            &Sexp::List(_) => v.get(self),
            _ => None,
        }
    }
    fn index_into_mut<'v>(&self, _v: &'v mut Sexp) -> Option<&'v mut Sexp> {
        unimplemented!()
    }
    fn index_or_insert<'v>(&self, _v: &'v mut Sexp) -> &'v mut Sexp {
        unimplemented!()
    }
}

impl Index for String {
    fn index_into<'v>(&self, v: &'v Sexp) -> Option<&'v Sexp> {
        self[..].index_into(v)
    }
    fn index_into_mut<'v>(&self, v: &'v mut Sexp) -> Option<&'v mut Sexp> {
        self[..].index_into_mut(v)
    }
    fn index_or_insert<'v>(&self, v: &'v mut Sexp) -> &'v mut Sexp {
        self[..].index_or_insert(v)
    }
}

impl<'a, T: ?Sized> Index for &'a T
where
    T: Index,
{
    fn index_into<'v>(&self, v: &'v Sexp) -> Option<&'v Sexp> {
        (**self).index_into(v)
    }
    fn index_into_mut<'v>(&self, v: &'v mut Sexp) -> Option<&'v mut Sexp> {
        (**self).index_into_mut(v)
    }
    fn index_or_insert<'v>(&self, v: &'v mut Sexp) -> &'v mut Sexp {
        (**self).index_or_insert(v)
    }
}

// Prevent users from implementing the Index trait.
mod private {
    pub trait Sealed {}
    impl Sealed for usize {}
    impl Sealed for str {}
    impl Sealed for String {}
    impl<'a, T: ?Sized> Sealed for &'a T
    where
        T: Sealed,
    {
    }
}

/// Used in panic messages.
struct Type<'a>(&'a Sexp);

impl<'a> fmt::Display for Type<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            Sexp::Nil => formatter.write_str("nil"),
            Sexp::Boolean(_) => formatter.write_str("boolean"),
            Sexp::Number(_) => formatter.write_str("number"),
            Sexp::String(_) => formatter.write_str("string"),
            Sexp::Symbol(_) => formatter.write_str("symbol"),
            Sexp::Keyword(_) => formatter.write_str("keyword"),
            Sexp::List(_) => formatter.write_str("list"),
            Sexp::Pair(_, _) => formatter.write_str("pair"),
        }
    }
}

// The usual semantics of Index is to panic on invalid indexing.
//
// That said, the usual semantics are for things like Vec and BTreeMap which
// have different use cases than Sexp. If you are working with a Vec, you know
// that you are working with a Vec and you can get the len of the Vec and make
// sure your indices are within bounds. The Sexp use cases are more
// loosey-goosey. You got some sexprs from an endpoint and you want to pull values
// out of it. Outside of this Index impl, you already have the option of using
// value.as_array() and working with the Vec directly, or matching on
// Sexp::List and getting the Vec directly. The Index impl means you can skip
// that and index directly into the thing using a concise syntax. You don't have
// to check the type, you don't have to check the len, it is all about what you
// expect the Sexp to look like.
//
// Basically the use cases that would be well served by panicking here are
// better served by using one of the other approaches: get and get_mut,
// as_array, or match. The value of this impl is that it adds a way of working
// with Sexp that is not well served by the existing approaches: concise and
// careless and sometimes that is exactly what you want.
impl<I> ops::Index<I> for Sexp
where
    I: Index,
{
    type Output = Sexp;

    /// Index into a `sexpr::Sexp` using the syntax `value[0]` or
    /// `value["k"]`.
    ///
    /// Returns `Sexp::Nil` if the type of `self` does not match the type of
    /// the index, for example if the index is a string and `self` is an array
    /// or a number. Also returns `Sexp::Nil` if the given key does not exist
    /// in the map or the given index is not within the bounds of the array.
    ///
    /// For retrieving deeply nested values, you should have a look at the
    /// `Sexp::pointer` method.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// let data = sexpr::from_str("(x . (y . (z zz)))")
    ///
    /// assert_eq!(data["x"]["y"], sexpr::from_str("(z zz)"));
    /// assert_eq!(data["x"]["y"][0], sexpr::from_str("z"));
    ///
    /// assert_eq!(data["a"], Sexp::Nil); // returns null for undefined values
    /// assert_eq!(data["a"]["b"], Sexp::Nil); // does not panic
    /// # }
    /// ```
    fn index(&self, index: I) -> &Sexp {
        static NIL: Sexp = Sexp::Nil;
        index.index_into(self).unwrap_or(&NIL)
    }
}

impl<I> ops::IndexMut<I> for Sexp
where
    I: Index,
{
    /// Write into a `sexpr::Sexp` using the syntax `value[0] = ...` or
    /// `value["k"] = ...`.
    ///
    /// If the index is a number, the value must be an array of length bigger
    /// than the index. Indexing into a value that is not an array or an array
    /// that is too small will panic.
    ///
    /// If the index is a string, the value must be an object or null which is
    /// treated like an empty object. If the key is not already present in the
    /// object, it will be inserted with a value of null. Indexing into a value
    /// that is neither an object nor null will panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// let mut data = sexp!((x . 0));
    ///
    /// // replace an existing key
    /// data["x"] = sexp!(1);
    ///
    /// // insert a new key
    /// data["y"] = sexp!((#f #f #f));
    ///
    /// // replace an array value
    /// data["y"][0] = sexp!(#f);
    ///
    /// // inserted a deeply nested key
    /// data["a"]["b"]["c"]["d"] = sexp!(#t);
    ///
    /// println!("{}", data);
    /// # }
    /// ```
    fn index_mut(&mut self, index: I) -> &mut Sexp {
        index.index_or_insert(self)
    }
}
