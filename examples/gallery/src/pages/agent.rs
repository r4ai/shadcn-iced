//! Pages of the agent components: code block, questionnaire, toast and
//! trace, mirroring the examples of the docs.
use super::{DemoMessage, Page};
use crate::{Message, example};

use iced_ui::code_block::{self, Line, Size as CodeSize, Variant as CodeVariant};
use iced_ui::prelude::*;
use iced_ui::questionnaire::{Choice, Question};
use iced_ui::trace::Status;

use iced::widget::{column, container, row, text};
use iced::{Element, Fill};

fn demo(message: DemoMessage) -> Message {
    Message::Demo(message)
}

/// `w-full max-w-md`
fn md<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content).width(Fill).max_width(448).into()
}

/// `w-full max-w-sm`
fn sm<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content).width(Fill).max_width(384).into()
}

const ROUTE: &str = r#"import { streamText } from "ai"

export async function POST(request: Request) {
  const { messages } = await request.json()

  const result = streamText({
    model: "anthropic/claude-sonnet-4.5",
    system: "You are a helpful assistant. Answer in one short paragraph.",
    messages,
  })

  return result.toUIMessageStreamResponse()
}"#;

const BUILD: &str = "$ pnpm build
> next build

 ✓ Compiled successfully in 4.2s
 ✓ Generating static pages (12/12)

Route (app)                Size  First Load JS
┌ ○ /                    1.9 kB         112 kB
├ ○ /_not-found           993 B         103 kB
└ ƒ /api/chat             141 B         102 kB";

const INSTALL: &str = "$ pnpm add ai @ai-sdk/react zod
Packages: +14
Progress: resolved 412, reused 398, downloaded 14, added 14, done
Done in 2.8s using pnpm v10.18.0";

const PACKAGE: &str = r#"{
  "name": "chat",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build"
  }
}"#;

const PATCH: &str = r#" export function Greeting({ name }: { name: string }) {
-  return <p>Hello, {name}</p>
+  return <p className="text-sm text-muted-foreground">
+    Hello, {name}!
+  </p>
 }"#;

const CONFIG: &str = r#"export default {
  experimental: {
    reactCompiler: true,
    viewTransition: true,
  },
  images: {
    remotePatterns: [{ hostname: "avatars.githubusercontent.com" }],
  },
}"#;

const ERROR: &str = "TypeError: Cannot read properties of undefined (reading 'map')
    at MessageList (components/message-list.tsx:18:22)
    at renderWithHooks (react-dom.development.js:16305:18)";

const LONG: &str = "Error: The model \"anthropic/claude-sonnet-4.5\" returned an error: Request too large. The prompt is 212,431 tokens and the context window is 200,000 tokens. Remove some messages or summarize the conversation and try again.";

