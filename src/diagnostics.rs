use std::sync::Arc;

use codespan::FileMap;

pub struct DiagnosticBuilder<'a> {
    file_map: Arc<FileMap<&'a str>>,
}