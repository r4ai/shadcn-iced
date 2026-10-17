//! A multi-step questionnaire: one question at a time, with choices, an
//! optional freeform answer, progress and navigation.
//!
//! Port of <https://ui.shadcn.com/docs/components/questionnaire>. The
//! questions are data, and the answers live in a [`State`] that you keep in
//! your application and update with the [`Event`]s of the view:
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

use shadcn_iced_core::icon::lucide;
use shadcn_iced_core::primitive::Pressable;
use shadcn_iced_core::theme::alpha;
use shadcn_iced_core::{Element, font, style, tokens};

use iced::widget::button::Status;
use iced::widget::{Column, Row, container, space, text};
use iced::{Alignment, Background, Border, Length, Padding, Theme, border};

use std::rc::Rc;

/// A choice of a [`Question`] (`QuestionnaireChoice`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// The label.
    pub label: String,
    /// An optional description below the label
    /// (`QuestionnaireChoiceDescription`).
    pub description: Option<String>,
}

impl Choice {
    /// Creates a choice with a label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
        }
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// A question (`QuestionnaireItem`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// The title (`QuestionnaireTitle`).
    pub title: String,
    /// An optional description (`QuestionnaireDescription`).
    pub description: Option<String>,
    /// The choices.
    pub choices: Vec<Choice>,
    /// Whether several choices can be picked (checkboxes instead of radio
    /// buttons).
    pub multiple: bool,
    /// Whether an answer is needed to continue; optional questions can be
    /// skipped.
    pub required: bool,
    /// The placeholder of a freeform answer, if any (`QuestionnaireInput`).
    pub input: Option<String>,
}

impl Question {
    /// Creates a required single-choice question.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            choices: Vec::new(),
            multiple: false,
            required: true,
            input: None,
        }
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Appends a choice.
    pub fn choice(mut self, choice: impl Into<Choice>) -> Self {
        self.choices.push(choice.into());
        self
    }

    /// Allows picking several choices.
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// Sets whether an answer is required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Adds a freeform answer with the given placeholder.
    pub fn input(mut self, placeholder: impl Into<String>) -> Self {
        self.input = Some(placeholder.into());
        self
    }
}

impl From<&str> for Choice {
    fn from(label: &str) -> Self {
        Choice::new(label)
    }
}

/// An interaction with a questionnaire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A choice of the current question was pressed.
    Toggle(usize),
    /// The freeform answer of the current question changed.
    Input(String),
    /// Go back to the previous question.
    Previous,
    /// Skip the current, optional, question.
    Skip,
    /// Go to the next question, if the current one is answered.
    Next,
    /// Submit the answers, if the last question is answered.
    Submit,
}

/// The answers of a questionnaire and the current question.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct State {
    current: usize,
    selected: Vec<Vec<usize>>,
    inputs: Vec<String>,
    invalid: bool,
}

impl State {
    /// Creates an empty state on the first question.
    pub fn new() -> Self {
        Self::default()
    }

    /// The index of the current question.
    pub fn current(&self) -> usize {
        self.current
    }

    /// The choices picked for each question, by index.
    pub fn selected(&self, question: usize) -> &[usize] {
        self.selected
            .get(question)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// The freeform answer of a question.
    pub fn input(&self, question: usize) -> &str {
        self.inputs.get(question).map(String::as_str).unwrap_or("")
    }

    /// The answers as text, one entry per question: the labels of the
    /// picked choices, or the freeform answer.
    pub fn answers(&self, questions: &[Question]) -> Vec<Vec<String>> {
        questions
            .iter()
            .enumerate()
            .map(|(index, question)| {
                let mut answers: Vec<String> = self
                    .selected(index)
                    .iter()
                    .filter_map(|choice| question.choices.get(*choice))
                    .map(|choice| choice.label.clone())
                    .collect();

                let input = self.input(index).trim();
                if !input.is_empty() {
                    answers.push(input.to_owned());
                }

                answers
            })
            .collect()
    }

    fn is_answered(&self, question: usize) -> bool {
        !self.selected(question).is_empty() || !self.input(question).trim().is_empty()
    }

    fn ensure(&mut self, count: usize) {
        if self.selected.len() < count {
            self.selected.resize(count, Vec::new());
            self.inputs.resize(count, String::new());
        }
    }

    /// Applies an event. Returns `true` when the questionnaire is
    /// submitted.
    pub fn update(&mut self, questions: &[Question], event: Event) -> bool {
        self.ensure(questions.len());
        let Some(question) = questions.get(self.current) else {
            return false;
        };
        let last = self.current + 1 >= questions.len();

        match event {
            Event::Toggle(choice) => {
                let selected = &mut self.selected[self.current];

                if question.multiple {
                    if let Some(position) = selected.iter().position(|c| *c == choice) {
                        selected.remove(position);
                    } else {
                        selected.push(choice);
                        selected.sort_unstable();
                    }
                } else {
                    *selected = vec![choice];
                    self.inputs[self.current].clear();
                }

                self.invalid = false;
            }
            Event::Input(value) => {
                if !question.multiple && !value.is_empty() {
                    self.selected[self.current].clear();
                }

                self.inputs[self.current] = value;
                self.invalid = false;
            }
            Event::Previous => {
                self.current = self.current.saturating_sub(1);
                self.invalid = false;
            }
            Event::Skip => {
                if !question.required {
                    self.selected[self.current].clear();
                    self.inputs[self.current].clear();
                    self.invalid = false;

                    if last {
                        return true;
                    }

                    self.current += 1;
                }
            }
            Event::Next | Event::Submit => {
                if question.required && !self.is_answered(self.current) {
                    self.invalid = true;
                    return false;
                }

                self.invalid = false;

                if last {
                    return event == Event::Submit;
                }

                self.current += 1;
            }
        }

        false
    }
}

/// A questionnaire. See the [module documentation](self).
pub struct Questionnaire<'a, Message> {
    questions: &'a [Question],
    state: &'a State,
    on_event: Box<dyn Fn(Event) -> Message + 'a>,
    progress: bool,
    shortcuts: bool,
    submit: String,
    next: String,
}

