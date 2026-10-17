//! The dashboard of a fictional store, after the dashboard example of
//! shadcn/ui (<https://ui.shadcn.com/examples/dashboard>): a top bar, the
//! metrics of the month, the revenue, the recent sales, the visitors and the
//! invoices. Every number is mock data.
//!
//! ```sh
//! cargo run -p dashboard
//! ```
//!
//! The website shows it as a live preview too (`/examples/dashboard`).
use shadcn_iced::avatar;
use shadcn_iced::button::{self, Variant};
use shadcn_iced::chart::{self, Series};
use shadcn_iced::menu::item;
use shadcn_iced::prelude::*;
use shadcn_iced::primitive::Align;
use shadcn_iced::table::Cell;
use shadcn_iced::{badge, font, style, tokens};

use iced::widget::{column, mouse_area, row, space, text};
use iced::{Center, Element, Fill, FillPortion, Theme, alignment, mouse};

/// The teams of the team switcher: a name and its initials.
const TEAMS: [(&str, &str); 3] = [
    ("Alicia Koch", "AK"),
    ("Acme Inc.", "AI"),
    ("Monsters Inc.", "MI"),
];

/// The recent sales: a name, an email and an amount.
const SALES: [(&str, &str, &str); 5] = [
    ("Olivia Martin", "olivia.martin@email.com", "+$1,999.00"),
    ("Jackson Lee", "jackson.lee@email.com", "+$39.00"),
    ("Isabella Nguyen", "isabella.nguyen@email.com", "+$299.00"),
    ("William Kim", "will@email.com", "+$99.00"),
    ("Sofia Davis", "sofia.davis@email.com", "+$39.00"),
];

/// The invoices of the Reports tab: a number, a customer, a status, a
/// payment method and an amount.
const INVOICES: [(&str, &str, Status, &str, &str); 7] = [
    (
        "INV001",
        "Olivia Martin",
        Status::Paid,
        "Credit Card",
        "$250.00",
    ),
    (
        "INV002",
        "Jackson Lee",
        Status::Pending,
        "PayPal",
        "$150.00",
    ),
    (
        "INV003",
        "Isabella Nguyen",
        Status::Unpaid,
        "Bank Transfer",
        "$350.00",
    ),
    (
        "INV004",
        "William Kim",
        Status::Paid,
        "Credit Card",
        "$450.00",
    ),
    ("INV005", "Sofia Davis", Status::Paid, "PayPal", "$550.00"),
    (
        "INV006",
        "Liam Johnson",
        Status::Pending,
        "Bank Transfer",
        "$200.00",
    ),
    (
        "INV007",
        "Emma Brown",
        Status::Unpaid,
        "Credit Card",
        "$300.00",
    ),
];

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const REVENUE: [f32; 12] = [
    4200.0, 3100.0, 5400.0, 2900.0, 4800.0, 3600.0, 5100.0, 4300.0, 2400.0, 3900.0, 5600.0, 4700.0,
];
const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

const BROWSERS: [&str; 5] = ["Chrome", "Safari", "Firefox", "Edge", "Other"];
const VISITORS: [f32; 5] = [275.0, 200.0, 187.0, 173.0, 90.0];

/// The state of the dashboard.
#[derive(Debug, Clone)]
pub struct Dashboard {
    team: usize,
    link: Link,
    search: String,
    period: Period,
    tab: Tab,
    downloaded: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Team(usize),
    Link(Link),
    Search(String),
    Period(Period),
    Tab(Tab),
    Download,
    /// A menu item that does nothing in this mock.
    Noop,
}

/// A link of the top bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Link {
    Overview,
    Customers,
    Products,
    Settings,
}

/// The period of the revenue chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    Week,
    Month,
    Year,
}

impl Period {
    const ALL: [Period; 3] = [Period::Week, Period::Month, Period::Year];

    fn label(self) -> &'static str {
        match self {
            Period::Week => "Last 7 days",
            Period::Month => "Last 30 days",
            Period::Year => "Last 12 months",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Analytics,
    Reports,
    Notifications,
}

/// The status of an invoice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Paid,
    Pending,
    Unpaid,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self {
            team: 0,
            link: Link::Overview,
            search: String::new(),
            period: Period::Year,
            tab: Tab::Overview,
            downloaded: false,
        }
    }
}

