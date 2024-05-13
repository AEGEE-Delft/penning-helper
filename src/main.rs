use penning_helper_conscribo::{
    AccountResult, AddChangeTransaction, ListAccounts, Transaction, TransactionResult,
};
use penning_helper_types::Date;

fn main() {
    let config = penning_helper_config::Config::load_from_file();
    let conscribo =
        penning_helper_conscribo::ConscriboClient::new_from_cfg(config.conscribo()).unwrap();

    let res: Result<AccountResult, penning_helper_conscribo::ConscriboError> =
        conscribo.do_request(ListAccounts::today());
    let res = res.unwrap();
    let maps = res.to_rekening_maps();
    // println!("{:#?}", maps);
    // let entry = maps.find_closest_match("paarse dassen v");
    // if let Some(entry) = entry {
    //     println!("{:#?}", entry);
    // }
    for e in maps.iter() {
        println!("{}", e.account_name);
    }
}
