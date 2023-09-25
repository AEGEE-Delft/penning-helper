use penning_helper_types::Euro;
use textdistance::nstr::damerau_levenshtein;

use crate::matcher::MatchResult;

#[derive(Debug, Clone)]
pub struct TurfList {
    rows: Vec<TurfListRow>,
}

impl TurfList {
    pub fn new(rows: impl IntoIterator<Item = TurfListRow>) -> Self {
        Self {
            rows: rows.into_iter().collect(),
        }
    }

    /// combine duplicate name/email combinations
    pub fn shrink(&mut self) {
        let mut new_rows: Vec<TurfListRow> = Vec::new();
        for row in self.rows.drain(..) {
            if let Some(new_row) = new_rows
                .iter_mut()
                .find(|r| r.name == row.name && r.email == row.email)
            {
                new_row.amount += row.amount;
            } else {
                new_rows.push(row);
            }
        }
        new_rows.retain_mut(|e| e.name != "" && e.name.to_lowercase() != "compass");
        new_rows.sort_by_key(|r| r.amount);
        new_rows.reverse();
        self.rows = new_rows;
    }

    pub fn rows(&self) -> &[TurfListRow] {
        &self.rows
    }

    pub fn iter(&self) -> impl Iterator<Item = &TurfListRow> {
        self.rows.iter()
    }
}

#[derive(Debug, Clone)]
pub struct TurfListRow {
    pub name: String,
    pub email: Option<String>,
    pub amount: Euro,
    pub iban: Option<String>,
}

impl TurfListRow {
    pub fn new(name: String, email: String, amount: Euro, iban: Option<String>) -> Self {
        Self {
            name,
            email: Some(email),
            amount,
            iban,
        }
    }

    pub fn new_no_email(name: String, amount: Euro) -> Self {
        Self {
            name,
            email: None,
            amount,
            iban: None,
        }
    }

    pub fn best_match(&self, options: &[String], match_on: MatchOn) -> MatchResult<usize> {
        let mut best_match = None;
        let mut best_score = f64::INFINITY;
        let target = match match_on {
            MatchOn::Name => &self.name,
            MatchOn::Email => self
                .email
                .as_ref()
                .ok_or(crate::matcher::MatchError::NoMatch)?,
        };
        if target == "" {
            return Err(crate::matcher::MatchError::NoMatch);
        }
        for (i, name) in options.iter().enumerate() {
            let score = damerau_levenshtein(name, target);
            // println!("{}: {} -> {}", name, target, score);
            if score < best_score {
                best_match = Some(i);
                best_score = score;
            }
        }
        // println!("{}: {} -> {}", target, options[best_match.unwrap()], best_score);
        if best_score > 0.3 {
            // println!("No match found for {}", target);
            return Err(crate::matcher::MatchError::NoMatch)
        }
        best_match.ok_or(crate::matcher::MatchError::NoMatch)
    }

    pub fn best_name_match(&self, options: &[String]) -> MatchResult<usize> {
        self.best_match(options, MatchOn::Name)
    }

    pub fn best_email_match(&self, options: &[String]) -> MatchResult<usize> {
        self.best_match(options, MatchOn::Email)
    }

    pub fn best_idx(&self, names: &[String], emails: &[String]) -> Option<(usize, Euro)> {
        let idx = match self.best_email_match(&emails) {
            Ok(idx) => idx,
            Err(_) => match self.best_name_match(&names) {
                Ok(idx) => idx,
                Err(_) => {
                    println!("No match found for {}", self.name);
                    return None;
                }
            },
        };
        Some((idx, self.amount))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MatchOn {
    Name,
    Email,
}
