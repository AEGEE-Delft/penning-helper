use std::{
    cell::RefCell,
    collections::HashSet,
    path::Path,
    sync::{Arc, RwLock},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

mod error;
pub use error::*;
mod requests;
pub use requests::*;

mod results;
pub use results::*;
mod async_client;
pub use async_client::ConscriboClient as AsyncConscriboClient;

use penning_helper_types::Date;

const VERSION: &str = "0.20161212";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResult {
    session_id: String,
}

#[derive(Debug, Clone)]
pub struct ConscriboClient {
    client: reqwest::blocking::Client,
    session_id: String,
    url: String,
    transactions: Arc<RwLock<Option<Vec<UnifiedTransaction>>>>,
    getting_transactions: RefCell<bool>,
}

pub trait RelationType: Serialize + DeserializeOwned {
    const ENTITY_TYPE: &'static str;

    fn entity_type() -> &'static str {
        Self::ENTITY_TYPE
    }

    fn fields() -> Vec<&'static str>;
}

impl ConscriboClient {
    pub fn new(
        username: impl ToString,
        password: impl ToString,
        url: impl ToString,
    ) -> ConscriboResult<Self> {
        let url = url.to_string();
        let client = reqwest::blocking::ClientBuilder::new()
            .timeout(None)
            .build()?;

        let login_request =
            LoginRequest::new(username.to_string(), password.to_string()).to_request();
        let res = client
            .post(&url)
            .header("X-Conscribo-API-Version", VERSION)
            .json(&login_request)
            .send()?;

        let res = res.json::<RootResult<LoginResult>>()?;
        let res = res.to_result()?;
        Ok(Self {
            client,
            session_id: res.session_id,
            url,
            transactions: Default::default(),
            getting_transactions: Default::default(),
        })
    }

    pub fn new_from_cfg(cfg: &penning_helper_config::ConscriboConfig) -> ConscriboResult<Self> {
        Self::new(&cfg.username, &cfg.password, &cfg.account_name)
    }

    pub fn do_request<A: ToRequest>(&self, req: A) -> ConscriboResult<A::Response> {
        let t = self.do_request_str(req)?;
        // println!("{}", t);
        let value: RootResult<A::Response> = serde_json::from_str(&t)?;
        value.to_result()
    }

    fn do_request_str<A: ToRequest>(&self, req: A) -> ConscriboResult<String> {
        let req = req.to_request();
        // let command = req.get_command();
        let t = self
            .client
            .post(&self.url)
            .header("X-Conscribo-API-Version", VERSION)
            .header("X-Conscribo-SessionId", &self.session_id)
            .json(&req)
            .send()?
            .text()?;
        Ok(t)
    }

    pub fn do_multi_request<A: ToRequest>(
        &self,
        reqs: Vec<A>,
    ) -> ConscriboResult<Vec<A::Response>> {
        let multi_req = ConscriboMultiRequest::new(reqs);
        let t = self
            .client
            .post(&self.url)
            .header("X-Conscribo-API-Version", VERSION)
            .header("X-Conscribo-SessionId", &self.session_id)
            .json(&multi_req)
            .send()?
            .text()?;
        // println!("{}", t);
        let value: MultiRootResult<A::Response> = serde_json::from_str(&t)?;
        value.into()
    }

    pub fn get_field_definitions(&self, entity_type: impl ToString) -> ConscriboResult<Vec<Field>> {
        let req = FieldReq::new(entity_type.to_string());
        let res: FieldRes = self.do_request(req)?;
        Ok(res.fields)
    }

