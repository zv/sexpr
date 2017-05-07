/// the transports mechanism is intended to provide a universal means of
/// representing S-expressions for transport from one machine to another.
/// sexpr supports the two most common transport representations: 'Canonical'
/// and 'Base64'

/// # Canonical
/// This representation is primarily used for digital signature transmissions.
/// It is uniquely defined for each S-expression. It is not intended to be
/// human-readable, but is very easy to parse, to be reasonably economical, and
/// to be unique for any S-expression.
///
/// The "canonical" form of an S-expression represents each octet-string in a
/// length-prefixed verbatim mode, and represents each list with no blanks
/// separating elements from each other or from the surrounding parentheses.
///
/// Here are some examples of canonical representations of S-expressions:
///
/// 	`(1:a1:b1:c)`
///   `(6:issuer3:bob)`
/// 	`(4:icon[12:image/bitmap]9:xxxxxxxxx)`
/// 	`(7:subject(3:ref5:alice6:mother))`

/// ## Base64
/// The Base64 representation is simply a RFC-2045 encoded variant of the
/// canonical representation, surrounded in braces.
/// Here's an example:
///
/// 	`{KDE6YTE6YjE6YykA}` (this is the earlier `(1:a1:b1:c)` encoded in base-64)
///
/// There is a difference between the brace notation for base-64 used here and
/// the || notation for base-64'd octet-strings described in `Config`. Here the
/// base-64 contents are converted to octets, and then re-scanned as if they
/// were given originally as octets. With the || notation, the contents are just
/// turned into an octet-string.

/// This trait is responsible for transforming an encoding (base64, 'canonical')
/// into a stream of tokens that can be ordinarily decoded.
trait SexpTransport {
    fn decode(&self, stream: &str) -> String;
    fn encoder(&self, sexp: Sexp) -> String;
}

struct Canonical;

impl SexpEncoding for Canonical {
    fn decode(&self, stream: &str) -> String {
        String::new()
    }

    fn encoder(&self, sexp: Sexp) -> String {
        String::new()
    }
}

