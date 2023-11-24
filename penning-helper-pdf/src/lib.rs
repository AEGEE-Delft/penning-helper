use genpdf::{
    elements::{LinearLayout, PaddedElement},
    fonts::{FontData, FontFamily},
    style::StyledString,
    Element, Margins,
};
use once_cell::sync::Lazy;
use penning_helper_types::{Date, Euro};

mod turflist;

pub use turflist::generate_turflist_pdf;

const FONTS: [&[u8]; 4] = [
    include_bytes!("../../fonts/Roboto-Regular.ttf"),
    include_bytes!("../../fonts/Roboto-Bold.ttf"),
    include_bytes!("../../fonts/Roboto-Italic.ttf"),
    include_bytes!("../../fonts/Roboto-BoldItalic.ttf"),
];

static FONT_FAMILY: Lazy<FontFamily<FontData>> = Lazy::new(|| FontFamily {
    regular: FontData::new(FONTS[0].to_vec(), None).unwrap(),
    bold: FontData::new(FONTS[1].to_vec(), None).unwrap(),
    italic: FontData::new(FONTS[2].to_vec(), None).unwrap(),
    bold_italic: FontData::new(FONTS[3].to_vec(), None).unwrap(),
});
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

#[derive(Debug, Clone, Copy)]
pub struct SimpleTransaction<'a> {
    cost: Euro,
    description: &'a str,
    date: Date,
}

impl<'a> SimpleTransaction<'a> {
    pub fn new(cost: Euro, description: &'a str, date: Date) -> Self {
        Self {
            cost,
            description,
            date,
        }
    }
}

pub fn create_invoice_pdf(transactions: Vec<SimpleTransaction>, name: &str) -> Vec<u8> {
    let total = transactions.iter().map(|t| t.cost).sum::<Euro>();
    let font = genpdf::fonts::from_files("./fonts", "Roboto", None).unwrap();
    let mut doc = genpdf::Document::new(font);
    doc.set_title("AEGEE-Delft Invoice");
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
    let logo = include_bytes!("../../penning-helper-mail/logo.png");
    let image = image::load_from_memory(logo).unwrap();
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

    // let text = if total == Euro::default() {
    //     format!("You are even with AEGEE-Delft, no action is required.")
    // } else if total > Euro::default() {
    //     format!("You owe AEGEE-Delft {:-}, it will be deducted in about 3 days. If this is wrong please send an email to treasurer@aegee-delft.nl!", total)
    // } else {
    //     format!("AEGEE-Delft owes you {:-}, it will be transferred in about 6 days. If this is wrong please send an email to treasurer@aegee-delft.nl!", total)
    // };

    // doc.push(genpdf::elements::Paragraph::new(text).padded((1, 1, 5, 1)));

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
