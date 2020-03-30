mod article;
mod beamer;
mod report;
mod thesis;

pub use self::article::Article;
pub use self::beamer::{Beamer, FrameEvent as BeamerFrameEvent};
pub use self::report::Report;
pub use self::thesis::Thesis;
