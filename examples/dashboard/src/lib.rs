//! The dashboard of a fictional company, after the `dashboard-01` block of
//! shadcn/ui (<https://ui.shadcn.com/blocks#dashboard-01>): an inset
//! sidebar, the key metrics, the visitors of the last three months and the
//! sections of a document, each one editable in a sheet. Every number is
//! mock data (see [`data`]).
//!
//! ```sh
//! cargo run -p dashboard
//! ```
//!
//! The website shows it as a live preview too (`/examples/dashboard`).
pub mod data;

use shadcn_iced::button::{self, Variant};
use shadcn_iced::chart::{self, Series};
use shadcn_iced::menu::item as menu_item;
use shadcn_iced::prelude::*;
use shadcn_iced::primitive::{Align, Side};
use shadcn_iced::sidebar::{self, group, item, unlabeled_group};
use shadcn_iced::table::Cell;
use shadcn_iced::{badge, font, style, toggle, tokens};

use iced::widget::{column, container, row, space, text};
use iced::{Center, Color, Element, Fill, Length, Theme};

/// The reviewers that can be assigned to a section.
const REVIEWERS: [&str; 3] = ["Eddie Lake", "Jamik Tashpulatov", "Emily Whalen"];

/// The types of a section.
const TYPES: [&str; 8] = [
    "Table of Contents",
    "Executive Summary",
    "Technical Approach",
    "Design",
    "Capabilities",
    "Focus Documents",
    "Narrative",
    "Cover Page",
];

const STATUSES: [&str; 3] = ["Done", "In Progress", "Not Started"];

/// The state of the dashboard.
#[derive(Debug, Clone)]
pub struct Dashboard {
    collapsed: bool,
    nav: &'static str,
    range: Range,
    view: View,
    sections: Vec<Section>,
    next_id: u32,
    hidden: Vec<TableColumn>,
    page: usize,
    per_page: usize,
    /// The section edited in the sheet, and its form.
    editing: Option<(u32, Form)>,
}

/// A section of the document (a row of the table).
#[derive(Debug, Clone)]
struct Section {
    id: u32,
    header: String,
    kind: String,
    status: String,
    target: String,
    limit: String,
    reviewer: Option<String>,
    selected: bool,
}

/// The form of the sheet that edits a section.
#[derive(Debug, Clone)]
struct Form {
    header: String,
    kind: Option<String>,
    status: Option<String>,
    target: String,
    limit: String,
    reviewer: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleSidebar,
    Nav(&'static str),
    Range(Range),
    View(View),
    ToggleColumn(TableColumn),
    AddSection,
    Select(u32, bool),
    SelectAll(bool),
    Target(u32, String),
    Limit(u32, String),
    Reviewer(u32, String),
    Copy(u32),
    Delete(u32),
    Page(usize),
    PerPage(usize),
    Edit(u32),
    FormHeader(String),
    FormKind(String),
    FormStatus(String),
    FormTarget(String),
    FormLimit(String),
    FormReviewer(String),
    Submit,
    CloseSheet,
    /// A menu item that does nothing in this mock.
    Noop,
}

/// The period of the visitors chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Range {
    Months3,
    Days30,
    Days7,
}

impl Range {
    fn days(self) -> usize {
        match self {
            Range::Months3 => 90,
            Range::Days30 => 30,
            Range::Days7 => 7,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Range::Months3 => "Last 3 months",
            Range::Days30 => "Last 30 days",
            Range::Days7 => "Last 7 days",
        }
    }
}

/// A tab of the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Outline,
    PastPerformance,
    KeyPersonnel,
    FocusDocuments,
}

/// A column of the table that can be hidden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableColumn {
    Type,
    Status,
    Target,
    Limit,
    Reviewer,
}

impl TableColumn {
    const ALL: [TableColumn; 5] = [
        TableColumn::Type,
        TableColumn::Status,
        TableColumn::Target,
        TableColumn::Limit,
        TableColumn::Reviewer,
    ];

