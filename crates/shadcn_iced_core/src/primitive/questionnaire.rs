//! The model of a multi-step questionnaire: the questions, their answers,
//! and the navigation between them.
//!
//! The questions are data ([`Question`], [`Choice`]). A [`State`] keeps the
//! current question and the answers, and applies the [`Event`]s of a view:
//! a choice is picked (or toggled when several can be picked), a freeform
//! answer replaces the choice of a single-choice question, a required
//! question must be answered before going to the next one or submitting
//! (otherwise the state becomes invalid), and an optional one can be
//! skipped. Its view is up to you.
//!
//! ```
//! # use shadcn_iced_core::primitive::questionnaire::{Event, Question, State};
//! let questions = [
//!     Question::new("Role").choice("Design").choice("Engineering"),
//!     Question::new("Topics").choice("Layout").choice("Color").multiple(true),
//! ];
//! let mut state = State::new();
//!
//! state.update(&questions, Event::Next);
//! assert!(state.is_invalid(), "the first question is required");
//!
//! state.update(&questions, Event::Toggle(1));
//! state.update(&questions, Event::Next);
//! state.update(&questions, Event::Toggle(0));
//! assert!(state.update(&questions, Event::Submit));
//! assert_eq!(state.answers(&questions), [vec!["Engineering"], vec!["Layout"]]);
//! ```
use std::borrow::Cow;

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

    /// Whether the current question was left unanswered while going to the
    /// next one (`data-invalid`), until it is answered or left.
    pub fn is_invalid(&self) -> bool {
        self.invalid
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

impl<'a> From<&'a State> for Cow<'a, State> {
    fn from(state: &'a State) -> Self {
        Cow::Borrowed(state)
    }
}

impl From<State> for Cow<'_, State> {
    fn from(state: State) -> Self {
        Cow::Owned(state)
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
