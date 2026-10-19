//! A multi-step questionnaire: one question at a time, with choices, an
//! optional freeform answer, progress and navigation.
//!
//! Port of <https://ui.shadcn.com/docs/components/questionnaire>. The
//! questions are data, and the answers live in a [`State`] that you keep in
//! your application and update with the [`Event`]s of the view (the model
//! of [`shadcn_iced_core::primitive::questionnaire`]):
//!
//! ```no_run
//! # use shadcn_iced::questionnaire::{questionnaire, Event, Question, State};
//! # #[derive(Clone)] enum Message { Questionnaire(Event) }
//! # fn view<'a>(questions: &'a [Question], state: &'a State) -> iced::Element<'a, Message> {
//! questionnaire(questions, state, Message::Questionnaire).into()
//! # }
//! # fn update(questions: &[Question], state: &mut State, event: Event) {
//! if state.update(questions, event) {
//!     // Submitted: read `state.answers()`.
//! }
//! # }
//! ```
use super::button::{Variant as ButtonVariant, button};

pub use shadcn_iced_core::primitive::questionnaire::{Choice, Event, Question, State};

use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::primitive::Pressable;
use shadcn_iced_core::primitive::typeset::{TABULAR_NUMS, typeset};
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::button::Status;
use iced::widget::{Column, Row, container, space, text};
use iced::{Alignment, Background, Border, Length, Padding, Theme, border};

use std::borrow::Cow;
use std::rc::Rc;

/// A questionnaire. See the [module documentation](self).
pub struct Questionnaire<'a, Message> {
    questions: &'a [Question],
    state: Cow<'a, State>,
    on_event: Box<dyn Fn(Event) -> Message + 'a>,
    progress: bool,
    shortcuts: bool,
    submit: String,
    next: String,
}

/// Creates a [`Questionnaire`] showing the current question of `state`,
/// borrowed or owned.
pub fn questionnaire<'a, Message: Clone + 'a>(
    questions: &'a [Question],
    state: impl Into<Cow<'a, State>>,
    on_event: impl Fn(Event) -> Message + 'a,
) -> Questionnaire<'a, Message> {
    Questionnaire {
        questions,
        state: state.into(),
        on_event: Box::new(on_event),
        progress: true,
        shortcuts: false,
        submit: "Submit".to_owned(),
        next: "Next".to_owned(),
    }
}

impl<'a, Message: Clone + 'a> Questionnaire<'a, Message> {
    /// Shows "Question 1 of 3" above the question
    /// (`QuestionnaireProgress`). Defaults to `true`.
    pub fn progress(mut self, progress: bool) -> Self {
        self.progress = progress;
        self
    }

    /// Shows a letter next to each choice (`shortcuts="letters"`).
    pub fn shortcuts(mut self, shortcuts: bool) -> Self {
        self.shortcuts = shortcuts;
        self
    }

    /// Sets the label of the submit button.
    pub fn submit_label(mut self, label: impl Into<String>) -> Self {
        self.submit = label.into();
        self
    }

    /// Sets the label of the next button.
    pub fn next_label(mut self, label: impl Into<String>) -> Self {
        self.next = label.into();
        self
    }
}

/// The style of a choice of an invalid question
/// (`data-invalid:border-destructive`).
pub fn invalid_choice_style(
    theme: &Theme,
    status: Status,
    checked: bool,
) -> iced::widget::button::Style {
    let mut style = choice_style(theme, status, checked);

    if !checked {
        style.border.color = tokens(theme).palette.destructive;
    }

    style
}

/// The style of a choice.
pub fn choice_style(theme: &Theme, status: Status, checked: bool) -> iced::widget::button::Style {
    let t = tokens(theme);
    let p = &t.palette;
    let dark = t.is_dark();
    let hovered = matches!(status, Status::Hovered | Status::Pressed);

    // `border-input dark:bg-input/20 hover:bg-muted/50
    // data-checked:border-primary/40 data-checked:bg-muted`
    let background = if checked {
        Some(p.muted)
    } else if hovered {
        Some(alpha(p.muted, 0.5))
    } else if dark {
        Some(alpha(p.input, 0.2))
    } else {
        None
    };

    iced::widget::button::Style {
        background: background.map(Background::Color),
        text_color: p.foreground,
        border: Border {
            color: if checked {
                alpha(p.primary, 0.4)
            } else {
                p.input
            },
            width: 1.0,
            radius: border::radius(t.radius_lg()),
        },
        ..iced::widget::button::Style::default()
    }
}

