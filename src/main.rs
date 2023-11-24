use std::{fmt::Display, collections::HashMap};

use genpdf::{
    elements::{self, LinearLayout, PaddedElement, Paragraph, TableLayout},
    style::StyledString,
    Element, Margins,
};
use penning_helper_conscribo::{
    AddChangeTransaction, ConscriboClient, ConscriboMultiRequest, UnifiedTransaction, MultiRootResult, Transaction, TransactionResult, ConscriboResult,
};

use penning_helper_types::{Address, Date, Euro};

fn main() {
    let config: penning_helper_config::Config =
        toml::from_str(std::fs::read_to_string(".sample.toml").unwrap().as_str()).unwrap();

    let client = ConscriboClient::new_from_cfg(config.conscribo()).unwrap();
    // let members = client.get_relations("lid").unwrap();
    // let others = client.get_relations("onbekend").unwrap();
    // let others = others.into_iter().filter(|o| !members.iter().any(|m|m.naam == o.naam)).collect::<Vec<_>>();
    // let all_relations = members
    //     .into_iter()
    //     .chain(others.into_iter())
    //     // .filter(|r| r.naam == "Julius de Jeu")
    //     .collect::<Vec<_>>();
    // // let f = std::fs::File::open("borrel.csv").unwrap();
    // // let mut turf_list = penning_helper_turflists::csv::read_csv(f).unwrap();
    // let mut turf_list = penning_helper_turflists::xlsx::read_excel("intro.xlsx", 420.0.into()).unwrap();
    // turf_list.shrink();
    // let names = all_relations
    //     .iter()
    //     .map(|r| r.naam.clone())
    //     .collect::<Vec<_>>();
    // let emails = all_relations
    //     .iter()
    //     .map(|r| r.email_address.clone())
    //     .collect::<Vec<_>>();
    // let mut no_matches: HashMap<String, Euro> = HashMap::new();
    // for entry in turf_list.iter() {
    //     // println!("{:#?}", entry);
    //     if entry.iban.is_some() {
    //         println!("{} already has an iban", entry.name);
    //         continue;
    //     }
    //     let Some((idx, _)) = entry.best_idx(&names, &emails) else {
    //         println!("{} has no match", entry.name);
    //         *no_matches.entry(entry.name.to_string()).or_default()+= entry.amount;
    //         continue;
    //     };
        
    //     println!(
    //         "matched {} to {} ({} spent) (source {})",
    //         entry.name, all_relations[idx].naam, entry.amount, all_relations[idx].source
    //     );
    // }
    // let matches = turf_list
    //     .iter()
    //     .flat_map(|t| t.best_idx(&names, &emails))
    //     .map(|(idx, amount)| (&all_relations[idx], amount))
    //     .collect::<Vec<_>>();
    // let transactions = matches
    //     .iter()
    //     .map(|(r, eur)| {
    //         let eur = *eur;
    //         let a = AddChangeTransaction::new(
    //             Date::today(),
    //             "Introweekend".to_string(),
    //         );
    //         let a = if eur > Euro::default() {
    //             a.add_debet("6022-01".to_string(), eur, "T2324-03".to_string(), r.code)
    //         } else if eur < Euro::default() {
    //             a.add_credit("6022-01".to_string(), eur, "T2324-03".to_string(), r.code)
    //         } else {
    //             a
    //         };
    //         // println!("{:#?}", a);
    //         // println!("{}", serde_json::to_string_pretty(&a).unwrap());
    //         a
    //     })
    //     .collect::<Vec<_>>();
    // // let message = ConscriboMultiRequest::new(transactions);
    // println!("{:#?}", transactions);
    // // let res: MultiRootResult<TransactionResult> = client.do_multi_request(transactions).unwrap();
    // // let transactions = include_str!("../transactions.json");
    // // let res: ConscriboResult<_> = serde_json::from_str::<MultiRootResult<TransactionResult>>(transactions).unwrap().into();

    // // println!("{:#?}", res);
    // println!("Could not find the following people:");
    // for (name, amount) in no_matches {
    //     println!("{}: {}", name, amount);
    // }


    // let mut transactions = client
    //     .get_transactions()
    //     .unwrap();

    // while matches!(transactions, None) {
    //     println!("Waiting for transactions to be available");
    //     std::thread::sleep(std::time::Duration::from_secs(5));
    //     transactions = client
    //         .get_transactions()
    //         .unwrap();
    // }
    // let transactions = transactions.unwrap();
    // let t2 = transactions.iter().filter(|t| t.code == 1016).collect::<Vec<_>>();
    // let total = t2.iter().map(|t| t.cost).sum::<Euro>();
    // let transactions_from = t2.iter().filter(|t| t.date >= Date::new(2023, 09, 01).unwrap()).collect::<Vec<_>>();
    // let total_from = transactions_from.iter().map(|t| t.cost).sum::<Euro>();

    // println!("You still had {} left from previous invoices", total - total_from);
    // for t in &transactions_from {
    //     println!("{}: {}", t.description, t.cost);
    // }
    // println!("total: {}", total);

    let l = penning_helper_turflists::csv::read_csv("/home/jdejeu/Downloads/balansen_export_2023-11-22.csv");
    println!("{:#?}", l);

}

