use std::str::FromStr;
use std::fmt;
use std::collections::HashMap;

use serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess};
use structopt::StructOpt;

use config::MaybeUnknown;

// https://www.sharelatex.com/learn/Page_size_and_margins#Fine_tuning_your_LaTeX_page_dimensions
#[derive(Debug, Clone, Default, Deserialize, StructOpt)]
pub struct Geometry {
    #[structopt(long = "papersize")]
    pub papersize: Option<MaybeUnknown<Papersize>>,
    #[structopt(long = "orientation")]
    pub orientation: Option<MaybeUnknown<Orientation>>,
    #[structopt(long = "margin")]
    pub margin: Option<String>,
    #[structopt(long = "textwidth")]
    pub textwidth: Option<String>,
    #[structopt(long = "textheight")]
    pub textheight: Option<String>,
    #[structopt(long = "total")]
    pub total: Option<String>,
    #[structopt(long = "left")]
    pub left: Option<String>,
    #[structopt(long = "lmargin")]
    pub lmargin: Option<String>,
    #[structopt(long = "inner")]
    pub inner: Option<String>,
    #[structopt(long = "right")]
    pub right: Option<String>,
    #[structopt(long = "rmargin")]
    pub rmargin: Option<String>,
    #[structopt(long = "outer")]
    pub outer: Option<String>,
    #[structopt(long = "top")]
    pub top: Option<String>,
    #[structopt(long = "tmargin")]
    pub tmargin: Option<String>,
    #[structopt(long = "bottom")]
    pub bottom: Option<String>,
    #[structopt(long = "bmargin")]
    pub bmargin: Option<String>,
    #[structopt(long = "headheight")]
    pub headheight: Option<String>,
    #[structopt(long = "footsep")]
    pub footsep: Option<String>,
    #[structopt(long = "footskip")]
    pub footskip: Option<String>,
    #[structopt(long = "marginparwidth")]
    pub marginparwidth: Option<String>,
    #[structopt(long = "marginpar")]
    pub marginpar: Option<String>,
    #[serde(flatten)]
    #[structopt(long = "others", raw(hidden = "true"), parse(try_from_str = "parse_others"))]
    pub others: Option<HashMap<String, String>>,
}

pub fn parse_others(s: &str) -> Result<HashMap<String, String>, &'static str> {
    let errmsg = "Expected `key=value,key2=value2` format for others";
    if !s.contains("=") {
        return Err(errmsg);
    }

    s.split(',').map(|pair| {
        let mut iter = pair.split("=");
        Ok((iter.next().ok_or(errmsg)?.to_string(), iter.next().ok_or(errmsg)?.to_string()))
    }).collect()
}

