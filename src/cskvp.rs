use std::collections::HashMap;
use std::borrow::Cow;
use std::mem;

use single::{self, Single};
use quoted_string::test_utils::TestSpec;

use crate::ext::{VecExt, CowExt};

#[derive(Debug, Default)]
pub struct Cskvp<'a> {
    label: Option<Cow<'a, str>>,
    caption: Option<Cow<'a, str>>,
    figure: Option<bool>,
    single: Vec<Cow<'a, str>>,
    double: HashMap<Cow<'a, str>, Cow<'a, str>>,
}

impl<'a> Cskvp<'a> {
    pub fn new(s: Cow<'a, str>) -> Cskvp<'a> {
        let mut parser = Parser::new(s);

        let mut single = Vec::new();
        let mut double = HashMap::new();
        let mut label = None;
        for value in parser {
            match value {
                Value::Double(key, value) => {
                    double.insert(key, value);
                },
                Value::Single(mut value) => if value.starts_with("#") {
                    if label.is_some() {
                        // TODO: warn
                        println!("Found two labels, taking the last: {} and {}",
                                 label.as_ref().unwrap(), &value[1..]);
                    }
                    value.truncate_left(1);
                    label = Some(value);
                } else {
                    single.push(value);
                }
            }
        }

        let figure_double = double.remove("figure")
            .and_then(|s| match s.parse() {
                Ok(val) => Some(val),
                Err(_) => {
                    // TODO: warn
                    println!("cannot parse `figure={}`, only `true` and `false` are allowed", s);
                    None
                }
            });
        let figure = single.remove_element(&"figure").map(|_| true);
        let nofigure = single.remove_element(&"nofigure").map(|_| false);

        let figure = match [figure_double, figure, nofigure].into_iter().cloned().flatten().single() {
            Ok(val) => Some(val),
            Err(single::Error::NoElements) => None,
            Err(single::Error::MultipleElements) => {
                // TODO: warn
                println!("only one of `figure=true`, `figure=false`, `figure` and `nofigure` \
                    allowed, found multiple");
                None
            }
        };

        Cskvp {
            label,
            caption: double.remove("caption"),
            figure,
            single,
            double,
        }
    }

    pub fn has_label(&self) -> bool {
        self.label.is_some()
    }

    pub fn take_label(&mut self) -> Option<Cow<'a, str>> {
        self.label.take()
    }

    pub fn take_figure(&mut self) -> Option<bool> {
        self.figure.take()
    }

    pub fn take_caption(&mut self) -> Option<Cow<'a, str>> {
        self.caption.take()
    }

    pub fn take_single(&mut self, attr: &str) -> Option<Cow<'a, str>> {
        self.single.remove_element(&attr)
    }

    pub fn take_single_by_index(&mut self, index: usize) -> Option<Cow<'a, str>> {
        if index >= self.single.len() {
            return None;
        }
        Some(self.single.remove(index))
    }

    pub fn take_double(&mut self, key: &str) -> Option<Cow<'a, str>> {
        self.double.remove(key)
    }

    /// Removes all elements from `self`.
    ///
    /// This can be used before dropping `Cskvp` to omit all "unused attribute" warnings.
    pub fn clear(&mut self) {
        let _ = self.label.take();
        let _ = self.figure.take();
        let _ = self.caption.take();
        self.single.clear();
        self.double.clear();
    }
}

impl<'a> Drop for Cskvp<'a> {
    fn drop(&mut self) {
        // TODO: warn
        if let Some(label) = self.label.as_ref() {
            println!("label ignored: {}", label);
        }
        if let Some(figure) = self.figure {
            println!("figure config ignored: {}", figure);
        }
        if let Some(caption) = self.caption.as_ref() {
            println!("caption ignored: {}", caption);
        }
        for (k, v) in self.double.drain() {
            println!("Unknown attribute `{}={}`", k, v);
        }
        for attr in self.single.drain(..) {
            println!("Unknown attribute `{}`", attr);
        }
    }
}

struct Parser<'a> {
    rest: Cow<'a, str>,
}

