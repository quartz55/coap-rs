use arrayvec::ArrayVec;
use std::fmt;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
pub struct Token(ArrayVec<[u8; 8]>);

impl Token {
    pub fn new(bytes: &[u8]) -> Self {
        Self(ArrayVec::from_iter(bytes.iter().cloned()))
    }

    pub fn empty() -> Self {
        Self(ArrayVec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn to_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<[u8]> for Token {
    fn eq(&self, other: &[u8]) -> bool {
        self.0 == *other
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.len() == 0 {
            write!(f, "NULL")
        } else {
            write!(f, "0x")?;
            for byte in &self.0 {
                write!(f, "{:x}", byte)?;
            }
            Ok(())
        }
    }
}