// fn stuff(cfg: &penning_helper_config::ConscriboConfig) -> (Euro, Vec<u8>) {
//     let client = ConscriboClient::new_from_cfg(cfg).unwrap();
//     // println!("{:?}", client);
//     // let res = client.get_field_definitions("persoon").unwrap();
//     // println!("{:#?}", res);
//     let members = client.get_relations("persoon").unwrap();
//     let me = members
//         .iter()
//         .find(|m| m.naam == "Julius de Jeu")
//         .unwrap()
//         .clone();
//     println!("{:#?}", me);
//     let unknowns = client.get_relations("onbekend").unwrap();
//     let all_relations = members
//         .into_iter()
//         .chain(unknowns.into_iter())
//         .collect::<Vec<_>>();
//     let all_relations_no_bank = all_relations
//         .iter()
//         .filter(|r| r.rekening.is_none())
//         .collect::<Vec<_>>();
//     for relation in all_relations_no_bank {
//         println!("{}", relation.naam);
//     }

//     let transactions = client
//         .get_transactions()
//         .unwrap().unwrap();

//     let transactions = transactions
//         .into_iter()
//         .filter(|t| t.code == me.code)
//         .collect::<Vec<_>>();
//     // println!("{:#?}", transactions);
//     let r = transactions.iter().map(|t| t.cost).sum::<Euro>();
//     println!("{}", r);
//     // penning_helper_sepa::gen_xml();
//     (r, create_pdf(transactions, &me.naam))
// }

trait SmallPad: Sized + Element {
    fn wrap_small_pad(self) -> PaddedElement<Self>
    where
        Self: Sized,
    {
        PaddedElement::new(self, Margins::trbl(1, 1, 1, 1))
    }
}

impl<E: Element> SmallPad for E {}

fn create_debit_credit_elements(cost: Euro) -> (impl Element, impl Element) {
    if cost > Euro::default() {
        (
            genpdf::elements::Paragraph::new(StyledString::new(
                format!("{:-}", cost),
                genpdf::style::Style::new().with_color(genpdf::style::Color::Rgb(255, 0, 0)),
            )),
            genpdf::elements::Paragraph::new(""),
        )
    } else if cost < Euro::default() {
        (
            genpdf::elements::Paragraph::new(""),
            genpdf::elements::Paragraph::new(StyledString::new(
                format!("{:-}", cost),
                genpdf::style::Style::new().with_color(genpdf::style::Color::Rgb(0, 255, 0)),
            )),
        )
    } else {
        (
            genpdf::elements::Paragraph::new(""),
            genpdf::elements::Paragraph::new(""),
        )
    }
}