/// The radio or checkbox indicator of a choice (`size-4`).
fn indicator<'a, Message: 'a>(checked: bool, multiple: bool) -> Element<'a, Message> {
    let mark: Element<'a, Message> = match (checked, multiple) {
        // `size-3.5`
        (true, true) => lucide::CHECK.sized(14).into(),
        // `bg-primary-foreground size-2 rounded-full`
        (true, false) => container(space().width(8).height(8))
            .style(|theme: &Theme| container::Style {
                background: Some(Background::Color(tokens(theme).palette.primary_foreground)),
                border: border::rounded(f32::MAX),
                ..container::Style::default()
            })
            .into(),
        (false, _) => space().into(),
    };

    container(mark)
        .center(16)
        .style(move |theme: &Theme| {
            let t = tokens(theme);
            let p = &t.palette;

            container::Style {
                background: if checked {
                    Some(Background::Color(p.primary))
                } else if t.is_dark() {
                    Some(Background::Color(alpha(p.input, 0.3)))
                } else {
                    None
                },
                text_color: Some(p.primary_foreground),
                border: Border {
                    color: if checked { p.primary } else { p.input },
                    width: 1.0,
                    radius: border::radius(if multiple { 4.0 } else { f32::MAX }),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// The letter of a choice (`size-5 rounded-md border font-mono
/// text-[0.625rem] font-medium`).
fn shortcut<'a, Message: 'a>(index: usize) -> Element<'a, Message> {
    let letter = char::from(b'A' + (index % 26) as u8).to_string();

    container(
        text(letter)
            .size(10)
            .font(iced::Font {
                weight: iced::font::Weight::Medium,
                ..font::mono()
            })
            .line_height(1.0),
    )
    .center(20)
    .style(|theme: &Theme| {
        let t = tokens(theme);
        let p = &t.palette;
        container::Style {
            background: Some(Background::Color(p.background)),
            text_color: Some(p.muted_foreground),
            border: style::border(p.input, t.radius_md()),
            ..container::Style::default()
        }
    })
    .into()
}

impl<'a, Message: Clone + 'a> From<Questionnaire<'a, Message>> for Element<'a, Message> {
    fn from(questionnaire: Questionnaire<'a, Message>) -> Self {
        let state = questionnaire.state.as_ref();
        let total = questionnaire.questions.len();
        let index = state.current().min(total.saturating_sub(1));
        let on_event: Rc<dyn Fn(Event) -> Message + 'a> = Rc::from(questionnaire.on_event);

        let Some(question) = questionnaire.questions.get(index) else {
            return space().into();
        };

        let first = index == 0;
        let last = index + 1 >= total;
        let selected = state.selected(index);
        let multiple = question.multiple;

        // `text-xs font-medium text-muted-foreground tabular-nums`
        let progress = questionnaire.progress.then(|| {
            typeset(format!("Question {} of {total}", index + 1))
                .size(style::text::XS)
                .font(font::medium())
                .feature(TABULAR_NUMS)
                .style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.muted_foreground),
                })
        });

        // The title is a `legend`, outside of the `gap-4` of the item;
        // `mb-4` without description.
        let title = text(question.title.clone())
            .size(style::text::BASE)
            // `cn-font-heading`
            .font(font::heading(iced::font::Weight::Medium))
            .line_height(1.375);

        let heading = Column::new()
            .push(title)
            .push(question.description.clone().map(|description| {
                text(description)
                    .size(style::text::SM)
                    .line_height(style::text::leading(style::text::SM))
                    .style(|theme: &Theme| text::Style {
                        color: Some(tokens(theme).palette.muted_foreground),
                    })
            }));

        let choices = question.choices.iter().enumerate().map(|(choice, item)| {
            let checked = selected.contains(&choice);
            let has_description = item.description.is_some();
            // `translate-y-[--spacing(0.45)]`, `translate-y-0.5` with a
            // description.
            let nudge = if has_description { 2.0 } else { 1.8 };

            // `leading-snug`
            let label = Column::new()
                .push(
                    text(item.label.clone())
                        .size(style::text::SM)
                        .line_height(1.375),
                )
                .push(item.description.clone().map(|description| {
                    text(description)
                        .size(style::text::SM)
                        .line_height(1.375)
                        .style(|theme: &Theme| text::Style {
                            color: Some(tokens(theme).palette.muted_foreground),
                        })
                }))
                // `gap-0.5`
                .spacing(2)
                .width(Length::Fill);

            let content =
                Row::new()
                    // `min-h-11`: the indicator column is at least 22px tall,
                    // within `py-2.5` and the border.
                    .push(
                        Column::new()
                            .push(
                                container(indicator(checked, multiple))
                                    .padding(Padding::ZERO.top(nudge)),
                            )
                            .push(space().height((44.0 - 2.0 * 11.0 - 16.0 - nudge).max(0.0))),
                    )
                    .push(label)
                    .push(questionnaire.shortcuts.then(|| {
                        container(shortcut(choice)).padding(Padding::ZERO.top(nudge - 2.0))
                    }))
                    // `gap-2.5`
                    .spacing(10)
                    .align_y(Alignment::Start);

            let invalid = state.is_invalid();

            // `px-3 py-2.5`, inside the border
            let choice = Pressable::new(container(content).center_y(Length::Shrink))
                .on_press(on_event(Event::Toggle(choice)))
                .width(Length::Fill)
                .padding([11, 13])
                .focus_style(move |theme, status, focused| {
                    let mut style = if invalid {
                        invalid_choice_style(theme, status, checked)
                    } else {
                        choice_style(theme, status, checked)
                    };

                    // `has-[>input:focus-visible]:border-ring`
                    if focused {
                        style.border.color = tokens(theme).palette.ring;
                    }

                    style
                });

            // The choice is the label of its input, which takes the focus:
            // `has-[>input:focus-visible]:ring-3 ring-ring/50`.
            Element::from(super::button::focus_ring(choice, |t| t.radius_lg()))
        });

        let input = question.input.as_ref().map(|placeholder| {
            super::input::input(placeholder, state.input(index))
                .on_input({
                    let on_event = on_event.clone();
                    move |value| on_event(Event::Input(value))
                })
                .invalid(state.is_invalid())
        });

        // `gap-2`
        let choices = Column::with_children(choices)
            .push(input)
            .spacing(8)
            .width(Length::Fill);

        // `mt-2 text-sm text-destructive`
        let error = state.is_invalid().then(|| {
            container(
                text(if question.required {
                    "Choose an answer to continue."
                } else {
                    "Choose an answer or skip this question."
                })
                .size(style::text::SM)
                .line_height(style::text::leading(style::text::SM))
                .style(|theme: &Theme| text::Style {
                    color: Some(tokens(theme).palette.destructive),
                }),
            )
            .padding(Padding::ZERO.top(8))
        });

        let item = Column::new()
            .push(heading)
            .push(choices)
            .push(error)
            .spacing(16)
            .width(Length::Fill);

        // `grid-cols-[minmax(0,1fr)_auto_auto] gap-2 sm:min-h-8`
        let actions = Row::new()
            .push((total > 1 && !first).then(|| {
                button("Previous")
                    .variant(ButtonVariant::Outline)
                    .on_press(on_event(Event::Previous))
            }))
            .push(space::horizontal())
            .push((!question.required).then(|| {
                button("Skip")
                    .variant(ButtonVariant::Outline)
                    .on_press(on_event(Event::Skip))
            }))
            .push(if last {
                button(questionnaire.submit.clone()).on_press(on_event(Event::Submit))
            } else {
                button(questionnaire.next.clone()).on_press(on_event(Event::Next))
            })
            .spacing(8)
            .height(32)
            .align_y(Alignment::Center);

        // `gap-4`
        Column::new()
            .push(progress)
            .push(item)
            .push(actions)
            .spacing(16)
            .width(Length::Fill)
            .into()
    }
}
