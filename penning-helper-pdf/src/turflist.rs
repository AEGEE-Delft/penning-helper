use std::path::PathBuf;

use genpdf::{elements::FrameCellDecorator, style::StyledString, Element, Margins};
use penning_helper_turflists::turflist::TurfListRow;
use rand::Rng;

use crate::FONT_FAMILY;

pub fn generate_turflist_pdf(
    data: &[(&str, &TurfListRow)],
    desription: &str,
    reference: &str,
) -> PathBuf {
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
    doc.push(genpdf::elements::Paragraph::new(StyledString::new(
        desription,
        genpdf::style::Style::new().with_font_size(20).bold(),
    )));
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

    for (who, t) in data {
        table
            .row()
            .element(genpdf::elements::PaddedElement::new(
                genpdf::elements::Paragraph::new(StyledString::new(
                    *who,
                    genpdf::style::Style::new(),
                )),
                Margins::trbl(1, 1, 1, 1),
            ))
            .element(genpdf::elements::PaddedElement::new(
                genpdf::elements::Paragraph::new(StyledString::new(
                    t.what
                        .as_ref()
                        .map(String::as_str)
                        .unwrap_or_else(|| &desription),
                    genpdf::style::Style::new(),
                )),
                Margins::trbl(1, 1, 1, 1),
            ))
            .element(genpdf::elements::PaddedElement::new(
                genpdf::elements::Paragraph::new(StyledString::new(
                    t.amount.to_string(),
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
