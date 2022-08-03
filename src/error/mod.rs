#[derive(Debug)]
pub struct RegexError;
impl std::error::Error for RegexError {}
impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "text does not match regex")
    }
}

#[derive(Debug)]
pub struct RegexCapturesError;
impl std::error::Error for RegexCapturesError {}
impl std::fmt::Display for RegexCapturesError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "text does not match regex")
    }
}

#[derive(Debug)]
pub struct ParseError;
impl std::error::Error for ParseError {}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "decimal string cannot be parse to target numeric type")
    }
}
