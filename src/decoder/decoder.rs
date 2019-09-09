use std::fmt;

#[derive(Debug, Copy, Clone)]
pub struct Decoder<'a> {
    pub bytes: &'a [u8],
    pub offset: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(bytes: &'a [u8]) -> Decoder {
        Decoder { bytes, offset: 0 }
    }

    pub fn eat_byte(&mut self) -> Result<u8, DecoderError> {
        match self.pick_byte() {
            Some(byte) => {
                self.offset += 1;
                Ok(byte)
            }
            None => Err(self.produce_error("Unexpected end of file")),
        }
    }

    pub fn pick_byte(&self) -> Option<u8> {
        if self.offset < self.bytes.len() {
            Some(self.bytes[self.offset])
        } else {
            None
        }
    }

    pub fn match_byte(&mut self, expected: u8) -> bool {
        match self.pick_byte() {
            Some(actual) if actual == expected => {
                self.offset += 1;
                true
            }
            _ => false,
        }
    }

    pub fn produce_error(&self, message: &str) -> DecoderError {
        DecoderError {
            offset: self.offset,
            message: String::from(message),
        }
    }
}

#[derive(Debug)]
pub struct DecoderError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "DecoderError: {} (offset: {})",
            self.message, self.offset
        )
    }
}

pub type DecoderResult<T> = Result<T, DecoderError>;