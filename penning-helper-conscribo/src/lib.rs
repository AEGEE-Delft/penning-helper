use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{atomic::AtomicUsize, Arc, Mutex, RwLock},
};

use chrono::NaiveDate;
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

#[derive(Clone)]
pub struct ConscriboClient {
    account_name: String,
    credentials: Option<Credentials>,
    session_id: Arc<RwLock<Option<String>>>,
    client: reqwest::blocking::Client,
    t_get: Arc<RwLock<Option<TransactionGet>>>,
    t_get_fast: Arc<RwLock<Option<TransactionGetFast>>>,
}

impl ConscriboClient {
    pub fn new(account_name: String) -> Self {
        Self {
            account_name,
            credentials: None,
            session_id: Arc::new(RwLock::new(Option::None)),
            client: reqwest::blocking::Client::new(),
            t_get: Default::default(),
            t_get_fast: Default::default(),
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

    pub fn get_transactions(&self) -> Result<GetTransactionResult, TransactionConvertError> {
        let r = { self.t_get.write().unwrap().take() };
        if let Some(mut t_get) = r {
            let r = self.execute(
                Transactions::new(1000, t_get.offset + 100)
                    .relations(t_get.relations.iter().map(String::as_str).collect())
                    .accounts(vec!["1001", "1002"]),
            );
            let r = r.unwrap();
            if let Some(m) = r.get_messages() {
                for message in m.errors() {
                    eprintln!("{:?}", message);
                }

                for message in m.warnings() {
                    eprintln!("{:?}", message);
                }

                for message in m.infos() {
                    eprintln!("{:?}", message);
                }
                return Err(TransactionConvertError::Other("oof".to_string()));
            } else if let Some(res) = r.response_owned() {
                let t = res.transactions();
                let t = t
                    .into_values()
                    .map(|t| t.unify())
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                let t: Vec<UnifiedTransaction> = t.into_iter().flatten().collect();
                // println!("Got {} transactions", t.len());
                // println!("Total transactions: {}", t_get.total);
                t_get.unifieds.extend(t);
                t_get.offset += 100;
                if t_get.unifieds.len() >= res.nr_transactions as usize {
                    return Ok(GetTransactionResult::Done(t_get.unifieds));
                } else {
                    let res = GetTransactionResult::NotDone {
                        total: t_get.total,
                        count: t_get.unifieds.len(),
                        from_cache: 0,
                    };
                    self.t_get.write().unwrap().replace(t_get);
                    return Ok(res);
                }
            } else {
                return Err(TransactionConvertError::Other("oof".to_string()));
            }
        } else {
            let all_relations: Vec<String> =
                self.get_relations().into_iter().map(|e| e.code).collect();
            let r = self.execute(
                Transactions::new(1000, 0)
                    .relations(all_relations.iter().map(String::as_str).collect())
                    .accounts(vec!["1001", "1002"]),
            );
            let r = r.unwrap();
            if let Some(m) = r.get_messages() {
                for message in m.errors() {
                    eprintln!("{:?}", message);
                }

                for message in m.warnings() {
                    eprintln!("{:?}", message);
                }

                for message in m.infos() {
                    eprintln!("{:?}", message);
                }
                return Err(TransactionConvertError::Other("oof".to_string()));
            } else if let Some(res) = r.response_owned() {
                let t = res.transactions();
                let t = t
                    .into_values()
                    .map(|t| t.unify())
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                let t: Vec<UnifiedTransaction> = t.into_iter().flatten().collect();
                if t.len() >= res.nr_transactions as usize {
                    return Ok(GetTransactionResult::Done(t));
                } else {
                    let r = GetTransactionResult::NotDone {
                        total: res.nr_transactions,
                        count: t.len(),
                        from_cache: 0,
                    };
                    self.t_get.write().unwrap().replace(TransactionGet {
                        total: res.nr_transactions,
                        offset: 0,
                        relations: all_relations,
                        unifieds: t,
                    });
                    return Ok(r);
                }
            } else {
                return Err(TransactionConvertError::Other("oof".to_string()));
            }
        }
    }

    pub fn get_transactions_faster(&self) -> Result<GetTransactionResult, TransactionConvertError> {
        let t_get_fast = self.t_get_fast.read().unwrap();
        if let Some(tgf) = t_get_fast.as_ref() {
            if tgf.count.load(std::sync::atomic::Ordering::SeqCst) >= tgf.total {
                let res1 = tgf.unifieds.lock().unwrap().clone();
                let mut cache: HashSet<_> =
                    tgf.cache.clone().unifieds.clone().into_iter().collect();
                cache.extend(res1.clone());
                let res: Vec<UnifiedTransaction> = cache.into_iter().collect();
                // find difference between res1 and res
                let diff: Vec<_> = res1.into_iter().filter(|e| !res.contains(e)).collect();
                println!("Found {} new transactions", diff.len());

                ClientCache::new(tgf.total, res.clone()).save();
                return Ok(GetTransactionResult::Done(res));
            } else {
                return Ok(GetTransactionResult::NotDone {
                    total: tgf.total as i64,
                    count: tgf.count.load(std::sync::atomic::Ordering::SeqCst),
                    from_cache: tgf.cache.unifieds.len(),
                });
            }
        } else {
            drop(t_get_fast);
            let cache = ClientCache::load().unwrap_or_else(|| ClientCache::empty());

            let all_relations: Vec<String> =
                self.get_relations().into_iter().map(|e| e.code).collect();
            let relations = Arc::new(all_relations);
            let r = self.execute(
                Transactions::new(0, 0)
                    .relations(relations.iter().map(String::as_str).collect())
                    .accounts(vec!["1001", "1002"])
                    .date_start(cache.date),
            );

            let r = r.unwrap();
            if let Some(m) = r.get_messages() {
                for message in m.errors() {
                    eprintln!("{:?}", message);
                }

                for message in m.warnings() {
                    eprintln!("{:?}", message);
                }

                for message in m.infos() {
                    eprintln!("{:?}", message);
                }
                return Err(TransactionConvertError::Other("oof".to_string()));
            } else if let Some(res) = r.response_owned() {
                let r = GetTransactionResult::NotDone {
                    total: res.nr_transactions,
                    count: 0,
                    from_cache: cache.unifieds.len(),
                };
                let tgf = TransactionGetFast {
                    total: res.nr_transactions as usize,
                    offset: Arc::new(AtomicUsize::new(0)),
                    count: Arc::new(AtomicUsize::new(0)),
                    relations: relations.clone(),
                    unifieds: Arc::new(Mutex::new(vec![])),
                    cache: Arc::new(cache),
                };
                self.t_get_fast.write().unwrap().replace(tgf);
                let c = self.clone();
                std::thread::spawn(move || {
                    c.get_transactions_faster_worker();
                });
                return Ok(r);
            } else {
                return Err(TransactionConvertError::Other("niks?".to_string()));
            }
        }
    }

    fn get_transactions_faster_worker(&self) {
        let relations = self.get_tgf_relations();
        let cache = self.get_tgf_cache();
        loop {
            let offset = self.get_tgf_offset();
            let t = self.tgf_transactions(offset as i64, &relations, cache.date);
            {
                let tgf = self.t_get_fast.read().unwrap();
                if let Some(tgf) = tgf.as_ref() {
                    tgf.count
                        .fetch_add(t.len(), std::sync::atomic::Ordering::SeqCst);
                    tgf.offset
                        .fetch_add(100, std::sync::atomic::Ordering::SeqCst);
                    tgf.unifieds.lock().unwrap().extend(t);
                    if tgf.count.load(std::sync::atomic::Ordering::SeqCst) >= tgf.total {
                        break;
                    }
                    println!("memes");
                } else {
                    break;
                }
            }
        }
    }

    fn get_tgf_offset(&self) -> usize {
        let tgf = self.t_get_fast.read().unwrap();
        if let Some(tgf) = tgf.as_ref() {
            tgf.offset.load(std::sync::atomic::Ordering::SeqCst)
        } else {
            0
        }
    }

    fn get_tgf_relations(&self) -> Arc<Vec<String>> {
        let tgf = self.t_get_fast.read().unwrap();
        if let Some(tgf) = tgf.as_ref() {
            tgf.relations.clone()
        } else {
            Arc::new(vec![])
        }
    }

    fn get_tgf_cache(&self) -> Arc<ClientCache> {
        let tgf = self.t_get_fast.read().unwrap();
        if let Some(tgf) = tgf.as_ref() {
            tgf.cache.clone()
        } else {
            Arc::new(ClientCache::empty())
        }
    }

    fn tgf_transactions(
        &self,
        offset: i64,
        relations: &[String],
        start_date: NaiveDate,
    ) -> Vec<UnifiedTransaction> {
        let r = self.execute(
            Transactions::new(100, offset)
                .relations(relations.iter().map(String::as_str).collect())
                .accounts(vec!["1001", "1002"])
                .date_start(start_date),
        );
        let r = r.unwrap();
        if let Some(m) = r.get_messages() {
            for message in m.errors() {
                eprintln!("{:?}", message);
            }

            for message in m.warnings() {
                eprintln!("{:?}", message);
            }

            for message in m.infos() {
                eprintln!("{:?}", message);
            }
            return vec![];
        } else if let Some(res) = r.response_owned() {
            let t = res.transactions();
            let t = t
                .into_values()
                .map(|t| t.unify())
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let t: Vec<UnifiedTransaction> = t.into_iter().flatten().collect();
            t
        } else {
            vec![]
        }
    }
}

struct TransactionGet {
    total: i64,
    offset: i64,
    relations: Vec<String>,
    unifieds: Vec<UnifiedTransaction>,
}

pub enum GetTransactionResult {
    Done(Vec<UnifiedTransaction>),
    NotDone {
        total: i64,
        count: usize,
        from_cache: usize,
    },
}

#[derive(Clone)]
struct TransactionGetFast {
    total: usize,
    offset: Arc<AtomicUsize>,
    count: Arc<AtomicUsize>,
    relations: Arc<Vec<String>>,
    unifieds: Arc<Mutex<Vec<UnifiedTransaction>>>,
    cache: Arc<ClientCache>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct ClientCache {
    total: usize,
    unifieds: Vec<UnifiedTransaction>,
    date: NaiveDate,
}

impl ClientCache {
    fn new(total: usize, unifieds: Vec<UnifiedTransaction>) -> Self {
        Self {
            total,
            unifieds,
            date: chrono::Local::now().date_naive(),
        }
    }

    fn empty() -> Self {
        Self {
            total: 0,
            unifieds: vec![],
            date: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
        }
    }

    pub fn load() -> Option<Self> {
        let dir = dirs::data_local_dir()
            .unwrap_or(PathBuf::from("."))
            .join("penning-helper")
            .join("clientcache.bin");
        let file = std::fs::File::open(dir).ok()?;
        let reader = std::io::BufReader::new(file);
        let res = bincode::deserialize_from(reader).ok()?;
        Some(res)
    }

    pub fn save(&self) {
        let dir = dirs::data_local_dir()
            .unwrap_or(PathBuf::from("."))
            .join("penning-helper");
        std::fs::create_dir_all(&dir).unwrap();
        let dir = dir.join("clientcache.bin");
        let file = std::fs::File::create(dir).unwrap();
        let writer = std::io::BufWriter::new(file);
        bincode::serialize_into(writer, self).unwrap();
    }
}