#[derive(Debug, PartialEq, Eq)]
enum Value<'a> {
    Double(Cow<'a, str>, Cow<'a, str>),
    Single(Cow<'a, str>),
}

impl<'a> Parser<'a> {
    fn new(s: Cow<'a, str>) -> Parser<'a> {
        Parser {
            rest: s,
        }
    }

    fn next_quoted(&mut self) -> Cow<'a, str> {
        assert!(self.rest.trim_start().starts_with('"'));

        match &self.rest {
            Cow::Borrowed(rest) => match quoted_string::parse::<TestSpec>(rest) {
                // TODO: error handling
                Err(e) => panic!("Invalid double-quoted string: {:?}", e),
                Ok(parsed) => {
                    self.rest = Cow::Borrowed(parsed.tail);
                    quoted_string::to_content::<TestSpec>(parsed.quoted_string).unwrap()
                }
            }
            Cow::Owned(rest) => match quoted_string::parse::<TestSpec>(rest) {
                // TODO: error handling
                Err(e) => panic!("Invalid double-quoted string: {:?}", e),
                Ok(parsed) => {
                    let content = quoted_string::to_content::<TestSpec>(parsed.quoted_string)
                        .unwrap().into_owned();
                    self.rest.truncate_left(self.rest.len() - parsed.tail.len());
                    Cow::Owned(content)
                }
            }
        }
    }

    fn next_unquoted(&mut self, delimiters: &[char]) -> Cow<'a, str> {
        let idx = delimiters.iter().cloned()
            .filter_map(|delim| self.rest.find(delim))
            .min()
            .unwrap_or(self.rest.len());
        let mut rest = self.rest.split_off(idx);
        let mut val = mem::replace(&mut self.rest, rest);
        val.trim_inplace();
        val
    }

    fn next_single(&mut self, delimiters: &[char]) -> Cow<'a, str> {
        self.rest.trim_left_inplace();
        if self.rest.starts_with('"') {
            self.next_quoted()
        } else {
            self.next_unquoted(delimiters)
        }
    }

    fn skip_delimiter(&mut self) -> Option<char> {
        self.rest.trim_left_inplace();
        let delim = self.rest.chars().next();
        if delim.is_some() {
            self.rest.truncate_left(1);
        }
        delim

    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Value<'a>;

    fn next(&mut self) -> Option<Value<'a>> {
        if self.rest.is_empty() {
            return None;
        }

        let mut key = self.next_single(&['=', ',']);
        key.trim_inplace();

        let delim = self.skip_delimiter();

        if let Some('=') = delim {
            let res = Some(Value::Double(key, self.next_single(&[','])));

            let delim = self.skip_delimiter();
            assert!(delim == Some(',') || delim == None, "invalid comma seperated key value pair");
            res
        } else {
            // TODO: error handling
            assert!(delim == Some(',') || delim == None, "invalid comma seperated value");
            Some(Value::Single(key))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parser() {
        let mut parser = Parser::new(Cow::Borrowed(r#"foo, bar = " baz, \"qux\"", quux"#));
        assert_eq!(parser.next(), Some(Value::Single(Cow::Borrowed("foo"))));
        assert_eq!(parser.next(), Some(Value::Double(Cow::Borrowed("bar"), Cow::Owned(r#" baz, "qux""#.to_string()))));
        assert_eq!(parser.next(), Some(Value::Single(Cow::Borrowed("quux"))));
    }

    #[test]
    fn test_quoted_string() {
        let parsed = quoted_string::parse::<TestSpec>(r#""foo = \"bar\"" tail"#).unwrap();
        assert_eq!(parsed.quoted_string, r#""foo = \"bar\"""#);
        assert_eq!(parsed.tail, " tail");
        let content = quoted_string::to_content::<TestSpec>(parsed.quoted_string).unwrap();
        let expected: Cow<'_, str> = Cow::Owned(r#"foo = "bar""#.to_string());
        assert_eq!(content, expected);
        let content = quoted_string::to_content::<TestSpec>(r#""foo = bar""#).unwrap();
        assert_eq!(content, Cow::Borrowed("foo = bar"));
    }
}
