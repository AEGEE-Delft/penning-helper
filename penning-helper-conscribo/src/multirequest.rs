use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    response::{ApiResponse, ResponseMessages},
    ApiCall,
};

#[derive(Serialize)]
pub struct MultiRequest {
    requests: Vec<MultiElement>,
}

impl MultiRequest {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    pub fn add<R>(&mut self, seq: impl ToString, content: R)
    where
        R: Into<MultiRequestElement> + ApiCall,
    {
        self.requests
            .push(MultiElement::new(seq.to_string(), content));
    }

    pub fn push<R>(mut self, seq: impl ToString, content: R) -> Self
    where
        R: Into<MultiRequestElement> + ApiCall,
    {
        self.add(seq, content);
        self
    }

    pub fn push_all<R>(mut self, elements: Vec<(impl ToString, R)>) -> Self
    where
        R: Into<MultiRequestElement> + ApiCall,
    {
        for (seq, content) in elements {
            self.add(seq, content);
        }
        self
    }
}

impl ApiCall for MultiRequest {
    type Response = MultiRequestResponse;

    const PATH: &'static str = "multirequest";

    const METHOD: reqwest::Method = reqwest::Method::POST;

    fn call(
        &self,
        client: &crate::ConscriboClient,
    ) -> Result<crate::response::ApiResponse<Self::Response>, crate::RequestError> {
        let url = format!(
            "https://api.secure.conscribo.nl/{}/{}",
            client.account_name,
            Self::PATH
        );
        // let mut request = client.client.post(&url).header("X-Conscribo-API-Version", VERSION);
        let mut request = client
            .client
            .request(Self::METHOD, &url)
            .header("X-Conscribo-API-Version", super::VERSION);
        if let Some(session_id) = client.session_id.read().unwrap().as_ref() {
            request = request.header("X-Conscribo-SessionId", session_id);
        }
        // let response = request.json(self).send().unwrap();
        if Self::METHOD == reqwest::Method::GET {
            request = request.query(self);
        } else {
            request = request.json(self);
        }
        let response = request.send()?;
        // let response = response.json::<ApiResponse<Self::Response>>()?;
        let response_text = response.text()?;
        // println!("{}", response_text);
        let response = serde_json::from_str::<MultiRequestResponse>(&response_text)?;

        Ok(ApiResponse::new(response))
    }
}

#[derive(Serialize)]
struct MultiElement {
    #[serde(rename = "Request.sequence")]
    seq: String,
    #[serde(rename = "Request.httpMethod")]
    method: String,
    #[serde(rename = "Request.url")]
    url: String,
    #[serde(rename = "Request.queryParams")]
    query_params: String,
    #[serde(flatten)]
    content: MultiRequestElement,
}

impl MultiElement {
    fn new<R>(seq: String, content: R) -> Self
    where
        R: Into<MultiRequestElement> + ApiCall,
    {
        Self {
            seq,
            method: R::METHOD.to_string(),
            url: R::PATH.to_string(),
            query_params: String::new(),
            content: content.into(),
        }
    }
}

macro_rules! multi_request_elements {
    ($($typ:ty => $name:ident => $as:ident),+ $(,)?) => {
        #[derive(Serialize)]
        #[serde(untagged)]
        pub enum MultiRequestElement {
            $(
                $name($typ),
            )+

        }
        $(
            impl From<$typ> for MultiRequestElement {
                fn from(it: $typ) -> Self {
                    Self::$name(it)
                }
            }
        )+
        #[derive(Deserialize, Default, Debug)]
        #[serde(untagged)]
        pub enum MultiRequestElementResponse {
            #[default]
            Empty,
            $(
                $name(<$typ as ApiCall>::Response),
            )+
        }

        impl MultiRequestElementResponse{
            $(
                pub fn $as(&self) -> Option<&<$typ as ApiCall>::Response> {
                    match self {
                        Self::$name(it) => Some(it),
                        _ => None,
                    }
                }
            )+
        }
    };
}

multi_request_elements!(
    super::session::Credentials => Credentials => as_credentials,
    super::entity_types::EntityTypes => EntityTypes => as_entity_types,
    super::accounts::AccountRequest => AccountRequest => as_account_request,
    super::entities::Entities => EntityRequest => as_entity_request,
    super::add_transaction::AddTransaction => AddTransaction => as_add_transaction,
);

#[derive(Deserialize, Default)]
pub struct MultiRequestResponse {
    responses: MRT,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum MRT {
    Vec(Vec<Element>),
    HashMap(HashMap<String, Element>),
}

impl Default for MRT {
    fn default() -> Self {
        Self::Vec(Vec::new())
    }
}

impl MultiRequestResponse {
    pub fn from_json(s: &str) -> ApiResponse<Self> {
        ApiResponse::new(serde_json::from_str(s).unwrap())
    }
}

#[derive(Deserialize, Debug)]
pub struct Element {
    status: i32,
    #[serde(rename = "Request.sequence")]
    seq: String,
    #[serde(rename = "Response.HTTPStatusCode")]
    code: i32,
    #[serde(flatten, default)]
    content: Option<MultiRequestElementResponse>,
    #[serde(default)]
    response_messages: Option<ResponseMessages>,
}

impl Element {
    pub fn status(&self) -> i32 {
        self.status
    }

    pub fn seq(&self) -> &str {
        &self.seq
    }

    pub fn code(&self) -> i32 {
        self.code
    }

    pub fn content(&self) -> Option<&MultiRequestElementResponse> {
        self.content.as_ref()
    }

    pub fn content_owned(self) -> Option<MultiRequestElementResponse> {
        self.content
    }

    pub fn content_unsafe(&self) -> &MultiRequestElementResponse {
        self.content().unwrap()
    }

    pub fn content_unsafe_owned(self) -> MultiRequestElementResponse {
        self.content_owned().unwrap()
    }

    pub fn get_messages(&self) -> Option<&ResponseMessages> {
        self.response_messages.as_ref()
    }
}

impl ApiResponse<MultiRequestResponse> {
    pub fn responses(&self) -> Option<HashMap<String, &Element>> {
        self.response().as_ref().map(|r| match &r.responses {
            MRT::Vec(v) => v.iter().map(|e| (e.seq.clone(), e)).collect(),
            MRT::HashMap(m) => m.iter().map(|(k, v)| (k.clone(), v)).collect(),
        })
    }
    pub fn responses_owned_unsafe(self) -> HashMap<String, Element> {
        match self.response_unsafe_owned().responses {
            MRT::Vec(v) => v.into_iter().map(|e| (e.seq.clone(), e)).collect(),
            MRT::HashMap(m) => m,
        }
    }
}