fn create_pdf(transactions: Vec<UnifiedTransaction>, name: &str) -> Vec<u8> {
    let total = transactions.iter().map(|t| t.cost).sum::<Euro>();
    let font = genpdf::fonts::from_files("./fonts", "Roboto", None).unwrap();
    let mut doc = genpdf::Document::new(font);
    doc.set_title("Gamers");
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_header(|h| {
        genpdf::elements::Paragraph::new(StyledString::new(
            format!("Page {}", h),
            genpdf::style::Style::new().with_font_size(8),
        ))
        .padded(5)
    });
    decorator.set_margins(20);
    doc.set_page_decorator(decorator);
    let image = image::open("logo.png").unwrap();
    let image = image.to_rgb8();
    let w = image.width();
    let max_w = 500;
    let scale_w = max_w as f64 / w as f64;
    let image = image::DynamicImage::ImageRgb8(image);

    let image = genpdf::elements::Image::from_dynamic_image(image)
        .expect("Failed to load test image")
        .with_alignment(genpdf::Alignment::Center)
        .with_scale((scale_w, scale_w));
    // doc.push(image);
    let mut table = genpdf::elements::TableLayout::new(vec![2, 1]);

    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(
        false, false, false,
    ));
    let header = LinearLayout::vertical()
        .element(genpdf::elements::Paragraph::new(name))
        .element(genpdf::elements::Paragraph::new(format!(
            "{}",
            Date::today()
        )))
        .element(genpdf::elements::Paragraph::new("AEGEE-Delft"));

    table.row().element(header).element(image).push().unwrap();
    doc.push(table);

    let text = if total == Euro::default() {
        format!("You are even with AEGEE-Delft, no action is required.")
    } else if total > Euro::default() {
        format!("You owe AEGEE-Delft {:-}, it will be deducted in about 3 days. If this is wrong please send an email to treasurer@aegee-delft.nl!", total)
    } else {
        format!("AEGEE-Delft owes you {:-}, it will be transferred in about 6 days. If this is wrong please send an email to treasurer@aegee-delft.nl!", total)
    };

    doc.push(genpdf::elements::Paragraph::new(text).padded((1, 1, 5, 1)));

    let mut table = genpdf::elements::TableLayout::new(vec![2, 3, 1, 1]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, true));
    table
        .row()
        .element(genpdf::elements::PaddedElement::new(
            genpdf::elements::Paragraph::new(StyledString::new(
                "Date",
                genpdf::style::Style::new().with_line_spacing(2.),
            )),
            Margins::trbl(1, 1, 5, 1),
        ))
        .element(genpdf::elements::Paragraph::new("Description").wrap_small_pad())
        .element(genpdf::elements::Paragraph::new("Debet").wrap_small_pad())
        .element(genpdf::elements::Paragraph::new("Credit").wrap_small_pad())
        .push()
        .unwrap();
    for t in &transactions {
        let (debet, credit) = create_debit_credit_elements(t.cost);

        table
            .row()
            .element(genpdf::elements::PaddedElement::new(
                genpdf::elements::Paragraph::new(StyledString::new(
                    format!("{}", t.date),
                    genpdf::style::Style::new().with_line_spacing(2.),
                )),
                Margins::trbl(1, 1, 5, 1),
            ))
            .element(
                genpdf::elements::Paragraph::new(format!("{}", t.description)).wrap_small_pad(),
            )
            .element(debet.wrap_small_pad())
            .element(credit.wrap_small_pad())
            .push()
            .unwrap();
    }

    let (debet, credit) = create_debit_credit_elements(total);
    table
        .row()
        .element(genpdf::elements::PaddedElement::new(
            genpdf::elements::Paragraph::new(StyledString::new(
                "Total",
                genpdf::style::Style::new().with_line_spacing(2.),
            )),
            Margins::trbl(1, 1, 5, 1),
        ))
        .element(genpdf::elements::Paragraph::new("").wrap_small_pad())
        .element(debet.wrap_small_pad())
        .element(credit.wrap_small_pad())
        .push()
        .unwrap();
    doc.push(table);

    const EXPLAINER: &str = "In this invoice the 'Debet' side is money you spent on AEGEE-Delft activities, like social drinks or activities. \
    The 'Credit' side is money you sent AEGEE-Delft, either directly through an invoice like this one, or by declaring costs you made for committees.";
    doc.push(genpdf::elements::Paragraph::new(EXPLAINER).wrap_small_pad());

    let mut buf = vec![];
    doc.render(&mut buf).unwrap();
    buf
}

