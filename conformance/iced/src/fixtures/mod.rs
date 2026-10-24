//! The iced counterparts of `conformance/src/fixtures/*.tsx`.
//!
//! Every component has a module with a `get` function that maps the ids of
//! its scenarios (`conformance/scenarios/<component>.json`) to fixtures.
use crate::store::{Msg, Store};

use iced::Element;

mod accordion;
mod alert;
mod alert_dialog;
mod aspect_ratio;
mod attachment;
mod avatar;
mod badge;
mod breadcrumb;
mod bubble;
mod button;
mod button_group;
mod calendar;
mod card;
mod carousel;
mod chart;
mod checkbox;
mod code_block;
mod collapsible;
mod combobox;
mod command;
mod context_menu;
mod date_picker;
mod dialog;
mod drawer;
mod dropdown_menu;
mod empty;
mod field;
mod hover_card;
mod input;
mod input_group;
mod input_otp;
mod item;
mod kbd;
mod label;
mod marker;
mod menubar;
mod message;
mod message_scroller;
mod native_select;
mod navigation_menu;
mod pagination;
mod popover;
mod progress;
mod questionnaire;
mod radio_group;
mod resizable;
mod scroll_area;
mod select;
mod separator;
mod sheet;
mod sidebar;
mod skeleton;
mod slider;
mod sonner;
mod spinner;
mod switch;
mod table;
mod tabs;
mod textarea;
mod toast;
mod toggle;
mod toggle_group;
mod tooltip;
mod trace;
mod typography;

/// What a scenario renders.
pub struct Fixture {
    /// Sets the initial state of the store.
    pub init: Box<dyn Fn(&mut Store)>,
    pub view: for<'a> fn(&'a Store) -> Element<'a, Msg>,
}

impl Fixture {
    pub fn new(view: for<'a> fn(&'a Store) -> Element<'a, Msg>) -> Self {
        Self {
            init: Box::new(|_| {}),
            view,
        }
    }

    pub fn init(mut self, init: impl Fn(&mut Store) + 'static) -> Self {
        self.init = Box::new(init);
        self
    }
}

/// Returns the fixture of a scenario, if implemented.
pub fn get(component: &str, id: &str) -> Option<Fixture> {
    match component {
        "accordion" => accordion::get(id),
        "alert" => alert::get(id),
        "alert-dialog" => alert_dialog::get(id),
        "aspect-ratio" => aspect_ratio::get(id),
        "attachment" => attachment::get(id),
        "avatar" => avatar::get(id),
        "badge" => badge::get(id),
        "breadcrumb" => breadcrumb::get(id),
        "bubble" => bubble::get(id),
        "button" => button::get(id),
        "button-group" => button_group::get(id),
        "calendar" => calendar::get(id),
        "card" => card::get(id),
        "carousel" => carousel::get(id),
        "chart" => chart::get(id),
        "checkbox" => checkbox::get(id),
        "code-block" => code_block::get(id),
        "collapsible" => collapsible::get(id),
        "combobox" => combobox::get(id),
        "command" => command::get(id),
        "context-menu" => context_menu::get(id),
        "date-picker" => date_picker::get(id),
        "dialog" => dialog::get(id),
        "drawer" => drawer::get(id),
        "dropdown-menu" => dropdown_menu::get(id),
        "empty" => empty::get(id),
        "field" => field::get(id),
        "hover-card" => hover_card::get(id),
        "input" => input::get(id),
        "input-group" => input_group::get(id),
        "input-otp" => input_otp::get(id),
        "item" => item::get(id),
        "kbd" => kbd::get(id),
        "label" => label::get(id),
        "marker" => marker::get(id),
        "menubar" => menubar::get(id),
        "message" => message::get(id),
        "message-scroller" => message_scroller::get(id),
        "native-select" => native_select::get(id),
        "navigation-menu" => navigation_menu::get(id),
        "pagination" => pagination::get(id),
        "popover" => popover::get(id),
        "progress" => progress::get(id),
        "questionnaire" => questionnaire::get(id),
        "radio-group" => radio_group::get(id),
        "resizable" => resizable::get(id),
        "scroll-area" => scroll_area::get(id),
        "select" => select::get(id),
        "separator" => separator::get(id),
        "sheet" => sheet::get(id),
        "sidebar" => sidebar::get(id),
        "skeleton" => skeleton::get(id),
        "slider" => slider::get(id),
        "sonner" => sonner::get(id),
        "spinner" => spinner::get(id),
        "switch" => switch::get(id),
        "table" => table::get(id),
        "tabs" => tabs::get(id),
        "textarea" => textarea::get(id),
        "toast" => toast::get(id),
        "toggle" => toggle::get(id),
        "toggle-group" => toggle_group::get(id),
        "tooltip" => tooltip::get(id),
        "trace" => trace::get(id),
        "typography" => typography::get(id),
        _ => None,
    }
}
