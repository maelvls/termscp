//! ## RadioGroup
//!
//! `RadioGroup` component renders a radio group

/**
 * MIT License
 *
 * termscp - Copyright (c) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
// locals
use super::super::props::TextSpan;
use super::{Canvas, Component, InputEvent, Msg, Payload, PropValue, Props, PropsBuilder};
// ext
use crossterm::event::KeyCode;
use tui::{
    layout::Rect,
    style::{Color, Style},
    text::Spans,
    widgets::{Block, BorderType, Tabs},
};

// -- states

/// ## OwnStates
///
/// OwnStates contains states for this component
#[derive(Clone)]
struct OwnStates {
    choice: usize,        // Selected option
    choices: Vec<String>, // Available choices
    focus: bool,          // has focus?
}

impl Default for OwnStates {
    fn default() -> Self {
        OwnStates {
            choice: 0,
            choices: Vec::new(),
            focus: false,
        }
    }
}

impl OwnStates {
    /// ### next_choice
    ///
    /// Move choice index to next choice
    pub fn next_choice(&mut self) {
        if self.choice + 1 < self.choices.len() {
            self.choice += 1;
        }
    }

    /// ### prev_choice
    ///
    /// Move choice index to previous choice
    pub fn prev_choice(&mut self) {
        if self.choice > 0 {
            self.choice -= 1;
        }
    }

    /// ### make_choices
    ///
    /// Set OwnStates choices from a vector of text spans
    pub fn make_choices(&mut self, spans: &[TextSpan]) {
        self.choices = spans.iter().map(|x| x.content.clone()).collect();
    }
}

// -- component

/// ## RadioGroup
///
/// RadioGroup component represents a group of tabs to select from
pub struct RadioGroup {
    props: Props,
    states: OwnStates,
}

impl RadioGroup {
    /// ### new
    ///
    /// Instantiate a new Radio Group component
    pub fn new(props: Props) -> Self {
        // Make states
        let mut states: OwnStates = OwnStates::default();
        // Update choices (vec of TextSpan to String)
        states.make_choices(props.texts.rows.as_ref().unwrap_or(&Vec::new()));
        // Get value
        if let PropValue::Unsigned(choice) = props.value {
            states.choice = choice;
        }
        RadioGroup { props, states }
    }
}

impl Component for RadioGroup {
    /// ### render
    ///
    /// Based on the current properties and states, renders a widget using the provided render engine in the provided Area
    /// If focused, cursor is also set (if supported by widget)
    #[cfg(not(tarpaulin_include))]
    fn render(&self, render: &mut Canvas, area: Rect) {
        if self.props.visible {
            // Make choices
            let choices: Vec<Spans> = self
                .states
                .choices
                .iter()
                .map(|x| Spans::from(x.clone()))
                .collect();
            // Make colors
            let (bg, fg, block_color): (Color, Color, Color) = match &self.states.focus {
                true => (
                    self.props.foreground,
                    self.props.background,
                    self.props.foreground,
                ),
                false => (Color::Reset, self.props.foreground, Color::Reset),
            };
            let title: String = match &self.props.texts.title {
                Some(t) => t.clone(),
                None => String::new(),
            };
            render.render_widget(
                Tabs::new(choices)
                    .block(
                        Block::default()
                            .borders(self.props.borders)
                            .border_type(BorderType::Rounded)
                            .style(Style::default())
                            .title(title),
                    )
                    .select(self.states.choice)
                    .style(Style::default().fg(block_color))
                    .highlight_style(
                        Style::default()
                            .add_modifier(self.props.get_modifiers())
                            .fg(fg)
                            .bg(bg),
                    ),
                area,
            );
        }
    }

    /// ### update
    ///
    /// Update component properties
    /// Properties should first be retrieved through `get_props` which creates a builder from
    /// existing properties and then edited before calling update.
    /// Returns a Msg to the view
    fn update(&mut self, props: Props) -> Msg {
        // Reset choices
        self.states
            .make_choices(props.texts.rows.as_ref().unwrap_or(&Vec::new()));
        // Get value
        if let PropValue::Unsigned(choice) = props.value {
            self.states.choice = choice;
        }
        self.props = props;
        // Msg none
        Msg::None
    }

    /// ### get_props
    ///
    /// Returns a props builder starting from component properties.
    /// This returns a prop builder in order to make easier to create
    /// new properties for the element.
    fn get_props(&self) -> PropsBuilder {
        PropsBuilder::from(self.props.clone())
    }

    /// ### on
    ///
    /// Handle input event and update internal states.
    /// Returns a Msg to the view
    fn on(&mut self, ev: InputEvent) -> Msg {
        // Match event
        if let InputEvent::Key(key) = ev {
            match key.code {
                KeyCode::Right => {
                    // Increment choice
                    self.states.next_choice();
                    // Return Msg On Change
                    Msg::OnChange(self.get_value())
                }
                KeyCode::Left => {
                    // Decrement choice
                    self.states.prev_choice();
                    // Return Msg On Change
                    Msg::OnChange(self.get_value())
                }
                KeyCode::Enter => {
                    // Return Submit
                    Msg::OnSubmit(self.get_value())
                }
                _ => {
                    // Return key event to activity
                    Msg::OnKey(key)
                }
            }
        } else {
            // Ignore event
            Msg::None
        }
    }

    /// ### get_value
    ///
    /// Get current value from component
    /// Returns the selected option
    fn get_value(&self) -> Payload {
        Payload::Unsigned(self.states.choice)
    }

    // -- events

    /// ### blur
    ///
    /// Blur component; basically remove focus
    fn blur(&mut self) {
        self.states.focus = false;
    }

    /// ### active
    ///
    /// Active component; basically give focus
    fn active(&mut self) {
        self.states.focus = true;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ui::layout::props::{TextParts, TextSpan};

    use crossterm::event::KeyEvent;

    #[test]
    fn test_ui_layout_components_radio() {
        // Make component
        let mut component: RadioGroup = RadioGroup::new(
            PropsBuilder::default()
                .with_texts(TextParts::new(
                    Some(String::from("yes or no?")),
                    Some(vec![
                        TextSpan::from("Yes!"),
                        TextSpan::from("No"),
                        TextSpan::from("Maybe"),
                    ]),
                ))
                .with_value(PropValue::Unsigned(1))
                .build(),
        );
        // Verify states
        assert_eq!(component.states.choice, 1);
        assert_eq!(component.states.choices.len(), 3);
        // Focus
        assert_eq!(component.states.focus, false);
        component.active();
        assert_eq!(component.states.focus, true);
        component.blur();
        assert_eq!(component.states.focus, false);
        // Get value
        assert_eq!(component.get_value(), Payload::Unsigned(1));
        // Handle events
        assert_eq!(
            component.on(InputEvent::Key(KeyEvent::from(KeyCode::Left))),
            Msg::OnChange(Payload::Unsigned(0)),
        );
        assert_eq!(component.get_value(), Payload::Unsigned(0));
        // Left again
        assert_eq!(
            component.on(InputEvent::Key(KeyEvent::from(KeyCode::Left))),
            Msg::OnChange(Payload::Unsigned(0)),
        );
        assert_eq!(component.get_value(), Payload::Unsigned(0));
        // Right
        assert_eq!(
            component.on(InputEvent::Key(KeyEvent::from(KeyCode::Right))),
            Msg::OnChange(Payload::Unsigned(1)),
        );
        assert_eq!(component.get_value(), Payload::Unsigned(1));
        // Right again
        assert_eq!(
            component.on(InputEvent::Key(KeyEvent::from(KeyCode::Right))),
            Msg::OnChange(Payload::Unsigned(2)),
        );
        assert_eq!(component.get_value(), Payload::Unsigned(2));
        // Right again
        assert_eq!(
            component.on(InputEvent::Key(KeyEvent::from(KeyCode::Right))),
            Msg::OnChange(Payload::Unsigned(2)),
        );
        assert_eq!(component.get_value(), Payload::Unsigned(2));
        // Submit
        assert_eq!(
            component.on(InputEvent::Key(KeyEvent::from(KeyCode::Enter))),
            Msg::OnSubmit(Payload::Unsigned(2)),
        );
        // Any key
        assert_eq!(
            component.on(InputEvent::Key(KeyEvent::from(KeyCode::Char('a')))),
            Msg::OnKey(KeyEvent::from(KeyCode::Char('a'))),
        );
    }
}