    fn label(self) -> &'static str {
        match self {
            TableColumn::Type => "Type",
            TableColumn::Status => "Status",
            TableColumn::Target => "Target",
            TableColumn::Limit => "Limit",
            TableColumn::Reviewer => "Reviewer",
        }
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        let sections: Vec<Section> = data::SECTIONS
            .iter()
            .enumerate()
            .map(
                |(index, (header, kind, status, target, limit, reviewer))| Section {
                    id: index as u32 + 1,
                    header: header.to_string(),
                    kind: kind.to_string(),
                    status: status.to_string(),
                    target: target.to_string(),
                    limit: limit.to_string(),
                    reviewer: (*reviewer != "Assign reviewer").then(|| reviewer.to_string()),
                    selected: false,
                },
            )
            .collect();

        Self {
            collapsed: false,
            nav: "Dashboard",
            range: Range::Months3,
            view: View::Outline,
            next_id: sections.len() as u32 + 1,
            sections,
            hidden: Vec::new(),
            page: 0,
            per_page: 10,
            editing: None,
        }
    }
}

impl Dashboard {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleSidebar => self.collapsed = !self.collapsed,
            Message::Nav(nav) => self.nav = nav,
            Message::Range(range) => self.range = range,
            Message::View(view) => self.view = view,
            Message::ToggleColumn(column) => {
                if self.hidden.contains(&column) {
                    self.hidden.retain(|c| *c != column);
                } else {
                    self.hidden.push(column);
                }
            }
            Message::AddSection => {
                self.sections.push(Section {
                    id: self.next_id,
                    header: "New section".to_owned(),
                    kind: "Narrative".to_owned(),
                    status: "Not Started".to_owned(),
                    target: "0".to_owned(),
                    limit: "0".to_owned(),
                    reviewer: None,
                    selected: false,
                });
                self.next_id += 1;
                self.page = self.pages() - 1;
            }
            Message::Select(id, selected) => {
                if let Some(section) = self.section(id) {
                    section.selected = selected;
                }
            }
            Message::SelectAll(selected) => {
                for section in &mut self.sections {
                    section.selected = selected;
                }
            }
            Message::Target(id, value) => {
                if let Some(section) = self.section(id) {
                    section.target = value;
                }
            }
            Message::Limit(id, value) => {
                if let Some(section) = self.section(id) {
                    section.limit = value;
                }
            }
            Message::Reviewer(id, reviewer) => {
                if let Some(section) = self.section(id) {
                    section.reviewer = Some(reviewer);
                }
            }
            Message::Copy(id) => {
                if let Some(index) = self.sections.iter().position(|s| s.id == id) {
                    let mut copy = self.sections[index].clone();
                    copy.id = self.next_id;
                    copy.header = format!("{} (copy)", copy.header);
                    copy.selected = false;
                    self.next_id += 1;
                    self.sections.insert(index + 1, copy);
                }
            }
            Message::Delete(id) => {
                self.sections.retain(|s| s.id != id);
                self.page = self.page.min(self.pages() - 1);
            }
            Message::Page(page) => self.page = page.min(self.pages() - 1),
            Message::PerPage(per_page) => {
                self.per_page = per_page;
                self.page = 0;
            }
            Message::Edit(id) => {
                if let Some(section) = self.sections.iter().find(|s| s.id == id) {
                    self.editing = Some((
                        id,
                        Form {
                            header: section.header.clone(),
                            kind: Some(section.kind.clone()),
                            status: Some(section.status.clone()),
                            target: section.target.clone(),
                            limit: section.limit.clone(),
                            reviewer: section.reviewer.clone(),
                        },
                    ));
                }
            }
            Message::FormHeader(value) => self.form(|form| form.header = value),
            Message::FormKind(value) => self.form(|form| form.kind = Some(value)),
            Message::FormStatus(value) => self.form(|form| form.status = Some(value)),
            Message::FormTarget(value) => self.form(|form| form.target = value),
            Message::FormLimit(value) => self.form(|form| form.limit = value),
            Message::FormReviewer(value) => self.form(|form| form.reviewer = Some(value)),
            Message::Submit => {
                if let Some((id, form)) = self.editing.take()
                    && let Some(section) = self.section(id)
                {
                    section.header = form.header;
                    section.kind = form.kind.unwrap_or_default();
                    section.status = form.status.unwrap_or_default();
                    section.target = form.target;
                    section.limit = form.limit;
                    section.reviewer = form.reviewer;
                }
            }
            Message::CloseSheet => self.editing = None,
            Message::Noop => {}
        }
    }

    fn section(&mut self, id: u32) -> Option<&mut Section> {
        self.sections.iter_mut().find(|s| s.id == id)
    }

    fn form(&mut self, edit: impl FnOnce(&mut Form)) {
        if let Some((_, form)) = &mut self.editing {
            edit(form);
        }
    }

    fn pages(&self) -> usize {
        self.sections.len().div_ceil(self.per_page).max(1)
    }

    /// The dashboard, filling the window: the main content scrolls.
    pub fn view(&self) -> Element<'_, Message> {
        let content = column![self.section_cards(), self.visitors(), self.table(),]
            .spacing(24)
            .padding([24, 24]);

        let main = column![
            self.site_header(),
            separator(),
            scroll_area(content).height(Fill),
        ];

        let layout = sidebar::layout(self.sidebar(), main);

        let Some((_, form)) = &self.editing else {
            return sheet(layout, false).into();
        };

        sheet(layout, true)
            .title(form.header.clone())
            .description("Showing total visitors for the last 6 months")
            .content(scroll_area(edit_form(form)).height(Fill))
            .footer(
                column![
                    button("Submit").width(Fill).on_press(Message::Submit),
                    button("Done")
                        .variant(Variant::Outline)
                        .width(Fill)
                        .on_press(Message::CloseSheet),
                ]
                .spacing(8),
            )
            .on_close(Message::CloseSheet)
            .size(420.0)
            .into()
    }

    /// The brand, the quick actions, the navigation, the documents, the
    /// secondary links and the user.
    fn sidebar(&self) -> sidebar::Sidebar<'_, Message> {
        let nav = |label: &'static str, icon| {
            item(label)
                .icon(icon)
                .active(self.nav == label)
                .on_press(Message::Nav(label))
        };

        let document = |label: &'static str, icon| {
            nav(label, icon).action(
                dropdown_menu(
                    container(lucide::ELLIPSIS.sized(16)).center(20),
                    menu()
                        .item(
                            menu_item("Open")
                                .icon(lucide::FOLDER)
                                .on_select(Message::Noop),
                        )
                        .item(
                            menu_item("Share")
                                .icon(lucide::SHARE)
                                .on_select(Message::Noop),
                        )
                        .separator()
                        .item(
                            menu_item("Delete")
                                .icon(lucide::TRASH_2)
                                .destructive()
                                .on_select(Message::Noop),
                        )
                        .min_width(96.0),
                )
                .side(Side::Right)
                .align(Align::Start),
            )
        };

        let main = unlabeled_group()
            .item(nav("Dashboard", lucide::LAYOUT_DASHBOARD))
            .item(nav("Lifecycle", lucide::LIST_CHECKS))
            .item(nav("Analytics", lucide::CHART_COLUMN))
            .item(nav("Projects", lucide::FOLDER))
            .item(nav("Team", lucide::USERS));

        let documents = group("Documents")
            .item(document("Data Library", lucide::DATABASE))
            .item(document("Reports", lucide::FILE_CHART_COLUMN))
            .item(document("Word Assistant", lucide::FILE_TEXT))
            .item(nav("More", lucide::ELLIPSIS));

        let secondary = [
            ("Settings", lucide::SETTINGS),
            ("Get Help", lucide::CIRCLE_HELP),
            ("Search", lucide::SEARCH),
        ];

        let sidebar = sidebar::sidebar()
            .variant(sidebar::Variant::Inset)
            .width(288.0)
            .collapsed(self.collapsed)
            // `Ctrl`/`Cmd`+`B`
            .on_toggle(|_| Message::ToggleSidebar);

        if self.collapsed {
            // Icons only: the secondary links join the groups.
            let secondary = secondary
                .into_iter()
                .fold(unlabeled_group(), |group, (label, icon)| {
                    group.item(nav(label, icon))
                });

            return sidebar
                .header_item(
                    item("Acme Inc.")
                        .icon(lucide::COMMAND)
                        .on_press(Message::Nav("Dashboard")),
                )
                .group(main)
                .group(documents)
                .group(secondary)
                .footer_item(
                    item("shadcn")
                        .icon_element(avatar("CN"))
                        .on_press(Message::Noop),
                );
        }

        let brand = sidebar_button(
            row![
                lucide::COMMAND.sized(20),
                text("Acme Inc.")
                    .size(style::text::BASE)
                    .font(font::semibold()),
            ]
            .spacing(8)
            .align_y(Center),
            Message::Nav("Dashboard"),
        );

        let quick_create = row![
            button("Quick Create")
                .icon(lucide::CIRCLE_PLUS)
                .width(Fill)
                .on_press(Message::AddSection),
            icon_button(lucide::MAIL)
                .variant(Variant::Outline)
                .size(button::Size::Icon)
                .on_press(Message::Noop),
        ]
        .spacing(8)
        .align_y(Center);

        // `NavSecondary` (`mt-auto`) and `NavUser`, at the bottom.
        let secondary =
            secondary
                .into_iter()
                .fold(iced::widget::Column::new(), |links, (label, icon)| {
                    links.push(sidebar_button(
                        row![icon.sized(16), text(label).size(style::text::SM)]
                            .spacing(8)
                            .align_y(Center),
                        Message::Nav(label),
                    ))
                });

        let user = dropdown_menu(
            sidebar_button(
                row![
                    avatar("CN"),
                    column![
                        text("shadcn").size(style::text::SM).font(font::medium()),
                        text("m@example.com").size(style::text::XS).style(muted),
                    ],
                    space::horizontal(),
                    lucide::ELLIPSIS_VERTICAL.sized(16),
                ]
                .spacing(8)
                .align_y(Center),
                Message::Noop,
            ),
            menu()
                .label("shadcn · m@example.com")
                .separator()
                .item(
                    menu_item("Account")
                        .icon(lucide::CIRCLE_USER)
                        .on_select(Message::Noop),
                )
                .item(
                    menu_item("Billing")
                        .icon(lucide::CREDIT_CARD)
                        .on_select(Message::Noop),
                )
                .item(
                    menu_item("Notifications")
                        .icon(lucide::BELL)
                        .on_select(Message::Noop),
                )
                .separator()
                .item(
                    menu_item("Log out")
                        .icon(lucide::LOG_OUT)
                        .on_select(Message::Noop),
                )
                .min_width(224.0),
        )
        .side(Side::Right)
        .align(Align::End);

        sidebar
            .header(column![brand, quick_create].spacing(8))
            .group(main)
            .group(documents)
            .footer(column![secondary, user].spacing(8))
    }

    /// The trigger of the sidebar and the title of the page.
    fn site_header(&self) -> Element<'_, Message> {
        row![
            sidebar::trigger(Message::ToggleSidebar),
            container(vertical_separator()).height(16).padding([0, 4]),
            text("Documents")
                .size(style::text::BASE)
                .font(font::medium()),
            space::horizontal(),
            button("GitHub")
                .variant(Variant::Ghost)
                .size(button::Size::Sm)
                .on_press(Message::Noop),
        ]
        .spacing(4)
        .align_y(Center)
        .height(48)
        .padding([0, 20])
        .into()
    }

    fn section_cards(&self) -> Element<'_, Message> {
        row![
            section_card(
                "Total Revenue",
                "$1,250.00",
                "+12.5%",
                true,
                "Trending up this month",
                "Visitors for the last 6 months",
            ),
            section_card(
                "New Customers",
                "1,234",
                "-20%",
                false,
                "Down 20% this period",
                "Acquisition needs attention",
            ),
            section_card(
                "Active Accounts",
                "45,678",
                "+12.5%",
                true,
                "Strong user retention",
                "Engagement exceed targets",
            ),
            section_card(
                "Growth Rate",
                "4.5%",
                "+4.5%",
                true,
                "Steady performance increase",
                "Meets growth projections",
            ),
        ]
        .spacing(16)
        .into()
    }

    /// The visitors of the selected period, desktop over mobile.
    fn visitors(&self) -> Element<'_, Message> {
        // From the reference day (June 30) back `days`, both included.
        let days = &data::VISITORS[data::VISITORS.len() - self.range.days() - 1..];

        let range = [Range::Months3, Range::Days30, Range::Days7]
            .into_iter()
            .fold(
                toggle_group(&[self.range], Message::Range)
                    .variant(toggle::Variant::Outline)
                    .spacing(0.0),
                |group, range| group.label(range, range.label()),
            );

        card()
            .title("Total Visitors")
            .description(format!(
                "Total for the {}",
                self.range.label().to_lowercase()
            ))
            .action(range)
            .content(
                chart::area(days.iter().map(|(day, ..)| *day))
                    .series(Series::new("Mobile", days.iter().map(|(_, _, m)| *m)))
                    .series(Series::new("Desktop", days.iter().map(|(_, d, _)| *d)))
                    .stacked(true)
                    .legend(false)
                    .margin([5, 12])
                    .height(250.0),
            )
            .width(Fill)
            .into()
    }

    /// The tabs, the column and section buttons, the sections and the
    /// pagination.
    fn table(&self) -> Element<'_, Message> {
        let customize = TableColumn::ALL.into_iter().fold(menu(), |menu, column| {
            menu.checkbox(
                column.label(),
                !self.hidden.contains(&column),
                Message::ToggleColumn(column),
            )
        });

        let toolbar = row![
            tabs(self.view, Message::View)
                .tab(View::Outline, "Outline")
                .tab(View::PastPerformance, "Past Performance")
                .tab(View::KeyPersonnel, "Key Personnel")
                .tab(View::FocusDocuments, "Focus Documents"),
            space::horizontal(),
            dropdown_menu(
                button("Customize Columns")
                    .icon(lucide::COLUMNS_3)
                    .trailing_icon(lucide::CHEVRON_DOWN)
                    .variant(Variant::Outline)
                    .size(button::Size::Sm),
                customize.min_width(224.0),
            )
            .align(Align::End),
            button("Add Section")
                .icon(lucide::PLUS)
                .variant(Variant::Outline)
                .size(button::Size::Sm)
                .on_press(Message::AddSection),
        ]
        .spacing(8)
        .align_y(Center);

        let content: Element<'_, Message> = if self.view == View::Outline {
            column![self.outline(), self.pagination()]
                .spacing(16)
                .into()
        } else {
            // `aspect-video rounded-lg border border-dashed`
            container(space())
                .width(Fill)
                .height(360)
                .style(|theme: &Theme| {
                    let t = tokens(theme);
                    container::Style {
                        border: style::border(t.palette.border, t.radius_lg()),
                        ..Default::default()
                    }
                })
                .into()
        };

        column![toolbar, content].spacing(24).into()
    }

    /// The sections of the current page.
    fn outline(&self) -> Element<'_, Message> {
        let shown = |column| !self.hidden.contains(&column);
        let all = !self.sections.is_empty() && self.sections.iter().all(|s| s.selected);

        let mut headers: Vec<Cell<'_, Message>> = vec![
            Cell::from(""),
            Cell::Element(checkbox(all).on_toggle(Message::SelectAll).into()),
            Cell::from("Header"),
        ];
        for column in TableColumn::ALL {
            if shown(column) {
                headers.push(Cell::from(match column {
                    TableColumn::Type => "Section Type",
                    other => other.label(),
                }));
            }
        }
        headers.push(Cell::from(""));

        let rows = self
            .sections
            .iter()
            .skip(self.page * self.per_page)
            .take(self.per_page);

        let table = rows.fold(table(headers), |table, section| {
            let id = section.id;

            let mut cells: Vec<Cell<'_, Message>> = vec![
                Cell::Element(
                    container(lucide::GRIP_VERTICAL.sized(12).style(muted))
                        .center(28)
                        .into(),
                ),
                Cell::Element(
                    checkbox(section.selected)
                        .on_toggle(move |selected| Message::Select(id, selected))
                        .into(),
                ),
                Cell::Element(
                    button(section.header.clone())
                        .variant(Variant::Link)
                        .padding_x(0.0)
                        .on_press(Message::Edit(id))
                        .into(),
                ),
            ];

            for column in TableColumn::ALL.into_iter().filter(|c| shown(*c)) {
                cells.push(Cell::Element(match column {
                    TableColumn::Type => badge(section.kind.clone())
                        .variant(badge::Variant::Outline)
                        .into(),
                    TableColumn::Status => status(&section.status),
                    TableColumn::Target => input("", &section.target)
                        .on_input(move |value| Message::Target(id, value))
                        .width(64)
                        .into(),
                    TableColumn::Limit => input("", &section.limit)
                        .on_input(move |value| Message::Limit(id, value))
                        .width(64)
                        .into(),
                    TableColumn::Reviewer => match &section.reviewer {
                        Some(reviewer) => text(reviewer.clone()).size(style::text::SM).into(),
                        None => select(None, move |reviewer: String| {
                            Message::Reviewer(id, reviewer)
                        })
                        .placeholder("Assign reviewer")
                        .options(REVIEWERS[..2].iter().map(|r| r.to_string()))
                        .size(shadcn_iced::select::Size::Sm)
                        .width(160)
                        .into(),
                    },
                }));
            }

            cells.push(Cell::Element(
                dropdown_menu(
                    icon_button(lucide::ELLIPSIS_VERTICAL)
                        .variant(Variant::Ghost)
                        .size(button::Size::IconSm),
                    menu()
                        .item(menu_item("Edit").on_select(Message::Edit(id)))
                        .item(menu_item("Make a copy").on_select(Message::Copy(id)))
                        .item(menu_item("Favorite").on_select(Message::Noop))
                        .separator()
                        .item(
                            menu_item("Delete")
                                .destructive()
                                .on_select(Message::Delete(id)),
                        )
                        .min_width(128.0),
                )
                .align(Align::End)
                .into(),
            ));

            table.row(cells)
        });

        // `overflow-hidden rounded-lg border`
        container(table)
            .padding([0, 8])
            .clip(true)
            .style(|theme: &Theme| {
                let t = tokens(theme);
                container::Style {
                    border: style::border(t.palette.border, t.radius_lg()),
                    ..Default::default()
                }
            })
            .into()
    }

    fn pagination(&self) -> Element<'_, Message> {
        let selected = self.sections.iter().filter(|s| s.selected).count();
        let pages = self.pages();
        let page = self.page;

        let nav = |icon, target: usize, enabled: bool| {
            icon_button(icon)
                .variant(Variant::Outline)
                .size(button::Size::IconSm)
                .on_press_maybe(enabled.then_some(Message::Page(target)))
        };

        row![
            text(format!(
                "{selected} of {} row(s) selected.",
                self.sections.len()
            ))
            .size(style::text::SM)
            .style(muted),
            space::horizontal(),
            row![
                text("Rows per page")
                    .size(style::text::SM)
                    .font(font::medium()),
                select(Some(self.per_page), Message::PerPage)
                    .options([10, 20, 30, 40, 50])
                    .size(shadcn_iced::select::Size::Sm)
                    .width(80),
            ]
            .spacing(8)
            .align_y(Center),
            text(format!("Page {} of {pages}", page + 1))
                .size(style::text::SM)
                .font(font::medium()),
            row![
                nav(lucide::CHEVRONS_LEFT, 0, page > 0),
                nav(lucide::CHEVRON_LEFT, page.saturating_sub(1), page > 0),
                nav(lucide::CHEVRON_RIGHT, page + 1, page + 1 < pages),
                nav(lucide::CHEVRONS_RIGHT, pages - 1, page + 1 < pages),
            ]
            .spacing(8),
        ]
        .spacing(32)
        .align_y(Center)
        .padding([0, 16])
        .into()
    }
}

