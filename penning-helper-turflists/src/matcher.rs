pub enum MatchError {
    NoMatch,
    NoEmail,
}

pub type MatchResult<T> = Result<T, MatchError>;
