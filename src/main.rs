use std::ops::Mul;

use penning_helper_conscribo::{
    accounts::AccountRequest,
    entities::{filters::Filter, Entities},
    multirequest::MultiRequest,
    session::Credentials,
    transactions::{Transactions, TransactionsResponse},
    ConscriboClient,
};

fn main() {
    let api_client = penning_helper_conscribo::ConscriboClient::new("aegee-delft-test".to_string());

    let res = api_client
        .execute(
            MultiRequest::new()
                .push("lid", Entities::new().filter(Filter::entity_type("lid")))
                .push(
                    "onbekend",
                    Entities::new().filter(Filter::entity_type("onbekend")),
                ),
        )
        .unwrap();
    let map = res.responses().unwrap();
    let lid = map.get("lid").unwrap();
    if let Some(m) = lid.get_messages() {
        for message in m.errors() {
            println!("{:?}", message);
        }

        for message in m.warnings() {
            println!("{:?}", message);
        }

        for message in m.infos() {
            println!("{:?}", message);
        }
        return;
    }

    let lid = lid.content_unsafe().as_entity_request();
    let onbekend = map.get("onbekend").unwrap();
    if let Some(m) = onbekend.get_messages() {
        for message in m.errors() {
            println!("{:?}", message);
        }

        for message in m.warnings() {
            println!("{:?}", message);
        }

        for message in m.infos() {
            println!("{:?}", message);
        }
        return;
    }

    let onbekend = onbekend.content_unsafe().as_entity_request();

    let r = lid.map(|it| it.entities.clone()).and_then(|it| {
        onbekend.map(|o| {
            let mut e = o.entities.clone();
            e.extend(it);
            e
        })
    });
    if let Some(res) = r {
        let t = api_client.execute(Transactions::new(100, 0)).unwrap();
        if let Some(m) = t.get_messages() {
            for message in m.errors() {
                println!("{:?}", message);
            }

            for message in m.warnings() {
                println!("{:?}", message);
            }

            for message in m.infos() {
                println!("{:?}", message);
            }
        } else {
            let t = TransactionsResponse::from_json(include_str!("../t.json"));
            println!("{:?}", t.transactions["1"].description);
        }
    } else if let Some(messages) = res.get_messages() {
        for message in messages.errors() {
            println!("{:?}", message);
        }

        for message in messages.warnings() {
            println!("{:?}", message);
        }

        for message in messages.infos() {
            println!("{:?}", message);
        }
    }
}