/// A metric: its name, value and change, and two lines about it.
fn section_card<'a>(
    description: &'a str,
    value: &'a str,
    change: &'a str,
    up: bool,
    trend: &'a str,
    note: &'a str,
) -> Element<'a, Message> {
    let icon = if up {
        lucide::TRENDING_UP
    } else {
        lucide::TRENDING_DOWN
    };

    card()
        .title_element(
            column![
                text(description).size(style::text::SM).style(muted),
                text(value).size(style::text::XL3).font(font::semibold()),
            ]
            .spacing(6),
        )
        .action(badge(change).variant(badge::Variant::Outline).icon(icon))
        .footer(
            column![
                row![
                    text(trend).size(style::text::SM).font(font::medium()),
                    icon.sized(16),
                ]
                .spacing(8)
                .align_y(Center),
                text(note).size(style::text::SM).style(muted),
            ]
            .spacing(6),
        )
        .width(Fill)
        .into()
}

/// The status of a section: a green check when done, a loader otherwise.
fn status<'a>(status: &str) -> Element<'a, Message> {
    let done = status == "Done";

    let icon = if done {
        lucide::CIRCLE_CHECK.sized(12).style(|theme: &Theme| {
            let dark = tokens(theme).palette.background.relative_luminance() < 0.5;
            text::Style {
                // `fill-green-500 dark:fill-green-400`
                color: Some(if dark {
                    Color::from_rgb8(0x4a, 0xde, 0x80)
                } else {
                    Color::from_rgb8(0x22, 0xc5, 0x5e)
                }),
            }
        })
    } else {
        lucide::LOADER.sized(12).style(muted)
    };

    badge::Badge::new(
        row![
            icon,
            text(status.to_owned())
                .size(style::text::XS)
                .font(font::medium())
                .style(muted),
        ]
        .spacing(4)
        .align_y(Center),
    )
    .variant(badge::Variant::Outline)
    .into()
}

