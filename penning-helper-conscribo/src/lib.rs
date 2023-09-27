use serde::{de::DeserializeOwned, Deserialize, Serialize};

mod error;
pub use error::*;
mod requests;
pub use requests::*;

mod results;
pub use results::*;

use penning_helper_types::Date;

const VERSION: &str = "0.20161212";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginResult {
    session_id: String,
}

#[derive(Debug)]
pub struct ConscriboClient {
    client: reqwest::blocking::Client,
    session_id: String,
    url: String,
}

impl ConscriboClient {
    pub fn new(
        username: impl ToString,
        password: impl ToString,
        url: impl ToString,
    ) -> ConscriboResult<Self> {
        let url = url.to_string();
        let client = reqwest::blocking::Client::new();
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
        })
    }

    pub fn new_from_cfg(cfg: &penning_helper_config::ConscriboConfig) -> ConscriboResult<Self> {
        Self::new(&cfg.username, &cfg.password, &cfg.url)
    }

    pub fn do_request<A: ToRequest, R: DeserializeOwned>(&self, req: A) -> ConscriboResult<R> {
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

        let value: RootResult<R> = serde_json::from_str(&t)?;
        value.to_result()
    }

    pub fn do_multi_request<A: ToRequest, R: DeserializeOwned>(
        &self,
        reqs: Vec<A>,
    ) -> ConscriboResult<Vec<R>> {
        let multi_req = ConscriboMultiRequest::new(reqs);
        let t = self
            .client
            .post(&self.url)
            .header("X-Conscribo-API-Version", VERSION)
            .header("X-Conscribo-SessionId", &self.session_id)
            .json(&multi_req)
            .send()?
            .text()?;
        println!("{}", t);
        let value: MultiRootResult<R> = serde_json::from_str(&t)?;
        value.into()
    }

    pub fn get_field_definitions(&self, entity_type: impl ToString) -> ConscriboResult<Vec<Field>> {
        let req = FieldReq::new(entity_type.to_string());
        let res: FieldRes = self.do_request(req)?;
        Ok(res.fields)
    }

    pub fn get_relations(&self, entity_type: impl ToString) -> ConscriboResult<Vec<Relation>> {
        // {
        //     if let Ok(f) =
        //         std::fs::File::open(format!("relations_{}.json", entity_type.to_string()))
        //     {
        //         let res: Relations = serde_json::from_reader(f)?;
        //         return Ok(res.into());
        //     }
        // }
        let et = entity_type.to_string();
        let req = ListRelations::new(
            et.clone(),
            vec![
                "code".to_string(),
                "naam".to_string(),
                "email".to_string(),
                "rekening".to_string(),
                "membership_started".to_string(),
            ],
        );
        let res: Relations = self.do_request(req)?;

        let mut res: Vec<Relation> = res.into();
        res.iter_mut().for_each(|r| r.source = et.clone());

        Ok(res)
    }

    pub fn get_transactions(
        &self,
        start_date: impl Into<Date>,
        end_date: impl Into<Date>,
    ) -> ConscriboResult<Vec<UnifiedTransaction>> {
        let mensen = self.get_relations("lid")?;
        let onbekend = self.get_relations("onbekend")?;
        let mensen = mensen
            .into_iter()
            .chain(onbekend.into_iter())
            .collect::<Vec<_>>();
        let codes = mensen.iter().map(|m| m.code.clone()).collect::<Vec<_>>();
        let req = ListTransactions::new(vec![
            TransactionFilter::DateStart(start_date.into()),
            TransactionFilter::DateEnd(end_date.into()),
            TransactionFilter::relations(codes),
        ]);

        let transactions: Transactions = self.do_request(req)?;

        let mut transactions = transactions.into_transactions();
        transactions.sort_by_key(|t| t.date);
        let transactions = transactions
            .into_iter()
            .map(|t| t.unify())
            .collect::<Result<Vec<UnifiedTransaction>, TransactionConvertError>>()?;

        Ok(transactions)
    }
}
