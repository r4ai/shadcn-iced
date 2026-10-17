use super::Fixture;
use crate::store::Msg;

use shadcn_iced::button::{Variant, icon_button};
use shadcn_iced::prelude::lucide;
use shadcn_iced::table::{Cell, table};

use iced::alignment::Horizontal;
use iced::widget::text;
use iced::{Element, Fill, Length};

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "demo" => Fixture::new(|_| {
            let invoices = [
                ("INV001", "Paid", "Credit Card", "$250.00"),
                ("INV002", "Pending", "PayPal", "$150.00"),
                ("INV003", "Unpaid", "Bank Transfer", "$350.00"),
                ("INV004", "Paid", "Credit Card", "$450.00"),
                ("INV005", "Paid", "PayPal", "$550.00"),
                ("INV006", "Pending", "Bank Transfer", "$200.00"),
                ("INV007", "Unpaid", "Credit Card", "$300.00"),
            ];

            invoices
                .iter()
                .fold(
                    table::<Msg, &str>(["Invoice", "Status", "Method", "Amount"]),
                    |table, (invoice, status, method, amount)| {
                        table.row::<Cell<'_, Msg>>([
                            medium(invoice).into(),
                            (*status).into(),
                            (*method).into(),
                            (*amount).into(),
                        ])
                    },
                )
                // `colSpan={3}`
                .footer(["Total", "", "", "$2,500.00"])
                .caption("A list of your recent invoices.")
                // `w-25`
                .widths([Length::Fixed(100.0), Fill, Fill, Fill])
                .align(3, Horizontal::Right)
                .into()
        }),
        "actions" => Fixture::new(|_| {
            let actions = |product: &'static str, price: &'static str| -> [Cell<'static, Msg>; 3] {
                [
                    medium(product).into(),
                    price.into(),
                    Element::from(
                        icon_button(lucide::ELLIPSIS)
                            .variant(Variant::Ghost)
                            .on_press(Msg::Noop),
                    )
                    .into(),
                ]
            };

            table::<Msg, &str>(["Product", "Price", "Actions"])
                .row(actions("Wireless Mouse", "$29.99"))
                .row(actions("Mechanical Keyboard", "$129.99"))
                .row(actions("USB-C Hub", "$49.99"))
                .align(2, Horizontal::Right)
                .into()
        }),
        _ => return None,
    })
}

/// `font-medium whitespace-nowrap`
fn medium(value: &'static str) -> Element<'static, Msg> {
    text(value)
        .size(14)
        .line_height(iced::Pixels(20.0))
        .wrapping(text::Wrapping::None)
        .font(shadcn_iced::font::medium())
        .into()
}
