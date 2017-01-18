// Contains the configuration parameters to the parser
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParsePipeBehavior {
    // Accept a base64 encoding of the octet string, e.g (|NFGq/E3wh9f4rJIQVXhS|)
    Base64Interior,
    // Accept everything within two pipes as a valid atom, e.g (|this is an atom with spaces|)
    QuoteInterior,
    // Pipes are treated just like any other atom character.
    None
}

#[derive(Clone, Copy, Debug)]
pub struct ParseConfig {
    // Should semicolons ignore the remainder of the line?
    pub semi_comments: bool,
    // Should atoms be read case-insensitively?
    pub case_sensitive_atoms: bool,
    // Accept '[' and ']' in addition to parenthesis
    pub square_brackets: bool,
    // Pipes can accept a multitude of differing options
    pub pipe_action: ParsePipeBehavior,
    // Escape #NUMBER# to it's appropriate hex decoding.
    pub hex_escapes: bool,
    // Escapes #xNUMBER (hex) and #bNUMBER (binary) to their respective encodings
    pub radix_escape: bool,
    // Accept `:keywords`
    pub colon_keywords: bool
}

/// Configuration for RFC 4648 standard base64 encoding
pub static STANDARD: ParseConfig = ParseConfig {
    semi_comments: true,
    square_brackets: true,
    case_sensitive_atoms: false,
    pipe_action: ParsePipeBehavior::None,
    hex_escapes: true,
    radix_escape: false,
    colon_keywords: true
};
