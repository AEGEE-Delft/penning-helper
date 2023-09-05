use genpdf::{
    elements::{self, LinearLayout, PaddedElement, Paragraph, TableLayout},
    style::StyledString,
    Element, Margins,
};
use penning_helper_conscribo::{
    AddChangeTransaction, ConscriboClient, ConscriboMultiRequest, UnifiedTransaction,
};

use penning_helper_types::{Address, Date, Euro};

fn main() {
    let config: penning_helper_config::Config =
        toml::from_str(std::fs::read_to_string(".sample.toml").unwrap().as_str()).unwrap();
    // let mail_server = penning_helper_mail::MailServer::new(config.mail(), config.sepa()).unwrap();
    // let (r, pdf_file) = stuff(config.conscribo());
    // mail_server
    //     .send_mail(
    //         "Julius de Jeu",
    //         "julius@asraphiel.dev",
    //         pdf_file,
    //         r,
    //         Date::today(),
    //     )
    //     .unwrap();
    let factuur = factuur_generator(
        &config,
        Address::new(
            "AEGEE-Utrecht",
            "Princetonplein 9",
            "3584CC",
            "Utrecht",
            Some("Netherlands"),
        ),
        Date::today(),
        1,
        &vec![
            FactuurItem::new("TestItem123".to_string(), 10.50.into(), 10.),
            FactuurItem::new("Waluigi".to_string(), 20.0.into(), 2.),
        ],
    );
    std::fs::write("factuur.pdf", factuur).unwrap();
    return;

    let client = ConscriboClient::new_from_cfg(config.conscribo()).unwrap();
    let members = client.get_relations("persoon").unwrap();
    let others = client.get_relations("onbekend").unwrap();
    let all_relations = members
        .into_iter()
        .chain(others.into_iter())
        .collect::<Vec<_>>();
    // let f = std::fs::File::open("bbq.xlsx").unwrap();
    let mut turf_list = penning_helper_turflists::xlsx::read_excel("bbq.xlsx", 6.0.into()).unwrap();
    turf_list.shrink();
    let names = all_relations
        .iter()
        .map(|r| r.naam.clone())
        .collect::<Vec<_>>();
    let emails = all_relations
        .iter()
        .map(|r| r.email_address.clone())
        .collect::<Vec<_>>();
    for entry in turf_list.iter() {
        // println!("{:#?}", entry);
        if entry.iban.is_some() {
            println!("{} already has an iban", entry.name);
            continue;
        }
        let idx = match entry.best_email_match(&emails) {
            Ok(idx) => idx,
            Err(_) => match entry.best_name_match(&names) {
                Ok(idx) => idx,
                Err(_) => {
                    println!("No match found for {}", entry.name);
                    continue;
                }
            },
        };
        println!(
            "matched {} to {} ({} spent)",
            entry.name, all_relations[idx].naam, entry.amount
        );
    }
    let matches = turf_list
        .iter()
        .flat_map(|t| t.best_idx(&names, &emails))
        .map(|(idx, amount)| (&all_relations[idx], amount))
        .collect::<Vec<_>>();
    let transactions = matches
        .iter()
        .map(|(r, eur)| {
            let eur = *eur;
            let a = AddChangeTransaction::new(
                Date::today(),
                "Turfjes Borrels door het jaar heen".to_string(),
            );
            let a = if eur > Euro::default() {
                a.add_debet("5001-10".to_string(), eur, "T2324-01".to_string(), r.code)
            } else if eur < Euro::default() {
                a.add_credit("5001-10".to_string(), eur, "T2324-01".to_string(), r.code)
            } else {
                a
            };
            a
        })
        .collect::<Vec<_>>();
    let message = ConscriboMultiRequest::new(transactions);
    // println!("{:#?}", message);
}

fn stuff(cfg: &penning_helper_config::ConscriboConfig) -> (Euro, Vec<u8>) {
    let client = ConscriboClient::new_from_cfg(cfg).unwrap();
    // println!("{:?}", client);
    // let res = client.get_field_definitions("persoon").unwrap();
    // println!("{:#?}", res);
    let members = client.get_relations("persoon").unwrap();
    let me = members
        .iter()
        .find(|m| m.naam == "Julius de Jeu")
        .unwrap()
        .clone();
    println!("{:#?}", me);
    let unknowns = client.get_relations("onbekend").unwrap();
    let all_relations = members
        .into_iter()
        .chain(unknowns.into_iter())
        .collect::<Vec<_>>();
    let all_relations_no_bank = all_relations
        .iter()
        .filter(|r| r.rekening.is_none())
        .collect::<Vec<_>>();
    for relation in all_relations_no_bank {
        println!("{}", relation.naam);
    }

    let transactions = client
        .get_transactions(Date::new(2022, 9, 1), Date::new(2099, 12, 31))
        .unwrap();

    let transactions = transactions
        .into_iter()
        .filter(|t| t.code == me.code)
        .collect::<Vec<_>>();
    // println!("{:#?}", transactions);
    let r = transactions.iter().map(|t| t.cost).sum::<Euro>();
    println!("{}", r);
    // penning_helper_sepa::gen_xml();
    (r, create_pdf(transactions, &me.naam))
}

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
        format!("AEGEE-Delft owes you {:-}, it will be transferred in about 3 days. If this is wrong please send an email to treasurer@aegee-delft.nl!", total)
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
