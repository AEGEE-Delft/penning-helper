use std::sync::{Arc, RwLock};

use entities::Entity;
use response::ApiResponse;
use serde::{de::DeserializeOwned, Serialize};
use session::Credentials;
use transactions::{TransactionConvertError, Transactions, UnifiedTransaction};

pub mod response;

pub mod entity_types;

pub mod session;

pub mod multirequest;

pub mod field_definitions;

pub mod entities;

pub mod accounts;

pub mod transactions;

pub mod add_invoice;

pub mod add_transaction;

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
        println!("{}", response_text);
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

#[derive(Clone)]
pub struct ConscriboClient {
    account_name: String,
    credentials: Option<Credentials>,
    session_id: Arc<RwLock<Option<String>>>,
    client: reqwest::blocking::Client,
    t_get: Arc<RwLock<Option<TransactionGet>>>,
}

impl ConscriboClient {
    pub fn new(account_name: String) -> Self {
        Self {
            account_name,
            credentials: None,
            session_id: Arc::new(RwLock::new(Option::None)),
            client: reqwest::blocking::Client::new(),
            t_get: Default::default(),
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

    pub fn get_relations(&self) -> Vec<Entity> {
        let leden = self
            .execute(
                entities::Entities::new().filter(entities::filters::Filter::entity_type("lid")),
            )
            .unwrap();
        let onbekend = self
            .execute(
                entities::Entities::new()
                    .filter(entities::filters::Filter::entity_type("onbekend")),
            )
            .unwrap();
        let mut entities = vec![];
        if let Some(leden) = leden.response_owned() {
            entities.extend(leden.entities.into_values());
        }
        if let Some(onbekend) = onbekend.response_owned() {
            entities.extend(onbekend.entities.into_values());
        }

        entities
    }

    pub fn get_transactions(
        &self,
    ) -> Result<Option<Vec<UnifiedTransaction>>, TransactionConvertError> {
        let r = { self.t_get.write().unwrap().take() };
        if let Some(mut t_get) = r {
            let r = self.execute(
                Transactions::new(100, t_get.offset + 100)
                    .relations(t_get.relations.iter().map(String::as_str).collect())
                    .accounts(vec!["1001", "1002"]),
            );
            let r = r.unwrap();
            if let Some(res) = r.response_owned() {
                let t = res.transactions;
                let t = t
                    .into_values()
                    .map(|t| t.unify())
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                let t: Vec<UnifiedTransaction> = t.into_iter().flatten().collect();
                println!("Got {} transactions", t.len());
                println!("Total transactions: {}", t_get.total);
                t_get.unifieds.extend(t);
                t_get.offset += 100;
                if t_get.unifieds.len() >= res.nr_transactions as usize {
                    return Ok(Some(t_get.unifieds));
                }
                self.t_get.write().unwrap().replace(t_get);
            }
        } else {
            let all_relations: Vec<String> =
                self.get_relations().into_iter().map(|e| e.code).collect();
            let r = self.execute(
                Transactions::new(100, 0)
                    .relations(all_relations.iter().map(String::as_str).collect())
                    .accounts(vec!["1001", "1002"]),
            );
            let r = r.unwrap();
            if let Some(res) = r.response_owned() {
                let t = res.transactions;
                let t = t
                    .into_values()
                    .map(|t| t.unify())
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                let t: Vec<UnifiedTransaction> = t.into_iter().flatten().collect();
                if t.len() >= res.nr_transactions as usize {
                    return Ok(Some(t));
                }
                self.t_get.write().unwrap().replace(TransactionGet {
                    total: res.nr_transactions,
                    offset: 0,
                    relations: all_relations,
                    unifieds: t,
                });
            }
        }

        Ok(None)
    }
}

struct TransactionGet {
    total: i64,
    offset: i64,
    relations: Vec<String>,
    unifieds: Vec<UnifiedTransaction>,
}
