use serde::{Deserialize, Serialize};

use crate::ApiCall;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credentials {
    user_name: String,
    pass_phrase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    two_fa_code: Option<i32>,
}

impl Credentials {
    pub fn new(user_name: String, pass_phrase: String) -> Self {
        Self {
            user_name,
            pass_phrase,
            two_fa_code: None,
        }
    }

    pub fn with_two_fa_code(mut self, two_fa_code: i32) -> Self {
        self.two_fa_code = Some(two_fa_code);
        self
    }
}

impl ApiCall for Credentials {
    type Response = CredentialsResponse;

    const PATH: &'static str = "sessions";

    const METHOD: reqwest::Method = reqwest::Method::POST;
}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CredentialsResponse {
    pub(crate) session_id: String,
}