struct FactuurItem {
    name: String,
    cost: Euro,
    amount: f64,
}

impl FactuurItem {
    pub fn new(name: String, cost: Euro, amount: f64) -> Self {
        Self { name, cost, amount }
    }
}

macro_rules! paragraphs {
    ($($e:expr),+ $(,)?) => {
        elements::LinearLayout::vertical()
        $(
            .element(elements::Paragraph::new($e))
        )+
    };
}

fn factuur_generator(
    cfg: &penning_helper_config::Config,
    address: Address,
    date: Date,
    factuur_number: u32,
    items: &[FactuurItem],
) -> Vec<u8> {
    let mut header = TableLayout::new(vec![2, 1]);
    let image = image::open("logo.png").unwrap();
    let image = image.to_rgb8();
    let w = image.width();
    let max_w = 500;
    let scale_w = max_w as f64 / w as f64;
    let image = image::DynamicImage::ImageRgb8(image);

    let aegee_address = elements::LinearLayout::vertical()
        .element(elements::Paragraph::new("AEGEE-Delft"))
        .element(elements::Paragraph::new("Kanaalweg 4"))
        .element(elements::Paragraph::new("2628 EB Delft"))
        .element(elements::Paragraph::new("The Netherlands"))
        .styled(genpdf::style::Style::new().with_font_size(8).bold())
        .padded((0, 0, 5, 0));

    let other_info = paragraphs!(
        "Tel: +31 6 13523999",
        "",
        "Email: treasurer@aegee-delft.nl",
        "Email: board@aegee-delft.nl",
        "Web: https://aegee-delft.nl",
        "",
        "Rabobank:",
        format!("BIC: {}", cfg.sepa().company_bic),
        format!("IBAN: {}", cfg.sepa().company_iban),
        "",
        "KvK: Delft, nr. 40398155"
    )
    .styled(genpdf::style::Style::new().with_font_size(8));

    let mut recipient_info = LinearLayout::vertical();
    {
        for line in address.iter_with_empty() {
            recipient_info = recipient_info.element(Paragraph::new(line));
        }
    }

    let image = genpdf::elements::Image::from_dynamic_image(image)
        .expect("Failed to load test image")
        .with_alignment(genpdf::Alignment::Left)
        .with_scale((scale_w, scale_w));
    header
        .row()
        .element(
            LinearLayout::vertical()
                .element(image)
                .element(recipient_info),
        )
        .element(
            LinearLayout::vertical()
                .element(aegee_address)
                .element(other_info)
                .padded((0, 0, 5, 0)),
        )
        .push()
        .unwrap();
    // header
    //     .row()
    //     .element(elements::Paragraph::new(format!("Naam: {}", name)).wrap_small_pad())
    //     .element(elements::Paragraph::new(format!("Datum: {}", date)).wrap_small_pad())
    //     .push()
    //     .unwrap();
    // header
    //     .row()
    //     .element(elements::Paragraph::new(format!("Adres: {}", address)).wrap_small_pad())
    //     .element(elements::Paragraph::new(format!("Factuurnummer: {}", 1)).wrap_small_pad())
    //     .push()
    //     .unwrap();
    let mut doc =
        genpdf::Document::new(genpdf::fonts::from_files("./fonts", "Roboto", None).unwrap());
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(20);
    doc.set_page_decorator(decorator);
    doc.set_title("Factuur");
    doc.push(header);

    let mut table = TableLayout::new(vec![5, 3]);
    table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, true));
    table
        .row()
        .element(
            elements::Paragraph::new("Factuur")
                .styled(genpdf::style::Style::new().with_font_size(28))
                .wrap_small_pad(),
        )
        .element(
            LinearLayout::vertical()
                .element(elements::Paragraph::new("Date of invoice:"))
                .element(elements::Paragraph::new(format!("{}", date)))
                .wrap_small_pad(),
        )
        .push()
        .unwrap();
    table
        .row()
        .element(
            LinearLayout::vertical()
                .element(elements::Paragraph::new("Your Reference:"))
                .element(elements::Paragraph::new(""))
                .wrap_small_pad(),
        )
        .element(
            LinearLayout::vertical()
                .element(elements::Paragraph::new("Our Reference:"))
                .element(elements::Paragraph::new(format!(
                    "F{}-{:0>3}",
                    cfg.year_format(),
                    factuur_number
                )))
                .wrap_small_pad(),
        )
        .push()
        .unwrap();

    doc.push(table.padded((0, 0, 5, 0)));

    let mut table = TableLayout::new(vec![5, 1, 1, 1]);
    table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, true));
    table
        .row()
        .element(elements::Paragraph::new("Omschrijving").wrap_small_pad())
        .element(elements::Paragraph::new("Aantal").wrap_small_pad())
        .element(elements::Paragraph::new("Prijs").wrap_small_pad())
        .element(elements::Paragraph::new("Totaal").wrap_small_pad())
        .push()
        .unwrap();
    for item in items {
        table
            .row()
            .element(elements::Paragraph::new(item.name.clone()).wrap_small_pad())
            .element(elements::Paragraph::new(format!("{}", item.amount)).wrap_small_pad())
            .element(elements::Paragraph::new(format!("{}", item.cost)).wrap_small_pad())
            .element(
                elements::Paragraph::new(format!("{}", item.cost * item.amount)).wrap_small_pad(),
            )
            .push()
            .unwrap();
    }
    table
        .row()
        .element(elements::Paragraph::new("Total").wrap_small_pad())
        .element(elements::Paragraph::new("").wrap_small_pad())
        .element(elements::Paragraph::new("").wrap_small_pad())
        .element(
            elements::Paragraph::new(format!(
                "{}",
                items.iter().map(|i| i.cost * i.amount).sum::<Euro>()
            ))
            .wrap_small_pad(),
        )
        .push()
        .unwrap();
    doc.push(table);
    let mut buf = vec![];
    doc.render(&mut buf).unwrap();
    buf
}

