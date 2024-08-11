use serde::{Deserialize, Serialize};

use crate::ApiCall;

#[derive(Debug, Default, Serialize)]
pub struct EntityTypes {}

impl ApiCall for EntityTypes {
    type Response = EntityTypesResponse;

    const PATH: &'static str = "relations/entityTypes";

    const METHOD: reqwest::Method = reqwest::Method::GET;
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EntityTypesResponse {
    entity_types: Vec<EntityType>,
}

impl EntityTypesResponse {
    pub fn entity_types(&self) -> &[EntityType] {
        &self.entity_types
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityType {
    pub type_name: String,
    pub lang_determiner: String,
    pub lang_singular: String,
    pub lang_plural: String,
}