impl Geometry {
    pub fn merge(self, g: Geometry) -> Geometry {
        let others = match (self.others, g.others) {
            (Some(mut s), Some(o)) => {
                s.extend(o);
                Some(s)
            }
            (Some(s), None) => Some(s),
            (None, Some(o)) => Some(o),
            (None, None) => None
        };
        Geometry {
            papersize: self.papersize.or(g.papersize),
            orientation: self.orientation.or(g.orientation),
            margin: self.margin.or(g.margin),
            textwidth: self.textwidth.or(g.textwidth),
            textheight: self.textheight.or(g.textheight),
            total: self.total.or(g.total),
            left: self.left.or(g.left),
            lmargin: self.lmargin.or(g.lmargin),
            inner: self.inner.or(g.inner),
            right: self.right.or(g.right),
            rmargin: self.rmargin.or(g.rmargin),
            outer: self.outer.or(g.outer),
            top: self.top.or(g.top),
            tmargin: self.tmargin.or(g.tmargin),
            bottom: self.bottom.or(g.bottom),
            bmargin: self.bmargin.or(g.bmargin),
            headheight: self.headheight.or(g.headheight),
            footsep: self.footsep.or(g.footsep),
            footskip: self.footskip.or(g.footskip),
            marginparwidth: self.marginparwidth.or(g.marginparwidth),
            marginpar: self.marginpar.or(g.marginpar),
            others,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Papersize {
    A0paper, A1paper, A2paper, A3paper, A4paper, A5paper, A6paper,
    B0paper, B1paper, B2paper, B3paper, B4paper, B5paper, B6paper,
    C0paper, C1paper, C2paper, C3paper, C4paper, C5paper, C6paper,
    B0j, B1j, B2j, B3j, B4j, B5j, B6j,
    AnsiAPaper, AnsiBPaper, AnsiCPaper, AnsiDPaper, AnsiEPaper,
    Letterpaper, Executivepaper, Legalpaper,
    // `papersize = "{30cm, 15cm}"`
    Custom(String, String)
}

impl FromStr for Papersize {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        Ok(match s.as_str() {
            "a0" | "a0paper" => Papersize::A0paper,
            "a1" | "a1paper" => Papersize::A1paper,
            "a2" | "a2paper" => Papersize::A2paper,
            "a3" | "a3paper" => Papersize::A3paper,
            "a4" | "a4paper" => Papersize::A4paper,
            "a5" | "a5paper" => Papersize::A5paper,
            "a6" | "a6paper" => Papersize::A6paper,
            "b0" | "b0paper" => Papersize::A0paper,
            "b1" | "b1paper" => Papersize::B1paper,
            "b2" | "b2paper" => Papersize::B2paper,
            "b3" | "b3paper" => Papersize::B3paper,
            "b4" | "b4paper" => Papersize::B4paper,
            "b5" | "b5paper" => Papersize::B5paper,
            "b6" | "b6paper" => Papersize::B6paper,
            "c0" | "c0paper" => Papersize::C0paper,
            "c1" | "c1paper" => Papersize::C1paper,
            "c2" | "c2paper" => Papersize::C2paper,
            "c3" | "c3paper" => Papersize::C3paper,
            "c4" | "c4paper" => Papersize::C4paper,
            "c5" | "c5paper" => Papersize::C5paper,
            "c6" | "c6paper" => Papersize::C6paper,
            "b0j" => Papersize::B0j,
            "b1j" => Papersize::B1j,
            "b2j" => Papersize::B2j,
            "b3j" => Papersize::B3j,
            "b4j" => Papersize::B4j,
            "b5j" => Papersize::B5j,
            "b6j" => Papersize::B6j,
            "ansia" | "ansiapaper" => Papersize::AnsiAPaper,
            "ansib" | "ansibpaper" => Papersize::AnsiBPaper,
            "ansic" | "ansicpaper" => Papersize::AnsiCPaper,
            "ansid" | "ansidpaper" => Papersize::AnsiDPaper,
            "ansie" | "ansiepaper" => Papersize::AnsiEPaper,
            "letter" | "letterpaper" => Papersize::Letterpaper,
            "executive" | "executivepaper" => Papersize::Executivepaper,
            "legal" | "legalpaper" => Papersize::Legalpaper,
            s if s.starts_with("{") && s.ends_with("}") && s.contains(",") => {
                let comma = s.find(',').unwrap();
                let width = s[1..comma].to_string();
                let height = s[(comma + 1)..s.len() - 1].trim().to_string();
                Papersize::Custom(width, height)
            }
            _ => return Err(format!("unknown papersize {:?}", s))
        })
    }
}

impl<'de> Deserialize<'de> for Papersize {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl Default for Papersize {
    fn default() -> Self {
        Papersize::A4paper
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Portrait,
    Landscape,
}

impl<'de> Deserialize<'de> for Orientation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl FromStr for Orientation {
    type Err = String;

    fn from_str(s: &str) -> Result<Orientation, Self::Err> {
        if s.eq_ignore_ascii_case("portrait") {
            Ok(Orientation::Portrait)
        } else if s.eq_ignore_ascii_case("landscape") {
            Ok(Orientation::Landscape)
        } else {
            Err(format!("unknown orientation {:?}", s))
        }
    }
}
