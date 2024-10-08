use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnNull};

use crate::ApiCall;

#[derive(Debug, Serialize)]
pub struct FieldDefs {
    entity_type: String,
}

impl FieldDefs {
    pub fn new(entity_type: String) -> Self {
        Self { entity_type }
    }
}

impl ApiCall for FieldDefs {
    type Response = FieldDefsResponse;

    const PATH: &'static str = "relations/fieldDefinitions/";

    const METHOD: reqwest::Method = reqwest::Method::GET;

    fn path_params(&self) -> Vec<&str> {
        vec![&self.entity_type]
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct FieldDefsResponse {
    fields: Vec<FieldDef>,
}

impl FieldDefsResponse {
    pub fn fields(&self) -> &[FieldDef] {
        &self.fields
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDef {
    pub field_name: String,
    pub entity_type: String,
    pub label: String,
    pub description: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde_as(as = "DefaultOnNull")]
    pub required: bool,
    pub read_only: bool,
    #[serde(default)]
    pub possible_values: Vec<String>,
    pub shared_field_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    Text,
    Textarea,
    Number,
    Date,
    Amount,
    Checkbox,
    Multicheckbox,
    Enum,
    Mailadres,
    Account,
    File,
    Folder,
}
