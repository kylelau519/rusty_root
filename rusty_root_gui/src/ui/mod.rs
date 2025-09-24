use iced::widget::{button, column, text, Column};
use iced::Theme;

/// Messages your UI produces.
#[derive(Debug, Clone)]
pub enum Message {
    Increment,
}

/// Application state that drives the view.
#[derive(Default)]
pub struct State {
    counter: i64,
}

/// Update logic: mutate `State` based on a `Message`.
pub fn update(state: &mut State, message: Message) {
    match message {
        Message::Increment => state.counter += 1,
    }
}

/// View logic: return a widget tree for the current `State`.
pub fn view(state: &State) -> Column<Message> {
    column![
        text(format!("Count: {}", state.counter)),
        button("+").on_press(Message::Increment),
    ]
}

/// Run the GUI with sensible defaults.
pub fn run() -> iced::Result {
    iced::application("Rusty Root GUI", update, view)
        .theme(|_| Theme::Dark)
        .centered()
        .run()
}
