use std::sync::{Arc, RwLock};

use response::ApiResponse;
use serde::{de::DeserializeOwned, Serialize};
use session::Credentials;

pub mod response;

pub mod entity_types;

pub mod session;

pub mod multirequest;

pub mod field_definitions;

pub mod entities;

pub mod accounts;

pub mod transactions;

pub mod add_invoice;

const VERSION: &'static str = "1.20240610";

pub trait ApiCall: Serialize {
    type Response: DeserializeOwned + Default;
    const PATH: &'static str;
    const METHOD: reqwest::Method;

    #[deprecated = "Prefer ConscriboClient::execute"]
    fn call(&self, client: &ConscriboClient) -> Result<ApiResponse<Self::Response>, RequestError> {
        let url = format!(
            "https://api.secure.conscribo.nl/{}/{}/{}",
            client.account_name,
            Self::PATH,
            self.path_params().join("/"),
        );
        // let mut request = client.client.post(&url).header("X-Conscribo-API-Version", VERSION);
        let mut request = client
            .client
            .request(Self::METHOD, &url)
            .header("X-Conscribo-API-Version", VERSION);
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
        // let now = SystemTime::now()
        //     .duration_since(SystemTime::UNIX_EPOCH)
        //     .unwrap()
        //     .as_millis();
        // std::fs::write(format!("./hidden/{}.json", now), &response_text).unwrap();
        let response = serde_json::from_str(&response_text)?;

        Ok(response)
    }

    fn path_params(&self) -> Vec<&str> {
        vec![]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

pub struct ConscriboClient {
    account_name: String,
    credentials: Option<Credentials>,
    session_id: Arc<RwLock<Option<String>>>,
    client: reqwest::blocking::Client,
}

impl ConscriboClient {
    pub fn new(account_name: String) -> Self {
        Self {
            account_name,
            credentials: None,
            session_id: Arc::new(RwLock::new(Option::None)),
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    pub fn with_session_id(self, session_id: String) -> Self {
        *self.session_id.write().unwrap() = Some(session_id);
        self
    }

    fn check_session_id(&self) {
        if self.session_id.read().unwrap().is_none() {
            if let Some(creds) = self.credentials.as_ref() {
                #[allow(deprecated)]
                let response = creds.call(self).unwrap();
                if let Some(session_id) = response.response() {
                    *self.session_id.write().unwrap() = Some(session_id.session_id.clone());
                } else {
                    let msgs = response.get_messages().unwrap();
                    for msg in msgs.errors() {
                        eprintln!("{:?}", msg);
                    }

                    for msg in msgs.warnings() {
                        eprintln!("{:?}", msg);
                    }

                    for msg in msgs.infos() {
                        eprintln!("{:?}", msg);
                    }

                    panic!("No session id returned");
                }
            } else {
                panic!("No session id set");
            }
        }
    }

    pub fn execute<A: ApiCall>(&self, call: A) -> Result<ApiResponse<A::Response>, RequestError> {
        self.check_session_id();
        #[allow(deprecated)]
        call.call(self)
    }
}
