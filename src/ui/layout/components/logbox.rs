//! ## LogBox
//!
//! `LogBox` component renders a log box view

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
use super::{Canvas, Component, InputEvent, Msg, Payload, Props, PropsBuilder};
// ext
use crossterm::event::KeyCode;
use std::collections::VecDeque;
use tui::{
    layout::{Corner, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, List, ListItem, ListState},
};

// -- states

/// ## OwnStates
///
/// OwnStates contains states for this component
#[derive(Clone)]
struct OwnStates {
    list_index: usize, // Index of selected element in list
    list_len: usize,   // Length of file list
    focus: bool,       // Has focus?
}

impl Default for OwnStates {
    fn default() -> Self {
        OwnStates {
            list_index: 0,
            list_len: 0,
            focus: false,
        }
    }
}

impl OwnStates {
    /// ### set_list_len
    ///
    /// Set list length
    pub fn set_list_len(&mut self, len: usize) {
        self.list_len = len;
    }

    /// ### get_list_index
    ///
    /// Return current value for list index
    pub fn get_list_index(&self) -> usize {
        self.list_index
    }

    /// ### incr_list_index
    ///
    /// Incremenet list index
    pub fn incr_list_index(&mut self) {
        // Check if index is at last element
        if self.list_index + 1 < self.list_len {
            self.list_index += 1;
        }
    }

    /// ### decr_list_index
    ///
    /// Decrement list index
    pub fn decr_list_index(&mut self) {
        // Check if index is bigger than 0
        if self.list_index > 0 {
            self.list_index -= 1;
        }
    }

    /// ### reset_list_index
    ///
    /// Reset list index to last element
    pub fn reset_list_index(&mut self) {
        self.list_index = 0; // Last element is always 0
    }
}

// -- Component

/// ## LogBox
///
/// LogBox list component
pub struct LogBox {
    props: Props,
    states: OwnStates,
}

impl LogBox {
    /// ### new
    ///
    /// Instantiates a new FileList starting from Props
    /// The method also initializes the component states.
    pub fn new(props: Props) -> Self {
        // Initialize states
        let mut states: OwnStates = OwnStates::default();
        // Set list length
        states.set_list_len(match &props.texts.table {
            Some(rows) => rows.len(),
            None => 0,
        });
        // Reset list index
        states.reset_list_index();
        LogBox { props, states }
    }
}

impl Component for LogBox {
    /// ### render
    ///
    /// Based on the current properties and states, renders a widget using the provided render engine in the provided Area
    /// If focused, cursor is also set (if supported by widget)
    #[cfg(not(tarpaulin_include))]
    fn render(&self, render: &mut Canvas, area: Rect) {
        if self.props.visible {
            // Make list
            let list_items: Vec<ListItem> = match self.props.texts.table.as_ref() {
                None => Vec::new(),
                Some(table) => table
                    .iter()
                    .enumerate()
                    .map(|(idx, row)| {
                        let mut columns: VecDeque<Span> = row
                            .iter()
                            .map(|col| {
                                Span::styled(
                                    col.content.clone(),
                                    Style::default()
                                        .add_modifier(col.get_modifiers())
                                        .fg(col.fg)
                                        .bg(col.bg),
                                )
                            })
                            .collect();
                        // Let's convert column spans into Spans rows NOTE: -4 because first line is always made by 5 columns; but there's always 1
                        let mut rows: Vec<Spans> = Vec::with_capacity(columns.len() - 4);
                        // Get first row
                        let mut first_row: Vec<Span> = vec![Span::styled(
                            match self.states.list_index == idx {
                                true => "> ",
                                false => "  ",
                            },
                            Style::default()
                                .fg(self.props.foreground)
                                .bg(self.props.background),
                        )];
                        for _ in 0..5 {
                            if let Some(col) = columns.pop_front() {
                                first_row.push(col);
                            }
                        }
                        rows.push(Spans::from(first_row));
                        // Fill remaining rows
                        let cycles: usize = columns.len();
                        for _ in 0..cycles {
                            if let Some(col) = columns.pop_front() {
                                rows.push(Spans::from(vec![col]));
                            }
                        }
                        ListItem::new(rows)
                    })
                    .collect(), // Make List item from TextSpan
            };
            let title: String = match self.props.texts.title.as_ref() {
                Some(t) => t.clone(),
                None => String::new(),
            };
            // Render

            let w = List::new(list_items)
                .block(
                    Block::default()
                        .borders(self.props.borders)
                        .border_style(match self.states.focus {
                            true => Style::default().fg(self.props.foreground),
                            false => Style::default(),
                        })
                        .title(title),
                )
                .start_corner(Corner::BottomLeft)
                .highlight_style(Style::default().add_modifier(self.props.get_modifiers()));
            let mut state: ListState = ListState::default();
            state.select(Some(self.states.list_index));
            render.render_stateful_widget(w, area, &mut state);
        }
    }