#[derive(Debug)]
enum CurrentState {
    New,
    Used,
    AsNew,
    Custom(String),
}

impl Display for CurrentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CurrentState::New => write!(f, "Nieuw"),
            CurrentState::Used => write!(f, "Gebruikt"),
            CurrentState::AsNew => write!(f, "Zo goed als nieuw"),
            CurrentState::Custom(s) => write!(f, "{}", s),
        }
    }
}

fn contract_generator(
    cfg: &penning_helper_config::Config,
    what: String,
    start_date: Date,
    end_date: Date,
    state: CurrentState,
    loaned_to: String,
    loaned_by: String,
) -> Vec<u8> {
    let mut doc =
        genpdf::Document::new(genpdf::fonts::from_files("./fonts", "Roboto", None).unwrap());
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(20);
    doc.set_page_decorator(decorator);
    doc.set_title("Uitleencontract");
    let image = image::open("logo.png").unwrap();
    let image = image.to_rgb8();
    let w = image.width();
    let max_w = 500;
    let scale_w = max_w as f64 / w as f64;
    let image = image::DynamicImage::ImageRgb8(image);

    let mut layout = genpdf::elements::LinearLayout::vertical();

    let mut table = genpdf::elements::TableLayout::new(vec![2, 1]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(
        false, false, false,
    ));

    table
        .row()
        .element(genpdf::elements::Paragraph::new(format!(
            "Uitleencontract {}",
            cfg.sepa().company_name
        )).styled(genpdf::style::Style::new().with_font_size(24).bold()))
        .element(
            genpdf::elements::Image::from_dynamic_image(image)
                .unwrap()
                .with_alignment(genpdf::Alignment::Right)
                .with_scale((scale_w, scale_w)),
        )
        .push()
        .unwrap();
    layout = layout.element(table);

    let l = elements::LinearLayout::vertical()
        .element(
            elements::Paragraph::new(format!(
                "Hierbij verklaart de ondergetekende dat het volgende eigendom van {}",
                cfg.sepa().company_name
            ))
            .wrap_small_pad(),
        )
        .element({
            let mut layout = elements::TableLayout::new(vec![1]);
            layout
                .row()
                .element(elements::Paragraph::new(what).padded((7, 1)))
                .push()
                .unwrap();
            layout.set_cell_decorator(elements::FrameCellDecorator::new(true, true, true));

            layout
        })
        .element({
            let mut layout = elements::TableLayout::new(vec![1]);
            layout
                .row()
                .element(
                    elements::Paragraph::new(format!("Huidige staat eigendom: {}", state))
                        .padded((7, 1)),
                )
                .push()
                .unwrap();
            layout.set_cell_decorator(elements::FrameCellDecorator::new(true, true, true));

            layout
        })
        .element(elements::Paragraph::new("Van:").wrap_small_pad())
        .element({
            let mut layout = elements::TableLayout::new(vec![1]);
            layout
                .row()
                .element(
                    elements::Paragraph::new(format!("Uitgeleend door: {}", loaned_by))
                        .padded((7, 1)),
                )
                .push()
                .unwrap();
            layout.set_cell_decorator(elements::FrameCellDecorator::new(true, true, true));

            layout
        })
        .element(elements::Paragraph::new("Tussen:").wrap_small_pad())
        .element({
            let mut layout = elements::TableLayout::new(vec![1, 1]);
            layout
                .row()
                .element(elements::Paragraph::new(format!("Startdatum: {}", start_date)).padded((7, 1)))
                .element(elements::Paragraph::new("Starttijd: ").padded((7, 1)))
                .push()
                .unwrap();
            layout
                .row()
                .element(elements::Paragraph::new(format!("Einddatum: {}", end_date)).padded((7, 1)))
                .element(elements::Paragraph::new("Eindtijd: ").padded((7, 1)))
                .push()
                .unwrap();
            layout.set_cell_decorator(elements::FrameCellDecorator::new(true, true, true));

            layout
        })
        .element(elements::Paragraph::new("Aan:").wrap_small_pad())
        .element({
            let mut layout = elements::TableLayout::new(vec![1]);
            layout
                .row()
                .element(
                    elements::Paragraph::new(format!("Uitgeleend aan: {}", loaned_to))
                        .padded((7, 1)),
                )
                .push()
                .unwrap();
            layout.set_cell_decorator(elements::FrameCellDecorator::new(true, true, true));

            layout
        })
        .element(
            
                paragraphs!(
                    "onder de verantwoordelijkheid van de ondergetekende valt.","De einddatum is tevens de datum waarop het eigendom terug wordt gegeven aan AEGEE-Delft.",
                    "Wanneer er schade ontstaat aan het eigendom/de eigendommen in de bovenstaande periode, dan zal de ondergetekende de geleden schade vergoeden.",
                    "Dit gebeurt in overleg en zal in de meeste gevallen neerkomen op:",
                    "    a. Reparatie vergoeden",
                    "    b. Dagwaarde vergoeden",
                    "    c. Het aanbieden/vergoeden van een vergelijkbaar product",
                    "    d. Vermindering in waarde door de geleden schade vergoeden",
                )
        )
        .element({
            let mut layout = elements::TableLayout::new(vec![2, 1]);
            layout
                .row()
                .element(elements::Paragraph::new("Voorletters en Achternaam:").padded((1,1,13,1)))
                .element(elements::Paragraph::new("Handtekening: ").padded((1,1,20,1)))
                .push()
                .unwrap();
            layout.set_cell_decorator(elements::FrameCellDecorator::new(true, true, true));

            layout
        }.padded((3,0)));

    layout = layout.element(l);

    doc.push(layout);
    let mut buf = vec![];
    doc.render(&mut buf).unwrap();
    buf
}

