use iced::{
    alignment::{self, Alignment},
    widget::{button, component, row, text, text_input, Component},
    Element, Length, Size,
};

pub struct NumericInput<Message> {
    value: Option<u64>,
    step: u64,
    on_change: Box<dyn Fn(Option<u64>) -> Message>,
}

pub fn numeric_input<Message>(
    value: Option<u64>,
    step: u64,
    on_change: impl Fn(Option<u64>) -> Message + 'static,
) -> NumericInput<Message> {
    NumericInput::new(value, step, on_change)
}

#[derive(Debug, Clone)]
pub enum Event {
    InputChanged(String),
    IncrementPressed,
    DecrementPressed,
}

impl<Message> NumericInput<Message> {
    pub fn new(
        value: Option<u64>,
        step: u64,
        on_change: impl Fn(Option<u64>) -> Message + 'static,
    ) -> Self {
        Self {
            value,
            step,
            on_change: Box::new(on_change),
        }
    }
}

impl<Message> Component<Message> for NumericInput<Message> {
    type Event = Event;
    type State = ();

    fn update(&mut self, _state: &mut Self::State, event: Event) -> Option<Message> {
        match event {
            Event::IncrementPressed => Some((self.on_change)(Some(
                self.value.unwrap_or_default().saturating_add(self.step),
            ))),
            Event::DecrementPressed => Some((self.on_change)(Some(
                self.value.unwrap_or_default().saturating_sub(self.step),
            ))),
            Event::InputChanged(value) => {
                if value.is_empty() {
                    Some((self.on_change)(None))
                } else {
                    value.parse().ok().map(Some).map(self.on_change.as_ref())
                }
            }
        }
    }

    fn view(&self, _state: &Self::State) -> Element<Event> {
        let button = |label, on_press| {
            button(
                text(label)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center),
            )
            .width(40)
            .height(40)
            .on_press(on_press)
        };

        row![
            button("-", Event::DecrementPressed),
            text_input(
                "Type a number",
                self.value
                    .as_ref()
                    .map(u64::to_string)
                    .as_deref()
                    .unwrap_or(""),
            )
            .on_input(Event::InputChanged)
            .padding(10),
            button("+", Event::IncrementPressed),
        ]
        .align_items(Alignment::Center)
        .spacing(10)
        .into()
    }

    fn size_hint(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }
}

impl<'a, Message> From<NumericInput<Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(numeric_input: NumericInput<Message>) -> Self {
        component(numeric_input)
    }
}
