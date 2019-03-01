use std::collections::HashMap;
use std::borrow::Cow;

use single::{self, Single};
use quoted_string::test_utils::TestSpec;

use crate::ext::{VecExt, CowExt};

#[derive(Debug, Default)]
pub struct Cskvp<'a> {
    label: Option<&'a str>,
    caption: Option<&'a str>,
    figure: Option<bool>,
    single: Vec<&'a str>,
    double: HashMap<&'a str, &'a str>,
}

impl<'a> Cskvp<'a> {
    pub fn new(s: &'a str) -> Cskvp<'a> {
        let mut parser = Parser::new(s);

        let mut single = Vec::new();
        let mut double = HashMap::new();
        let mut label = None;
        for part in s.split(',') {
            let part = part.trim();
            if part.contains("=") {
                let i = part.find('=').unwrap();
                let key = &part[..i];
                let value = &part[(i+1)..];
                double.insert(key, value);
            } else {
                if part.starts_with("#") {
                    if label.is_some() {
                        // TODO: warn
                        println!("Found two labels, taking the last: {} and {}",
                                 label.as_ref().unwrap(), &part[1..]);
                    }
                    label = Some(&part[1..]);
                } else {
                    single.push(part);
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
        self.label.take().map(Cow::Borrowed)
    }

    pub fn take_figure(&mut self) -> Option<bool> {
        self.figure.take()
    }

    pub fn take_caption(&mut self) -> Option<Cow<'a, str>> {
        self.caption.take().map(Cow::Borrowed)
    }

    pub fn take_single(&mut self, attr: &str) -> Option<Cow<'a, str>> {
        self.single.remove_element(&attr)
            .map(Cow::Borrowed)
    }

    pub fn take_single_by_index(&mut self, index: usize) -> Option<Cow<'a, str>> {
        if index >= self.single.len() {
            return None;
        }
        Some(Cow::Borrowed(self.single.remove(index)))
    }

    pub fn take_double(&mut self, key: &str) -> Option<Cow<'a, str>> {
        self.double.remove(key)
            .map(Cow::Borrowed)
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
        if let Some(label) = self.label {
            println!("label ignored: {}", label);
        }
        if let Some(figure) = self.figure {
            println!("figure config ignored: {}", figure);
        }
        if let Some(caption) = self.caption {
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
    rest: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
enum Value<'a> {
    KeyValue(Cow<'a, str>, Cow<'a, str>),
    Value(Cow<'a, str>),
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Parser<'a> {
        Parser {
            rest: s,
        }
    }

    fn next_quoted(&mut self) -> Cow<'a, str> {
        assert!(self.rest.trim_start().starts_with('"'));

        match quoted_string::parse::<TestSpec>(self.rest) {
            // TODO: error handling
            Err(e) => panic!("Invalid double-quoted string: {:?}", e),
            Ok(parsed) => {
                self.rest = parsed.tail;
                quoted_string::to_content::<TestSpec>(parsed.quoted_string).unwrap()
            }
        }
    }

    fn next_unquoted(&mut self, delimiters: &[char]) -> &'a str {
        let idx = delimiters.iter().cloned()
            .filter_map(|delim| self.rest.find(delim))
            .min()
            .unwrap_or(self.rest.len());
        let val = &self.rest[..idx];
        self.rest = &self.rest[idx..];
        val
    }

    fn next_single(&mut self, delimiters: &[char]) -> Cow<'a, str> {
        self.rest = self.rest.trim_start();
        let mut res = if self.rest.starts_with('"') {
            self.next_quoted()
        } else {
            Cow::Borrowed(self.next_unquoted(delimiters).trim())
        };
        res
    }

    fn skip_delimiter(&mut self) -> Option<char> {
        self.rest = self.rest.trim_start();
        let delim = self.rest.chars().next();
        if delim.is_some() {
            self.rest = &self.rest[1..];
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
            let res = Some(Value::KeyValue(key, self.next_single(&[','])));

            let delim = self.skip_delimiter();
            assert!(delim == Some(',') || delim == None, "invalid comma seperated key value pair");
            res
        } else {
            // TODO: error handling
            assert!(delim == Some(',') || delim == None, "invalid comma seperated value");
            Some(Value::Value(key))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parser() {
        let mut parser = Parser::new(r#"foo, bar = " baz, \"qux\"", quux"#);
        assert_eq!(parser.next(), Some(Value::Value(Cow::Borrowed("foo"))));
        assert_eq!(parser.next(), Some(Value::KeyValue(Cow::Borrowed("bar"), Cow::Owned(r#" baz, "qux""#.to_string()))));
        assert_eq!(parser.next(), Some(Value::Value(Cow::Borrowed("quux"))));
    }
}
