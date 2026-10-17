use super::Fixture;
use crate::store::Msg;

use shadcn_iced::code_block::{Line, Size, Variant, code_block};
use shadcn_iced::prelude::lucide;

use iced::widget::container;
use iced::{Element, Fill};

const INSTALL: &str = "pnpm add ai @ai-sdk/react zod";

const PACKAGE: &str = "{
  \"name\": \"chat\",
  \"private\": true,
  \"scripts\": {
    \"dev\": \"next dev\",
    \"build\": \"next build\"
  }
}";

const PATCH: &str = " export function Greeting() {
-  return <p>Hello</p>
+  return <p>Hello!</p>
 }";

const ERROR: &str = "TypeError: Cannot read properties of undefined (reading 'map') at MessageList (components/message-list.tsx:18:22)";

/// `w-full max-w-md`
fn md(content: Element<'_, Msg>) -> Element<'_, Msg> {
    container(content).width(Fill).max_width(448).into()
}

pub fn get(id: &str) -> Option<Fixture> {
    Some(match id {
        "header" => Fixture::new(|_| {
            md(code_block(INSTALL)
                .title("Terminal")
                .title_icon(lucide::TERMINAL)
                .copy_button(Msg::Noop, false)
                .into())
        }),
        "line-numbers" => Fixture::new(|_| {
            md(code_block(PACKAGE)
                .title("package.json")
                .title_icon(lucide::FILE_CODE)
                .line_numbers(true)
                .line(3, Line::Highlighted)
                .line(4, Line::Highlighted)
                .into())
        }),
        "diff" => Fixture::new(|_| {
            md(code_block(PATCH)
                .title("greeting.tsx")
                .title_icon(lucide::FILE_CODE)
                .diff()
                .into())
        }),
        "floating-actions" => Fixture::new(|_| {
            md(code_block(INSTALL)
                .copy_button(Msg::Noop, false)
                .floating_actions(true)
                .into())
        }),
        "destructive-sm" => Fixture::new(|_| {
            md(code_block(ERROR)
                .title("Error")
                .copy_button(Msg::Noop, false)
                .wrap(true)
                .size(Size::Sm)
                .variant(Variant::Destructive)
                .into())
        }),
        _ => return None,
    })
}
