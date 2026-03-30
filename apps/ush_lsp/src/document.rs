use std::collections::BTreeMap;

use anyhow::{Result, anyhow};
use lsp_types::Uri;

#[derive(Default)]
pub struct DocumentStore {
    docs: BTreeMap<Uri, String>,
}

impl DocumentStore {
    pub fn open(&mut self, uri: Uri, text: String) {
        self.docs.insert(uri, text);
    }

    pub fn update(&mut self, uri: &Uri, text: String) {
        self.docs.insert(uri.clone(), text);
    }

    pub fn read(&self, uri: &Uri) -> Result<String> {
        if let Some(text) = self.docs.get(uri) {
            return Ok(text.clone());
        }
        Err(anyhow!("document not open: {}", uri.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::DocumentStore;

    #[test]
    fn open_then_read_returns_buffered_text() {
        let uri = lsp_types::Uri::from_str("file:///tmp/test.ush").expect("uri");
        let mut docs = DocumentStore::default();
        docs.open(uri.clone(), "print \"ok\"".to_string());

        assert_eq!(docs.read(&uri).expect("read"), "print \"ok\"");
    }
}
