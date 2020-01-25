use std::str::FromStr;
use std::fmt;

use librsvg::{Length, LengthUnit};

use lexical::ErrorCode;

// TODO: actually use this everywhere
#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub value: f64,
    pub unit: SizeUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeUnit {
    Px,
    Em,
    Ex,
    In,
    Cm,
    Mm,
    Pt,
    Pc,
    Percent,
}

pub enum ParseSizeError {
    ParseError(ErrorCode),
    UnknownSuffix(String),
}

impl From<lexical::Error> for ParseSizeError {
    fn from(err: lexical::Error) -> Self {
        ParseSizeError::ParseError(err.code)
    }
}

impl fmt::Display for ParseSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseSizeError::ParseError(code) => write!(f, "parse error: {:?}", code),
            ParseSizeError::UnknownSuffix(suffix) => write!(f, "unknown suffix: {:?}", suffix),
        }
    }
}

impl Size {
    /// Converts this size to the pixel size, scaling it depending on the size and given ppi, pointsize,
    /// or according to total_size (if it's a percentage).
    #[allow(unused)]
    pub fn to_f64(&self, ppi: f64, pointsize: f64, total_size: f64) -> f64 {
        match self.to_f64_opt(ppi, pointsize) {
            Some(val) => val,
            None => {
                assert_eq!(self.unit, SizeUnit::Percent);
                total_size * self.value / 100.0
            }
        }
    }

    /// Converts this size to the pixel size, scaling it depending on the size and given ppi and pointsize.
    ///
    /// Returns `None` if the unit is `Percent`.
    /// This can be used when there isn't a total_size.
    pub fn to_f64_opt(&self, ppi: f64, pointsize: f64) -> Option<f64> {
        // https://github.com/ImageMagick/ImageMagick/blob/55939508d807026de24b6668545a65e1f44d9933/coders/svg.c#L428-L443
        Some(match self.unit {
            SizeUnit::Cm => ppi / 2.54 * self.value,
            SizeUnit::Em => pointsize * self.value,
            SizeUnit::Ex => pointsize * self.value / 2.0,
            SizeUnit::In => ppi * self.value,
            SizeUnit::Mm => ppi / 25.4 * self.value,
            SizeUnit::Pc => ppi / 6.0 * self.value,
            SizeUnit::Pt => self.value,
            SizeUnit::Px => self.value,
            SizeUnit::Percent => return None,
        })
    }
}

impl FromStr for Size {
    type Err = ParseSizeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (value, consumed) = lexical::parse_partial(s)?;
        let suffix = &s[consumed..];
        let unit = match suffix.trim_start() {
            "px" => SizeUnit::Px,
            "em" => SizeUnit::Em,
            "ex" => SizeUnit::Ex,
            "in" => SizeUnit::In,
            "cm" => SizeUnit::Cm,
            "mm" => SizeUnit::Mm,
            "pt" => SizeUnit::Pt,
            "pc" => SizeUnit::Pc,
            "%" => SizeUnit::Percent,
            _ => return Err(ParseSizeError::UnknownSuffix(suffix.to_string())),
        };
        Ok(Size {
            value,
            unit,
        })
    }
}

impl From<Length> for Size {
    fn from(len: Length) -> Self {
        let Length { length, unit } = len;
        let unit = match unit {
            LengthUnit::Px => SizeUnit::Px,
            LengthUnit::Em => SizeUnit::Em,
            LengthUnit::Ex => SizeUnit::Ex,
            LengthUnit::In => SizeUnit::In,
            LengthUnit::Cm => SizeUnit::Cm,
            LengthUnit::Mm => SizeUnit::Mm,
            LengthUnit::Pt => SizeUnit::Pt,
            LengthUnit::Pc => SizeUnit::Pc,
            LengthUnit::Percent => SizeUnit::Percent,
        };
        Size {
            value: length,
            unit,
        }
    }
}
