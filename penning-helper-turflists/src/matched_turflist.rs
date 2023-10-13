use std::ops::Deref;

use crate::turflist::TurfListRow;

#[derive(Debug, Clone)]
pub struct MatchedTurflist {
    rows: Vec<MatchedTurflistRow>,
}

impl MatchedTurflist {
    pub fn new(rows: Vec<MatchedTurflistRow>) -> Self {
        Self { rows }
    }

    pub fn iter(&self) -> impl Iterator<Item = &MatchedTurflistRow> {
        self.rows.iter()
    }
}

#[derive(Debug, Clone)]
pub struct MatchedTurflistRow {
    idx: Option<usize>,
    row: TurfListRow,
}

impl MatchedTurflistRow {
    pub fn new(idx: Option<usize>, row: TurfListRow) -> Self {
        Self { idx, row }
    }

    pub fn idx(&self) -> Option<usize> {
        self.idx
    }

    pub fn row(&self) -> &TurfListRow {
        &self.row
    }
}

impl Deref for MatchedTurflistRow {
    type Target = TurfListRow;

    fn deref(&self) -> &Self::Target {
        &self.row
    }
}
