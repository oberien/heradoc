use std::collections::HashMap;

pub struct Cskvp<'a> {
    label: Option<&'a str>,
    single: Vec<&'a str>,
    double: HashMap<&'a str, &'a str>,
}

impl<'a> Cskvp<'a> {
    pub fn new(s: &'a str) -> Cskvp<'a> {
        let mut single = Vec::new();
        let mut double = HashMap::new();
        let mut label = None;
        for part in s.split(',') {
            let part = part.trim();
            if part.contains("=") {
                let i = part.find('=').unwrap();
                double.insert(&part[..i], &part[(i+1)..]);
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
        Cskvp {
            label,
            single,
            double,
        }
    }

    pub fn label(&mut self) -> Option<&'a str> {
        self.label.take()
    }

    pub fn single(&mut self, attr: &str) -> Option<&'a str> {
        let pos = self.single.iter().position(|&s| attr == s)?;
        Some(self.single.remove(pos))
    }

    pub fn single_remove(&mut self, index: usize) -> Option<&'a str> {
        if index >= self.single.len() {
            return None;
        }
        Some(self.single.remove(index))
    }

    pub fn double(&mut self, key: &str) -> Option<&'a str> {
        self.double.remove(key)
    }
}

impl<'a> Drop for Cskvp<'a> {
    fn drop(&mut self) {
        for (k, v) in self.double.drain() {
            // TODO: warn
            println!("Unknown attribute `{}={}`", k, v);
        }
        for attr in self.single.drain(..) {
            // TODO: warn
            println!("Unknown attribute `{}`", attr);
        }
    }
}
