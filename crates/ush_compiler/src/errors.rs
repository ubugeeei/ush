use crate::types::{AstString as String, HeapVec as Vec};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ErrorType {
    Known(String),
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ErrorSet {
    items: Vec<ErrorType>,
}

impl ErrorSet {
    pub(crate) fn insert(&mut self, error: ErrorType) {
        if self.items.iter().any(|item| item == &error) {
            return;
        }
        self.items.push(error);
        self.items.sort();
    }

    pub(crate) fn extend(&mut self, other: &Self) {
        for error in &other.items {
            self.insert(error.clone());
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub(crate) fn render(&self) -> String {
        self.items
            .iter()
            .map(ErrorType::render)
            .collect::<Vec<_>>()
            .join(" | ")
            .into()
    }
}

impl ErrorType {
    fn render(&self) -> String {
        match self {
            Self::Known(name) => name.clone(),
            Self::Unknown => "unknown".into(),
        }
    }
}
