// let r = foobar
// .conscribo
// .run(|c| c.get_transactions(Date::new(2020, 1, 1), Date::new(2050, 1, 1)))
// .transpose();
// match r {
// Ok(t) => {
//     self.transactions = t.map(|t| {
//         let mut m = HashMap::new();
//         for tr in t {
//             m.entry(tr.code).or_insert_with(|| vec![]).push(tr);
//         }
//         m.into_iter()
//             .map(|(code, uts)| {
//                 if let Some(m) = members.iter().find(|m| m.code == code) {
//                     RelationTransaction {
//                         t: uts,
//                         name: m.naam.clone(),
//                         iban: m
//                             .rekening
//                             .as_ref()
//                             .map(|a| a.iban.clone())
//                             .unwrap_or("".to_string()),
//                         email: m.email_address.clone(),
//                     }
//                 } else {
//                     RelationTransaction {
//                         t: uts,
//                         name: "Unknown".to_string(),
//                         iban: "Unknown".to_string(),
//                         email: "Unknown".to_string(),
//                     }
//                 }
//             })
//             .collect::<Vec<_>>()
//     });
// }
// Err(e) => {
//     if let Some(s) = ERROR_STUFF.get() {
//         s.send(format!("Error: {}", e)).unwrap();
//     }
//     self.transactions = Some(vec![]);
// }
// }