impl Dashboard {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Team(team) => self.team = team,
            Message::Link(link) => self.link = link,
            Message::Search(search) => self.search = search,
            Message::Period(period) => self.period = period,
            Message::Tab(tab) => self.tab = tab,
            Message::Download => self.downloaded = true,
            Message::Noop => {}
        }
    }

    /// The dashboard, at its natural height (the app scrolls it).
    pub fn view(&self) -> Element<'_, Message> {
        let title = row![
            text("Dashboard").size(style::text::XL3).font(font::bold()),
            space::horizontal(),
            dropdown_menu(
                button(self.period.label())
                    .icon(lucide::CALENDAR)
                    .variant(Variant::Outline),
                Period::ALL
                    .into_iter()
                    .fold(menu(), |menu, period| {
                        menu.radio(
                            period.label(),
                            period == self.period,
                            Message::Period(period),
                        )
                    })
                    .min_width(180.0),
            )
            .align(Align::End),
            if self.downloaded {
                button("Downloaded").icon(lucide::CHECK)
            } else {
                button("Download").icon(lucide::DOWNLOAD)
            }
            .on_press(Message::Download),
        ]
        .spacing(8)
        .align_y(Center);

        let content = match self.tab {
            Tab::Overview | Tab::Notifications => self.overview(),
            Tab::Analytics => analytics(),
            Tab::Reports => self.reports(),
        };

        let body = column![
            title,
            tabs(self.tab, Message::Tab)
                .tab(Tab::Overview, "Overview")
                .tab(Tab::Analytics, "Analytics")
                .tab(Tab::Reports, "Reports")
                .disabled_tab(Tab::Notifications, "Notifications")
                .content(content),
        ]
        .spacing(16)
        .padding(iced::Padding::new(32.0).top(24.0));

        column![self.top_bar(), separator(), body].into()
    }

    /// The team switcher, the links, the search and the user menu.
    fn top_bar(&self) -> Element<'_, Message> {
        let (team, initials) = TEAMS[self.team];

        let switcher = dropdown_menu(
            button::Button::new(
                row![
                    avatar(initials).size(avatar::Size::Sm),
                    text(team).size(style::text::SM),
                    space::horizontal(),
                    lucide::CHEVRONS_UP_DOWN.sized(16).style(muted),
                ]
                .spacing(8)
                .align_y(Center),
            )
            .variant(Variant::Outline)
            .width(200),
            menu()
                .label("Personal Account")
                .radio(TEAMS[0].0, self.team == 0, Message::Team(0))
                .separator()
                .label("Teams")
                .radio(TEAMS[1].0, self.team == 1, Message::Team(1))
                .radio(TEAMS[2].0, self.team == 2, Message::Team(2))
                .separator()
                .item(
                    item("Create Team")
                        .icon(lucide::CIRCLE_PLUS)
                        .on_select(Message::Noop),
                )
                .min_width(200.0),
        );

        let links = [
            (Link::Overview, "Overview"),
            (Link::Customers, "Customers"),
            (Link::Products, "Products"),
            (Link::Settings, "Settings"),
        ]
        .into_iter()
        .fold(row![].spacing(24), |links, (link, label)| {
            let active = link == self.link;

            links.push(
                mouse_area(
                    text(label)
                        .size(style::text::SM)
                        .font(font::medium())
                        .style(move |theme: &Theme| text::Style {
                            color: (!active).then(|| tokens(theme).palette.muted_foreground),
                        }),
                )
                .interaction(mouse::Interaction::Pointer)
                .on_press(Message::Link(link)),
            )
        });

        let user = dropdown_menu(
            button::Button::new(avatar("SC"))
                .variant(Variant::Ghost)
                .size(button::Size::Icon)
                .rounded(true),
            menu()
                .label("shadcn · m@example.com")
                .separator()
                .item(item("Profile").shortcut("⇧⌘P").on_select(Message::Noop))
                .item(item("Billing").shortcut("⌘B").on_select(Message::Noop))
                .item(item("Settings").shortcut("⌘S").on_select(Message::Noop))
                .item(item("New Team").on_select(Message::Noop))
                .separator()
                .item(item("Log out").shortcut("⇧⌘Q").on_select(Message::Noop))
                .min_width(224.0),
        )
        .align(Align::End);

        row![
            switcher,
            links,
            space::horizontal(),
            input("Search...", &self.search)
                .icon(lucide::SEARCH)
                .on_input(Message::Search)
                .width(260),
            user,
        ]
        .spacing(24)
        .align_y(Center)
        .padding([14, 16])
        .into()
    }

    /// The metrics, the revenue and the recent sales.
    fn overview(&self) -> Element<'_, Message> {
        let metrics = row![
            metric(
                "Total Revenue",
                lucide::DOLLAR_SIGN,
                "$45,231.89",
                "+20.1% from last month"
            ),
            metric(
                "Subscriptions",
                lucide::USERS,
                "+2,350",
                "+180.1% from last month"
            ),
            metric(
                "Sales",
                lucide::CREDIT_CARD,
                "+12,234",
                "+19% from last month"
            ),
            metric(
                "Active Now",
                lucide::ACTIVITY,
                "+573",
                "+201 since last hour"
            ),
        ]
        .spacing(16);

        let (categories, values): (Vec<String>, Vec<f32>) = match self.period {
            Period::Week => DAYS
                .iter()
                .enumerate()
                .map(|(i, day)| (day.to_string(), 120.0 + ((i * 53) % 9) as f32 * 40.0))
                .unzip(),
            Period::Month => (1..=30)
                .map(|day| (format!("{day}"), 40.0 + ((day * 37) % 11) as f32 * 16.0))
                .unzip(),
            Period::Year => MONTHS
                .iter()
                .zip(REVENUE)
                .map(|(month, revenue)| (month.to_string(), revenue))
                .unzip(),
        };

        let revenue = card()
            .title("Overview")
            .description(format!(
                "Revenue of the {}",
                self.period.label().to_lowercase()
            ))
            .content(
                chart::bar(categories)
                    .series(Series::new("Revenue", values))
                    .legend(false)
                    .height(320.0),
            )
            .width(FillPortion(4));

        let query = self.search.to_lowercase();
        let sales: Vec<_> = SALES
            .iter()
            .filter(|(name, email, _)| {
                name.to_lowercase().contains(&query) || email.contains(&query)
            })
            .collect();

        let list: Element<'_, Message> = if sales.is_empty() {
            text("No sales match your search.")
                .size(style::text::SM)
                .style(muted)
                .into()
        } else {
            sales
                .into_iter()
                .fold(column![].spacing(24), |list, (name, email, amount)| {
                    list.push(
                        row![
                            avatar(initials(name)).size(avatar::Size::Lg),
                            column![
                                text(*name).size(style::text::SM).font(font::medium()),
                                text(*email).size(style::text::SM).style(muted),
                            ]
                            .spacing(2),
                            space::horizontal(),
                            text(*amount).font(font::medium()),
                        ]
                        .spacing(16)
                        .align_y(Center),
                    )
                })
                .into()
        };

        let recent = card()
            .title("Recent Sales")
            .description("You made 265 sales this month.")
            .content(list)
            .width(FillPortion(3));

        column![metrics, row![revenue, recent].spacing(16)]
            .spacing(16)
            .into()
    }

    /// The invoices, filtered by the search.
    fn reports(&self) -> Element<'_, Message> {
        let query = self.search.to_lowercase();

        let table = INVOICES
            .iter()
            .filter(|(number, customer, ..)| {
                customer.to_lowercase().contains(&query) || number.to_lowercase().contains(&query)
            })
            .fold(
                table(["Invoice", "Customer", "Status", "Method", "Amount"]),
                |table, (number, customer, status, method, amount)| {
                    let (label, variant) = match status {
                        Status::Paid => ("Paid", badge::Variant::Default),
                        Status::Pending => ("Pending", badge::Variant::Secondary),
                        Status::Unpaid => ("Unpaid", badge::Variant::Destructive),
                    };

                    table.row([
                        Cell::Element(text(*number).font(font::medium()).size(14).into()),
                        Cell::from(*customer),
                        Cell::Element(badge(label).variant(variant).into()),
                        Cell::from(*method),
                        Cell::from(*amount),
                    ])
                },
            )
            .footer(["Total", "", "", "", "$2,250.00"])
            .align(4, alignment::Horizontal::Right);

        card()
            .title("Invoices")
            .description("The invoices of the month. Search them by customer or number.")
            .content(table)
            .width(Fill)
            .into()
    }
}