    /// ### update
    ///
    /// Update component properties
    /// Properties should first be retrieved through `get_props` which creates a builder from
    /// existing properties and then edited before calling update
    fn update(&mut self, props: Props) -> Msg {
        self.props = props;
        // re-Set list length
        self.states.set_list_len(match &self.props.texts.table {
            Some(rows) => rows.len(),
            None => 0,
        });
        // Reset list index
        self.states.reset_list_index();
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
    /// Handle input event and update internal states
    fn on(&mut self, ev: InputEvent) -> Msg {
        // Match event
        if let InputEvent::Key(key) = ev {
            match key.code {
                KeyCode::Up => {
                    // Update states
                    self.states.incr_list_index();
                    Msg::None
                }
                KeyCode::Down => {
                    // Update states
                    self.states.decr_list_index();
                    Msg::None
                }
                KeyCode::PageUp => {
                    // Update states
                    for _ in 0..8 {
                        self.states.incr_list_index();
                    }
                    Msg::None
                }
                KeyCode::PageDown => {
                    // Update states
                    for _ in 0..8 {
                        self.states.decr_list_index();
                    }
                    Msg::None
                }
                _ => {
                    // Return key event to activity
                    Msg::OnKey(key)
                }
            }
        } else {
            // Unhandled event
            Msg::None
        }
    }

    /// ### get_value
    ///
    /// Return component value. File list return index
    fn get_value(&self) -> Payload {
        Payload::Unsigned(self.states.get_list_index())
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
    use crate::ui::layout::props::{TableBuilder, TextParts, TextSpan};

    use crossterm::event::{KeyCode, KeyEvent};

    #[test]
    fn test_ui_layout_components_logbox() {
        let mut component: LogBox = LogBox::new(
            PropsBuilder::default()
                .with_texts(TextParts::table(
                    Some(String::from("Log")),
                    TableBuilder::default()
                        .add_col(TextSpan::from("12:29"))
                        .add_col(TextSpan::from("system crashed"))
                        .add_row()
                        .add_col(TextSpan::from("12:38"))
                        .add_col(TextSpan::from("system alive"))
                        .build(),
                ))
                .build(),
        );
        // Verify states
        assert_eq!(component.states.list_index, 0);
        assert_eq!(component.states.list_len, 2);
        assert_eq!(component.states.focus, false);
        // Focus
        component.active();
        assert_eq!(component.states.focus, true);
        component.blur();
        assert_eq!(component.states.focus, false);
        // Increment list index
        component.states.list_index += 1;
        assert_eq!(component.states.list_index, 1);
        // Update
        component.update(
            component
                .get_props()
                .with_texts(TextParts::table(
                    Some(String::from("Log")),
                    TableBuilder::default()
                        .add_col(TextSpan::from("12:29"))
                        .add_col(TextSpan::from("system crashed"))
                        .add_row()
                        .add_col(TextSpan::from("12:38"))
                        .add_col(TextSpan::from("system alive"))
                        .add_row()
                        .add_col(TextSpan::from("12:41"))
                        .add_col(TextSpan::from("system is going down for REBOOT"))
                        .build(),
                ))
                .build(),
        );
        // Verify states
        assert_eq!(component.states.list_index, 0); // Last item
        assert_eq!(component.states.list_len, 3);
        // get value
        assert_eq!(component.get_value(), Payload::Unsigned(0));
        // RenderData
        assert_eq!(component.states.list_index, 0);
        // Set cursor to 0
        component.states.list_index = 0;
        // Handle inputs
        assert_eq!(
            component.on(InputEvent::Key(KeyEvent::from(KeyCode::Up))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 1);
        // Index should be decremented
        assert_eq!(
            component.on(InputEvent::Key(KeyEvent::from(KeyCode::Down))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 0);
        // Index should be 2
        assert_eq!(
            component.on(InputEvent::Key(KeyEvent::from(KeyCode::PageUp))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 2);
        // Index should be 0
        assert_eq!(
            component.on(InputEvent::Key(KeyEvent::from(KeyCode::PageDown))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 0);
        // On key
        assert_eq!(
            component.on(InputEvent::Key(KeyEvent::from(KeyCode::Backspace))),
            Msg::OnKey(KeyEvent::from(KeyCode::Backspace))
        );
    }
}
