//! The questionnaire keeps its answers in a [`State`] updated with [`Event`]s,
//! which the key-value store cannot hold. The fixture stores the log of the
//! events instead and replays it to rebuild the state, which the view owns,
//! on every view.
use super::Fixture;
use crate::store::{Msg, Store, Value};

use shadcn_iced::questionnaire::{Choice, Event, Question, State, questionnaire};

use iced::Element;
use iced::Fill;
use iced::widget::container;

use std::sync::OnceLock;

/// `w-full max-w-md`
fn md(content: Element<'_, Msg>) -> Element<'_, Msg> {
    container(content).width(Fill).max_width(448).into()
}

fn questions() -> &'static [Question] {
    static QUESTIONS: OnceLock<Vec<Question>> = OnceLock::new();

    QUESTIONS.get_or_init(|| {
        vec![
            Question::new("What should the agent build next?")
                .description("Choose a direction or describe another task.")
                .choice(
                    Choice::new("Tool call timeline")
                        .description("Show what the agent ran and what came back."),
                )
                .choice(
                    Choice::new("Approval checkpoints")
                        .description("Ask before sensitive or destructive actions."),
                )
                .choice(
                    Choice::new("Sub-agent handoffs")
                        .description("Make delegated work and results easier to follow."),
                )
                .input("Describe another feature…"),
            Question::new("What should every progress update include?")
                .description("Select all that apply, or skip this question.")
                .choice("Progress")
                .choice("Decisions")
                .choice("Risks")
                .choice("Next step")
                .multiple(true)
                .required(false),
            Question::new("When should work begin?")
                .description("Choose when the agent should begin the work.")
                .choice("Start now")
                .choice("Next development cycle")
                .choice("Add it to the backlog"),
        ]
    })
}

fn encode(event: &Event) -> String {
    match event {
        Event::Toggle(choice) => format!("t{choice}"),
        Event::Input(value) => format!("i{value}"),
        Event::Previous => "p".to_owned(),
        Event::Skip => "s".to_owned(),
        Event::Next => "n".to_owned(),
        Event::Submit => "u".to_owned(),
    }
}

fn decode(line: &str) -> Option<Event> {
    let (kind, rest) = line.split_at(1);

    Some(match kind {
        "t" => Event::Toggle(rest.parse().ok()?),
        "i" => Event::Input(rest.to_owned()),
        "p" => Event::Previous,
        "s" => Event::Skip,
        "n" => Event::Next,
        "u" => Event::Submit,
        _ => return None,
    })
}

fn view(store: &Store) -> Element<'_, Msg> {
    let log = store.str("log").to_owned();
    let mut state = State::new();

    for event in log.lines().filter_map(decode) {
        let _ = state.update(questions(), event);
    }

    md(questionnaire(questions(), state, move |event| {
        Msg::Set(
            "log",
            Value::Str(format!("{log}{}\n", encode(&event).replace('\n', " "))),
        )
    })
    .shortcuts(true)
    .into())
}

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "flow" | "skip" => Fixture::new(view),
        _ => return None,
    })
}