/// Creates a [`Questionnaire`] showing the current question of `state`.
pub fn questionnaire<'a, Message: Clone + 'a>(
    questions: &'a [Question],
    state: &'a State,
    on_event: impl Fn(Event) -> Message + 'a,
) -> Questionnaire<'a, Message> {
    Questionnaire {
        questions,
        state,
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
        let state = questionnaire.state;
        let total = questionnaire.questions.len();
        let index = state.current.min(total.saturating_sub(1));
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
            text(format!("Question {} of {total}", index + 1))
                .size(style::text::XS)
                .font(font::medium())
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
                        .font(font::medium())
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
                    // `min-h-11`: the indicator column is at least 20px tall.
                    .push(
                        Column::new()
                            .push(
                                container(indicator(checked, multiple))
                                    .padding(Padding::ZERO.top(nudge)),
                            )
                            .push(space().height((20.0 - 16.0 - nudge).max(0.0))),
                    )
                    .push(label)
                    .push(questionnaire.shortcuts.then(|| {
                        container(shortcut(choice)).padding(Padding::ZERO.top(nudge - 2.0))
                    }))
                    // `gap-2.5`
                    .spacing(10)
                    .align_y(Alignment::Start);

            // `px-3 py-2.5`, inside the border
            Pressable::new(container(content).center_y(Length::Shrink))
                .on_press(on_event(Event::Toggle(choice)))
                .width(Length::Fill)
                .padding([11, 13])
                .style(move |theme, status| choice_style(theme, status, checked))
                .into()
        });

        let input = question.input.as_ref().map(|placeholder| {
            super::input::input(placeholder, state.input(index))
                .on_input({
                    let on_event = on_event.clone();
                    move |value| on_event(Event::Input(value))
                })
                .invalid(state.invalid)
        });

        // `gap-2`
        let choices = Column::with_children(choices)
            .push(input)
            .spacing(8)
            .width(Length::Fill);

        // `mt-2 text-sm text-destructive`
        let error = state.invalid.then(|| {
            text(if question.required {
                "Choose an answer to continue."
            } else {
                "Choose an answer or skip this question."
            })
            .size(style::text::SM)
            .style(|theme: &Theme| text::Style {
                color: Some(tokens(theme).palette.destructive),
            })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn questions() -> Vec<Question> {
        vec![
            Question::new("One").choice("A").choice("B"),
            Question::new("Two")
                .choice("C")
                .choice("D")
                .multiple(true)
                .required(false),
        ]
    }

    #[test]
    fn requires_an_answer() {
        let questions = questions();
        let mut state = State::new();

        assert!(!state.update(&questions, Event::Next));
        assert!(state.invalid);
        assert_eq!(state.current(), 0);

        state.update(&questions, Event::Toggle(1));
        state.update(&questions, Event::Next);
        assert_eq!(state.current(), 1);

        state.update(&questions, Event::Toggle(0));
        state.update(&questions, Event::Toggle(1));
        assert_eq!(state.selected(1), [0, 1]);
        assert!(state.update(&questions, Event::Submit));
        assert_eq!(
            state.answers(&questions),
            [vec!["B".to_owned()], vec!["C".to_owned(), "D".to_owned()]]
        );
    }

    #[test]
    fn skips_optional_questions() {
        let questions = questions();
        let mut state = State::new();
        state.update(&questions, Event::Skip);
        assert_eq!(state.current(), 0, "required questions cannot be skipped");

        state.update(&questions, Event::Input("Other".to_owned()));
        state.update(&questions, Event::Next);
        assert!(state.update(&questions, Event::Skip));
    }
}