/// The visitors of the last six months, and their browsers.
fn analytics<'a>() -> Element<'a, Message> {
    let visitors = card()
        .title("Visitors")
        .description("January - June 2024")
        .content(
            chart::area(MONTHS[..6].iter().copied())
                .series(Series::new(
                    "Desktop",
                    [186.0, 305.0, 237.0, 73.0, 209.0, 214.0],
                ))
                .series(Series::new(
                    "Mobile",
                    [80.0, 200.0, 120.0, 190.0, 130.0, 140.0],
                ))
                .stacked(true)
                // Room for the first and the last month under the plot.
                .margin([5, 12])
                .height(300.0),
        )
        .width(FillPortion(4));

    let browsers = card()
        .title("Browsers")
        .description("The browsers of the visitors")
        .content(chart::pie(BROWSERS, VISITORS).donut(true).height(300.0))
        .width(FillPortion(3));

    row![visitors, browsers].spacing(16).into()
}

/// A card with a metric: `text-sm font-medium` title, icon, `text-2xl
/// font-bold` value and `text-xs` change.
fn metric<'a>(
    title: &'a str,
    icon: shadcn_iced::Icon,
    value: &'a str,
    change: &'a str,
) -> Element<'a, Message> {
    card()
        .title_element(text(title).size(style::text::SM).font(font::medium()))
        .action(icon.sized(16).style(muted))
        .content(
            column![
                text(value).size(style::text::XL2).font(font::bold()),
                text(change).size(style::text::XS).style(muted),
            ]
            .spacing(4),
        )
        .width(Fill)
        .into()
}

/// The initials of a name (`Olivia Martin` → `OM`).
fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .collect()
}

fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}