fn logs() -> String {
    (0..40)
        .map(|index| {
            format!(
                "[{:02}] GET /api/chat 200 in {}ms",
                index + 1,
                12 + (index * 7) % 40
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub const CODE_BLOCK: Page = Page {
    id: "code-block",
    title: "Code Block",
    description: "Displays code or terminal output with a header, actions and line numbers.",
    view: |gallery| {
        let d = &gallery.demo;
        let copy = |value: &str| demo(DemoMessage::CopyCode(value.to_owned()));
        let copied = |value: &str| d.copied.as_deref() == Some(value);

        column![
            example(
                "Demo",
                container(
                    code_block(ROUTE)
                        .title("app/api/chat/route.ts")
                        .title_icon(lucide::FILE_CODE)
                        .action(code_block::action(
                            lucide::WRAP_TEXT,
                            demo(DemoMessage::CodeWrap(!d.code_wrap))
                        ))
                        .action(code_block::action(lucide::DOWNLOAD, Message::Noop))
                        .copy_button(copy(ROUTE), copied(ROUTE))
                        .line_numbers(true)
                        .wrap(d.code_wrap)
                        .lines_with(|index, _| if (5..=9).contains(&index) {
                            Line::Highlighted
                        } else {
                            Line::Default
                        })
                )
                .max_width(512)
            ),
            example("Basic", md(code_block(BUILD))),
            example(
                "Header",
                md(code_block(INSTALL)
                    .title("Terminal")
                    .title_icon(lucide::TERMINAL)
                    .copy_button(copy("pnpm add ai @ai-sdk/react zod"), copied("pnpm add ai @ai-sdk/react zod")))
            ),
            example(
                "Floating Actions",
                md(code_block(INSTALL)
                    .copy_button(copy(INSTALL), copied(INSTALL))
                    .floating_actions(true))
            ),
            example("Line Numbers", md(code_block(PACKAGE).line_numbers(true))),
            example(
                "Start Line",
                md(code_block("  \"scripts\": {\n    \"dev\": \"next dev\",\n    \"build\": \"next build\"\n  }")
                    .line_numbers(true)
                    .start_line(4))
            ),
            example(
                "Wrap",
                md(column![code_block(LONG), code_block(LONG).wrap(true)].spacing(16))
            ),
            example(
                "Highlighted Lines",
                md(code_block(CONFIG)
                    .line_numbers(true)
                    .line(2, Line::Highlighted)
                    .line(3, Line::Highlighted))
            ),
            example(
                "Diff",
                md(code_block(PATCH)
                    .diff()
                    .title("components/greeting.tsx")
                    .copy_button(copy(PATCH), copied(PATCH)))
            ),
            example(
                "Height",
                md(code_block(logs())
                    .title("Logs")
                    .copy_button(copy("logs"), copied("logs"))
                    .max_height(192.0))
            ),
            example(
                "Show More",
                md(code_block(logs())
                    .title("Logs")
                    .max_height(if d.code_expanded { 4096.0 } else { 256.0 })
                    .footer(
                        button(if d.code_expanded { "Show less" } else { "Show more" })
                            .variant(iced_ui::button::Variant::Ghost)
                            .size(iced_ui::button::Size::Xs)
                            .on_press(demo(DemoMessage::CodeExpanded(!d.code_expanded)))
                    ))
            ),
            example(
                "Size",
                md(column![
                    code_block(INSTALL)
                        .title("Terminal")
                        .title_icon(lucide::TERMINAL)
                        .copy_button(copy(INSTALL), copied(INSTALL))
                        .wrap(true),
                    code_block(INSTALL)
                        .title("Terminal")
                        .title_icon(lucide::TERMINAL)
                        .copy_button(copy(INSTALL), copied(INSTALL))
                        .wrap(true)
                        .size(CodeSize::Sm),
                ]
                .spacing(16))
            ),
            example(
                "Destructive",
                md(code_block(ERROR)
                    .title("Error")
                    .copy_button(copy(ERROR), copied(ERROR))
                    .wrap(true)
                    .variant(CodeVariant::Destructive))
            ),
        ]
        .into()
    },
};

/// The questions of the questionnaire demo.
pub fn questions() -> Vec<Question> {
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
}

pub const QUESTIONNAIRE: Page = Page {
    id: "questionnaire",
    title: "Questionnaire",
    description: "A multi-step form that asks one question at a time.",
    view: |gallery| {
        let d = &gallery.demo;

        column![
            example(
                "Demo",
                md(questionnaire(&d.questions, &d.questionnaire, |event| {
                    demo(DemoMessage::Questionnaire(event))
                })
                .shortcuts(true)
                .submit_label("Save plan"))
            ),
            example(
                "Card",
                md(card().content(
                    questionnaire(&d.questions, &d.questionnaire, |event| {
                        demo(DemoMessage::Questionnaire(event))
                    })
                    .progress(false)
                    .submit_label("Save plan")
                ))
            ),
        ]
        .into()
    },
};

pub const TOAST: Page = Page {
    id: "toast",
    title: "Toast",
    description: "A succinct message that is displayed temporarily.",
    view: |_gallery| {
        use iced_ui::toast::Kind;

        let kind = |label: &'static str, kind: Kind| {
            button(label)
                .variant(iced_ui::button::Variant::Outline)
                .on_press(demo(DemoMessage::BaseToast(Some(kind))))
        };

        column![
            example(
                "Demo",
                button("Show Toast")
                    .variant(iced_ui::button::Variant::Outline)
                    .on_press(demo(DemoMessage::BaseToast(None)))
            ),
            example(
                "Types",
                row![
                    kind("Default", Kind::Default),
                    kind("Success", Kind::Success),
                    kind("Info", Kind::Info),
                    kind("Warning", Kind::Warning),
                    kind("Error", Kind::Error),
                    kind("Loading", Kind::Loading),
                ]
                .spacing(8)
                .wrap()
            ),
        ]
        .into()
    },
};

fn output<'a>(content: &str) -> Element<'a, Message> {
    code_block(content)
        .size(CodeSize::Sm)
        .wrap(true)
        .max_height(256.0)
        .into()
}

pub const TRACE: Page = Page {
    id: "trace",
    title: "Trace",
    description: "Shows the steps of an agent: tool calls, edits, searches and reasoning.",
    view: |_| {
        column![
            example(
                "Demo",
                sm(trace([
                    trace_line("Thought for 5s")
                        .icon(lucide::CIRCLE_CHECK)
                        .collapsible(text(
                            "CI installs with a frozen lockfile, so the install stops when package.json and pnpm-lock.yaml disagree. I should check what the last commit changed, then reproduce the install."
                        ).size(14).line_height(1.625))
                        .into(),
                    trace_line("Ran pnpm --version")
                        .icon(lucide::TERMINAL)
                        .collapsible(output("$ pnpm --version\n10.18.0"))
                        .into(),
                    trace_line("Ran git diff --stat HEAD~1")
                        .icon(lucide::TERMINAL)
                        .collapsible(output(
                            " package.json | 2 +-\n 1 file changed, 1 insertion(+), 1 deletion(-)"
                        ))
                        .into(),
                    trace_line("Failed to run pnpm install --frozen-lockfile")
                        .icon(lucide::CIRCLE_ALERT)
                        .status(Status::Error)
                        .collapsible(output(
                            "ERR_PNPM_OUTDATED_LOCKFILE  Cannot install with \"frozen-lockfile\" because pnpm-lock.yaml is not up to date with package.json"
                        ))
                        .default_open(true)
                        .into(),
                    trace_line("Used 2 sources")
                        .icon(lucide::GLOBE)
                        .collapsible(trace([
                            trace_line("pnpm install | pnpm.io")
                                .icon(lucide::GLOBE)
                                .trailing_icon(lucide::ARROW_UP_RIGHT)
                                .on_press(Message::Noop)
                                .into(),
                            trace_line("Continuous Integration | pnpm.io")
                                .icon(lucide::GLOBE)
                                .trailing_icon(lucide::ARROW_UP_RIGHT)
                                .on_press(Message::Noop)
                                .into(),
                        ]))
                        .into(),
                ]))
            ),
            example(
                "Trace Line",
                sm(trace([trace_line("Created branch fix/frozen-lockfile")
                    .icon(lucide::GIT_BRANCH)
                    .into()]))
            ),
            example(
                "Status",
                sm(trace([
                    trace_line("Ran pnpm install")
                        .icon(lucide::CIRCLE_CHECK)
                        .into(),
                    trace_line("Failed to run pnpm lint")
                        .icon(lucide::CIRCLE_ALERT)
                        .status(Status::Error)
                        .into(),
                    trace_line("Running pnpm build")
                        .icon(lucide::LOADER_CIRCLE)
                        .status(Status::Running)
                        .into(),
                    trace_line("Run pnpm test")
                        .icon(lucide::CIRCLE)
                        .status(Status::Pending)
                        .into(),
                ]))
            ),
            example(
                "Description",
                sm(trace([
                    trace_line("Edited components/settings/notification-preferences-form.tsx")
                        .icon(lucide::FILE_PEN)
                        .description("+48 −12")
                        .mono_description(true)
                        .into(),
                    trace_line("Ran pnpm test")
                        .icon(lucide::TERMINAL)
                        .description("14s")
                        .into(),
                    trace_line("Searched for useChat")
                        .icon(lucide::SEARCH)
                        .description("8 results")
                        .into(),
                ]))
            ),
            example(
                "Collapsible",
                sm(trace_line("Ran node --version")
                    .icon(lucide::TERMINAL)
                    .collapsible(output("$ node --version\nv24.18.0"))
                    .default_open(true))
            ),
            example(
                "Link",
                sm(trace([
                    trace_line("Read apps/v4/lib/docs.ts")
                        .icon(lucide::FILE_TEXT)
                        .trailing_icon(lucide::CHEVRON_RIGHT)
                        .on_press(Message::Noop)
                        .into(),
                    trace_line("Install Astro | docs.astro.build")
                        .icon(lucide::GLOBE)
                        .trailing_icon(lucide::ARROW_UP_RIGHT)
                        .on_press(Message::Noop)
                        .into(),
                ]))
            ),
            example(
                "Group",
                sm(trace([
                    trace_line("Searched for minimumReleaseAge")
                        .icon(lucide::SEARCH)
                        .into(),
                    trace_line("Read pnpm-workspace.yaml")
                        .icon(lucide::FILE_TEXT)
                        .into(),
                    trace_line("Ran pnpm install")
                        .icon(lucide::TERMINAL)
                        .into(),
                ]))
            ),
            example(
                "Chain of Thought",
                sm(trace_line("Looking into the slow deploy")
                    .status(Status::Running)
                    .collapsible(trace([
                        trace_line("Searched for the checkout build log")
                            .icon(lucide::SEARCH)
                            .collapsible(
                                row![
                                    badge("vercel.com")
                                        .variant(iced_ui::badge::Variant::Secondary),
                                    badge("github.com")
                                        .variant(iced_ui::badge::Variant::Secondary),
                                ]
                                .spacing(6)
                            )
                            .into(),
                        trace_line("Read the last twelve deploys")
                            .icon(lucide::FILE_TEXT)
                            .trailing_icon(lucide::CHEVRON_RIGHT)
                            .on_press(Message::Noop)
                            .into(),
                        trace_line("Searching for the cache key")
                            .icon(lucide::SEARCH)
                            .status(Status::Running)
                            .collapsible(output("cache key image optimization"))
                            .into(),
                    ]))
                    .default_open(true))
            ),
        ]
        .into()
    },
};