    pub fn get_relations<'a, R: 'a + RelationType>(&self) -> ConscriboResult<Vec<R>> {
        // {
        //     if let Ok(f) =
        //         std::fs::File::open(format!("relations_{}.json", entity_type.to_string()))
        //     {
        //         let res: Relations = serde_json::from_reader(f)?;
        //         return Ok(res.into());
        //     }
        // }
        let fields = R::fields();
        // let mut fields = vec![
        //     "code".to_string(),
        //     "weergavenaam".to_string(),
        //     "email".to_string(),
        //     "rekening".to_string(),
        //     "membership_started".to_string(),
        //     "geen_invoice".to_string(),
        // ];
        // if et == "lid" {
        //     fields.push("alumni_lidmaatschap_gestart".to_string());
        //     fields.push("alumni_lidmaatschap_be__indigt".to_string());
        //     fields.push("alumni_contributie".to_string());
        // }
        let req = ListRelations::new(R::ENTITY_TYPE.to_string(), fields);
        let res: Relations<R> = self.do_request(req)?;

        let res: Vec<R> = res.into();
        // res.iter_mut().for_each(|r| r.naam = format!("{} ({})", r.naam, r.code));

        Ok(res)
    }

    pub fn update_relations(
        &self,
        relations: Vec<UpdateRelation>,
    ) -> ConscriboResult<Vec<ReplaceRelationsResult>> {
        let r: Vec<ReplaceRelationsResult> = self.do_multi_request(relations)?;
        Ok(r)
    }

    pub fn get_transactions(
        &self,
        cache_path: &Path,
    ) -> ConscriboResult<Option<Vec<UnifiedTransaction>>> {
        let running = { *self.getting_transactions.borrow() };
        if running {
            let t = { self.transactions.read().unwrap().clone() };
            return Ok(t);
        }
        let cache_path = cache_path.to_path_buf();
        let cache = if let Ok(mut f) = std::fs::File::open(&cache_path) {
            let cache: Cache = serde_json::from_reader(&mut f)?;

            cache
        } else {
            Cache::default()
        };
        *self.getting_transactions.borrow_mut() = true;
        let mensen: Vec<Member> = self.get_relations()?;
        let onbekend: Vec<NonMember> = self.get_relations()?;
        let mensen = mensen
            .into_iter()
            .chain(onbekend.into_iter().map(|m| m.into()))
            .collect::<Vec<_>>();
        let codes = mensen.iter().map(|m| m.code.clone()).collect::<Vec<_>>();
        let req = ListTransactions::new(vec![
            TransactionFilter::relations(codes),
            TransactionFilter::DateStart(cache.last_date),
        ]);
        let client = self.client.clone();
        let sesh = self.session_id.clone();
        let url = self.url.clone();
        let req = req.to_request();

        let lock = self.transactions.clone();

        std::thread::Builder::new()
            .name("Transaction gatherer 9000".to_string())
            .spawn(move || {
                let cache = cache;
                println!("Getting transactions");
                let r = client
                    .post(&url)
                    .header("X-Conscribo-API-Version", VERSION)
                    .header("X-Conscribo-SessionId", &sesh)
                    .json(&req)
                    .send()
                    .unwrap()
                    .text()
                    .unwrap();
                // println!("{}", r);

                let value: RootResult<Transactions> = serde_json::from_str(&r).unwrap();

                let transactions = value.to_result().unwrap();
                println!("Got {} transactions", transactions.len());

                let mut transactions = transactions.into_transactions();
                transactions.sort_by_key(|t| t.date);
                let transactions = transactions
                    .into_iter()
                    .map(|t| t.unify())
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                let mut transactions: HashSet<UnifiedTransaction> =
                    transactions.into_iter().flatten().collect();
                transactions.extend(cache.transactions.into_iter());
                // remove duplicates from transactions
                println!("Got {} transactions", transactions.len());

                let cache = Cache {
                    last_date: transactions
                        .iter()
                        .map(|t| t.date)
                        .max()
                        .unwrap_or(Date::today()),
                    transactions: transactions.iter().cloned().collect(),
                };
                let mut f = std::fs::File::create(cache_path).unwrap();
                serde_json::to_writer(&mut f, &cache).unwrap();
                println!("Converted {} transactions", transactions.len());
                lock.write()
                    .unwrap()
                    .replace(transactions.into_iter().collect::<Vec<_>>());
            })?;

        Ok(None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Cache {
    last_date: Date,
    transactions: Vec<UnifiedTransaction>,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            last_date: Date::new(2020, 01, 01).unwrap(),
            transactions: Default::default(),
        }
    }
}
