use std::path::PathBuf;

use genpdf::{elements::FrameCellDecorator, style::StyledString, Element, Margins};
use penning_helper_turflists::turflist::TurfListRow;
use penning_helper_types::Euro;
use rand::Rng;

use crate::FONT_FAMILY;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleTurfRow<'s> {
    who: &'s str,
    what: &'s str,
    total: Euro,
}

impl<'s> From<(&'s str, &'s TurfListRow)> for SimpleTurfRow<'s> {
    fn from((who, row): (&'s str, &'s TurfListRow)) -> Self {
        Self {
            who,
            what: row.what.as_ref().map(String::as_str).unwrap_or(""),
            total: row.amount,
        }
    }
}

impl<'s> SimpleTurfRow<'s> {
    pub fn new(who: &'s str, what: &'s str, total: Euro) -> Self {
        Self { who, what, total }
    }
}

pub fn generate_turflist_pdf<'a>(
    data: Vec<impl Into<SimpleTurfRow<'a>>>,
    desription: &str,
    reference: &str,
) -> PathBuf {
    let data: Vec<SimpleTurfRow> = data.into_iter().map(|d| d.into()).collect::<Vec<_>>();
    let mut doc = genpdf::Document::new(FONT_FAMILY.clone());
    doc.set_title(desription);
    doc.set_line_spacing(1.2);
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(20);
    let re = reference.to_string();
    decorator.set_header(move |h| {
        let re = re.clone();
        let mut table = genpdf::elements::TableLayout::new(vec![1, 1]);
        table
            .row()
            .element(
                genpdf::elements::Paragraph::new(StyledString::new(
                    format!("Page {}", h),
                    genpdf::style::Style::new().with_font_size(8),
                ))
                .padded(5),
            )
            .element(
                genpdf::elements::Paragraph::new(StyledString::new(
                    re,
                    genpdf::style::Style::new().with_font_size(8),
                ))
                .aligned(genpdf::Alignment::Right)
                .padded(5),
            )
            .push()
            .unwrap();
        table
    });
    doc.set_page_decorator(decorator);
    doc.push(genpdf::elements::PaddedElement::new(
        genpdf::elements::Paragraph::new(StyledString::new(
            desription,
            genpdf::style::Style::new().with_font_size(20).bold(),
        )),
        Margins::trbl(0, 0, 5, 0),
    ));
    // doc.push(genpdf::elements::Paragraph::new(StyledString::new(
    //     reference,
    //     genpdf::style::Style::new().with_font_size(10),
    // )));

    let mut table = genpdf::elements::TableLayout::new(vec![1, 2, 1]);
    table.set_cell_decorator(FrameCellDecorator::new(true, true, true));
    table
        .row()
        .element(genpdf::elements::PaddedElement::new(
            genpdf::elements::Paragraph::new(StyledString::new(
                "Who",
                genpdf::style::Style::new().bold(),
            )),
            Margins::trbl(1, 1, 5, 1),
        ))
        .element(genpdf::elements::PaddedElement::new(
            genpdf::elements::Paragraph::new(StyledString::new(
                "What",
                genpdf::style::Style::new().bold(),
            )),
            Margins::trbl(1, 1, 5, 1),
        ))
        .element(genpdf::elements::PaddedElement::new(
            genpdf::elements::Paragraph::new(StyledString::new(
                "total",
                genpdf::style::Style::new().bold(),
            )),
            Margins::trbl(1, 1, 5, 1),
        ))
        .push()
        .unwrap();

    for r in data {
        table
            .row()
            .element(genpdf::elements::PaddedElement::new(
                genpdf::elements::Paragraph::new(StyledString::new(
                    r.who,
                    genpdf::style::Style::new(),
                )),
                Margins::trbl(1, 1, 1, 1),
            ))
            .element(genpdf::elements::PaddedElement::new(
                genpdf::elements::Paragraph::new(StyledString::new(
                    r.what,
                    genpdf::style::Style::new(),
                )),
                Margins::trbl(1, 1, 1, 1),
            ))
            .element(genpdf::elements::PaddedElement::new(
                genpdf::elements::Paragraph::new(StyledString::new(
                    r.total.to_string(),
                    genpdf::style::Style::new(),
                )),
                Margins::trbl(1, 1, 1, 1),
            ))
            .push()
            .unwrap();
    }
    doc.push(table);

    let mut temp_file = std::env::temp_dir();
    let mut rng = rand::thread_rng();
    let random_name: String = std::iter::repeat(())
        .map(|()| rng.sample(rand::distributions::Alphanumeric) as char)
        .take(10)
        .collect();
    temp_file.push(random_name);
    temp_file.set_extension("pdf");
    doc.render_to_file(&temp_file).unwrap();
    temp_file
}
