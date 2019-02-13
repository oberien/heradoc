use std::collections::HashMap;
use std::borrow::Cow;

use single::{self, Single};

use crate::ext::VecExt;

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
        // TODO: allow double quoted strings and spaces after comma: `foo,bar="baz, qux", quux`
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
