#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum GitRefScope {
    Local,
    Remote,
}

pub(super) struct GitStatusHeader {
    pub(super) branch: String,
    pub(super) details: Vec<String>,
}

pub(super) struct GitStatusRow {
    pub(super) path: String,
    pub(super) original_path: Option<String>,
    pub(super) badges: Vec<String>,
    pub(super) style: &'static str,
}

pub(super) struct GitBranchRow {
    pub(super) scope: GitRefScope,
    pub(super) name: String,
    pub(super) current: bool,
    pub(super) upstream: Option<String>,
    pub(super) commit: String,
    pub(super) date: String,
    pub(super) subject: String,
}

pub(super) struct GitLogRow {
    pub(super) commit: String,
    pub(super) date: String,
    pub(super) author: String,
    pub(super) refs: Vec<String>,
    pub(super) subject: String,
}
