use std::{collections::HashSet, path::Path};

use penning_helper_types::Date;
use serde::de::DeserializeOwned;

use crate::{
    Cache, ConscriboMultiRequest, ConscriboResult, Field, FieldReq, FieldRes, ListRelations,
    ListTransactions, LoginRequest, LoginResult, Member, MultiRootResult, NonMember, RelationType,
    Relations, RootResult, ToRequest, TransactionFilter, Transactions, UnifiedTransaction, VERSION,
};

pub struct ConscriboClient {
    client: reqwest::Client,
    session_id: String,
    url: String,
}

impl ConscriboClient {
    pub async fn new(
        username: impl ToString,
        password: impl ToString,
        url: impl ToString,
    ) -> ConscriboResult<Self> {
        let url = url.to_string();
        let client = reqwest::ClientBuilder::new().build()?;

        let login_request =
            LoginRequest::new(username.to_string(), password.to_string()).to_request();
        let res = client
            .post(&url)
            .header("X-Conscribo-API-Version", VERSION)
            .json(&login_request)
            .send()
            .await?;

        let res = res.json::<RootResult<LoginResult>>().await?;
        let res = res.to_result()?;
        Ok(Self {
            client,
            session_id: res.session_id,
            url,
        })
    }

    pub async fn new_from_cfg(
        cfg: &penning_helper_config::ConscriboConfig,
    ) -> ConscriboResult<Self> {
        Self::new(&cfg.username, &cfg.password, &cfg.url).await
    }

    pub async fn do_request<A: ToRequest, R: DeserializeOwned>(
        &self,
        req: A,
    ) -> ConscriboResult<R> {
        let t = self.do_request_str(req).await?;
        // println!("{}", t);
        let value: RootResult<R> = serde_json::from_str(&t)?;
        value.to_result()
    }

    async fn do_request_str<A: ToRequest>(&self, req: A) -> ConscriboResult<String> {
        let req = req.to_request();
        // let command = req.get_command();
        let t = self
            .client
            .post(&self.url)
            .header("X-Conscribo-API-Version", VERSION)
            .header("X-Conscribo-SessionId", &self.session_id)
            .json(&req)
            .send()
            .await?
            .text()
            .await?;
        Ok(t)
    }

    pub async fn do_multi_request<A: ToRequest, R: DeserializeOwned>(
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
            .send()
            .await?
            .text()
            .await?;
        // println!("{}", t);
        let value: MultiRootResult<R> = serde_json::from_str(&t)?;
        value.into()
    }

    pub async fn get_field_definitions(
        &self,
        entity_type: impl ToString,
    ) -> ConscriboResult<Vec<Field>> {
        let req = FieldReq::new(entity_type.to_string());
        let res: FieldRes = self.do_request(req).await?;
        Ok(res.fields)
    }

    pub async fn get_relations<R: RelationType>(&self) -> ConscriboResult<Vec<R>> {
        let req: ListRelations<R> = ListRelations::new(R::ENTITY_TYPE, R::fields());
        let res: Relations<R> = self.do_request(req).await?;

        let mut res: Vec<R> = res.into();

        // res.iter_mut().for_each(|r| r.naam = format!("{} ({})", r.naam, r.code));

        Ok(res)
    }

    pub async fn get_transactions(
        &self,
        cache_path: &Path,
    ) -> ConscriboResult<Option<Vec<UnifiedTransaction>>> {
        let cache_path = cache_path.to_path_buf();
        let cache = if let Ok(mut f) = std::fs::File::open(&cache_path) {
            let cache: Cache = serde_json::from_reader(&mut f)?;

            cache
        } else {
            Cache::default()
        };

        let mensen = self.get_relations::<Member>().await?;
        let onbekend = self.get_relations::<NonMember>().await?;
        let mensen = mensen
            .into_iter()
            .chain(onbekend.into_iter().map(|o| o.into()))
            .collect::<Vec<_>>();
        let codes = mensen.iter().map(|m| m.code.clone()).collect::<Vec<_>>();
        let req = ListTransactions::new(vec![
            TransactionFilter::relations(codes),
            TransactionFilter::DateStart(cache.last_date),
        ]);

        let value: RootResult<Transactions> = self.do_request(req).await?;

        let transactions = value.to_result()?;

        let mut transactions = transactions.into_transactions();
        transactions.sort_by_key(|t| t.date);
        let transactions = transactions
            .into_iter()
            .map(|t| t.unify())
            .collect::<Result<Vec<_>, _>>()?;

        let mut transactions: HashSet<UnifiedTransaction> =
            transactions.into_iter().flatten().collect();
        transactions.extend(cache.transactions.into_iter());
        // remove duplicates from transactions

        let cache = Cache {
            last_date: transactions
                .iter()
                .map(|t| t.date)
                .max()
                .unwrap_or(Date::today()),
            transactions: transactions.iter().cloned().collect(),
        };
        let mut f = std::fs::File::create(cache_path)?;
        serde_json::to_writer(&mut f, &cache)?;

        Ok(Some(transactions.into_iter().collect()))
    }
}