/// The chart and the fields of a section, in the sheet.
fn edit_form(form: &Form) -> Element<'_, Message> {
    const MONTHS: [&str; 6] = ["January", "February", "March", "April", "May", "June"];

    let chart = chart::area(MONTHS)
        .series(Series::new(
            "Mobile",
            [80.0, 200.0, 120.0, 190.0, 130.0, 140.0],
        ))
        .series(Series::new(
            "Desktop",
            [186.0, 305.0, 237.0, 73.0, 209.0, 214.0],
        ))
        .stacked(true)
        .legend(false)
        .margin([5, 12])
        .tick_formatter(|month| month[..3].to_owned())
        .height(200.0);

    let select_field = |label: &'static str,
                        placeholder: &'static str,
                        value: &Option<String>,
                        options: &[&str],
                        on_select: fn(String) -> Message| {
        field(
            label,
            select(value.clone(), on_select)
                .placeholder(placeholder)
                .options(options.iter().map(|o| o.to_string()))
                .width(Fill),
        )
    };

    column![
        chart,
        separator(),
        row![
            text("Trending up by 5.2% this month").font(font::medium()),
            lucide::TRENDING_UP.sized(16),
        ]
        .spacing(8)
        .align_y(Center),
        text(
            "Showing total visitors for the last 6 months. This is just some random text \
             to test the layout. It spans multiple lines and should wrap around."
        )
        .size(style::text::SM)
        .style(muted),
        separator(),
        field(
            "Header",
            input("", &form.header).on_input(Message::FormHeader)
        ),
        row![
            select_field(
                "Type",
                "Select a type",
                &form.kind,
                &TYPES,
                Message::FormKind
            ),
            select_field(
                "Status",
                "Select a status",
                &form.status,
                &STATUSES,
                Message::FormStatus
            ),
        ]
        .spacing(16),
        row![
            field(
                "Target",
                input("", &form.target).on_input(Message::FormTarget)
            ),
            field("Limit", input("", &form.limit).on_input(Message::FormLimit)),
        ]
        .spacing(16),
        select_field(
            "Reviewer",
            "Select a reviewer",
            &form.reviewer,
            &REVIEWERS,
            Message::FormReviewer
        ),
    ]
    .spacing(16)
    .width(Length::Fill)
    .into()
}

/// A full-width button of the sidebar (`SidebarMenuButton`).
fn sidebar_button<'a>(
    content: impl Into<Element<'a, Message>>,
    on_press: Message,
) -> Element<'a, Message> {
    iced::widget::button(content)
        .padding([6, 8])
        .width(Fill)
        .on_press(on_press)
        .style(|theme, status| sidebar::button_style(theme, status, false))
        .into()
}

fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).palette.muted_foreground),
    }
}
