//! ## AuthActivity
//!
//! `auth_activity` is the module which implements the authentication activity

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
use super::{
    AuthActivity, FileTransferParams, COMPONENT_BOOKMARKS_LIST, COMPONENT_INPUT_ADDR,
    COMPONENT_INPUT_BOOKMARK_NAME, COMPONENT_INPUT_PASSWORD, COMPONENT_INPUT_PORT,
    COMPONENT_INPUT_USERNAME, COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK,
    COMPONENT_RADIO_BOOKMARK_DEL_RECENT, COMPONENT_RADIO_BOOKMARK_SAVE_PWD,
    COMPONENT_RADIO_PROTOCOL, COMPONENT_RADIO_QUIT, COMPONENT_RECENTS_LIST, COMPONENT_TEXT_ERROR,
    COMPONENT_TEXT_HELP,
};
use crate::ui::activities::keymap::*;
use crate::ui::layout::{Msg, Payload};

// -- update

impl AuthActivity {
    /// ### update
    ///
    /// Update auth activity model based on msg
    /// The function exits when returns None
    pub(super) fn update(&mut self, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
        let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
        // Match msg
        match ref_msg {
            None => None, // Exit after None
            Some(msg) => match msg {
                // Focus ( DOWN )
                (COMPONENT_INPUT_ADDR, &MSG_KEY_DOWN) => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_PORT);
                    None
                }
                (COMPONENT_INPUT_PORT, &MSG_KEY_DOWN) => {
                    // Give focus to port
                    self.view.active(COMPONENT_RADIO_PROTOCOL);
                    None
                }
                (COMPONENT_RADIO_PROTOCOL, &MSG_KEY_DOWN) => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_USERNAME);
                    None
                }
                (COMPONENT_INPUT_USERNAME, &MSG_KEY_DOWN) => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_PASSWORD);
                    None
                }
                (COMPONENT_INPUT_PASSWORD, &MSG_KEY_DOWN) => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_ADDR);
                    None
                }
                // Focus ( UP )
                (COMPONENT_INPUT_PASSWORD, &MSG_KEY_UP) => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_USERNAME);
                    None
                }
                (COMPONENT_INPUT_USERNAME, &MSG_KEY_UP) => {
                    // Give focus to port
                    self.view.active(COMPONENT_RADIO_PROTOCOL);
                    None
                }
                (COMPONENT_RADIO_PROTOCOL, &MSG_KEY_UP) => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_PORT);
                    None
                }
                (COMPONENT_INPUT_PORT, &MSG_KEY_UP) => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_ADDR);
                    None
                }
                (COMPONENT_INPUT_ADDR, &MSG_KEY_UP) => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_PASSWORD);
                    None
                }
                // <TAB> bookmarks
                (COMPONENT_BOOKMARKS_LIST, &MSG_KEY_TAB)
                | (COMPONENT_RECENTS_LIST, &MSG_KEY_TAB) => {
                    // Give focus to address
                    self.view.active(COMPONENT_INPUT_ADDR);
                    None
                }
                // Any <TAB>, go to bookmarks
                (_, &MSG_KEY_TAB) => {
                    self.view.active(COMPONENT_BOOKMARKS_LIST);
                    None
                }
                // Bookmarks commands
                // <RIGHT> / <LEFT>
                (COMPONENT_BOOKMARKS_LIST, &MSG_KEY_RIGHT) => {
                    // Give focus to recents
                    self.view.active(COMPONENT_RECENTS_LIST);
                    None
                }
                (COMPONENT_RECENTS_LIST, &MSG_KEY_LEFT) => {
                    // Give focus to bookmarks
                    self.view.active(COMPONENT_BOOKMARKS_LIST);
                    None
                }
                // <DEL | 'E'>
                (COMPONENT_BOOKMARKS_LIST, &MSG_KEY_DEL)
                | (COMPONENT_BOOKMARKS_LIST, &MSG_KEY_CHAR_E) => {
                    // Show delete popup
                    self.mount_bookmark_del_dialog();
                    None
                }
                (COMPONENT_RECENTS_LIST, &MSG_KEY_DEL)
                | (COMPONENT_RECENTS_LIST, &MSG_KEY_CHAR_E) => {
                    // Show delete popup
                    self.mount_recent_del_dialog();
                    None
                }
                // Enter
                (COMPONENT_BOOKMARKS_LIST, Msg::OnSubmit(Payload::Unsigned(idx))) => {
                    self.load_bookmark(*idx);
                    // Give focus to input password
                    self.view.active(COMPONENT_INPUT_PASSWORD);
                    None
                }
                (COMPONENT_RECENTS_LIST, Msg::OnSubmit(Payload::Unsigned(idx))) => {
                    self.load_recent(*idx);
                    // Give focus to input password
                    self.view.active(COMPONENT_INPUT_PASSWORD);
                    None
                }
                // Bookmark radio
                // Del bookmarks
                (
                    COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK,
                    Msg::OnSubmit(Payload::Unsigned(index)),
                ) => {
                    // hide bookmark delete
                    self.umount_bookmark_del_dialog();
                    // Index must be 0 => YES
                    match *index {
                        0 => {
                            // Get selected bookmark
                            match self.view.get_value(COMPONENT_BOOKMARKS_LIST) {
                                Some(Payload::Unsigned(index)) => {
                                    // Delete bookmark
                                    self.del_bookmark(index);
                                    // Update bookmarks
                                    self.view_bookmarks()
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    }
                }
                (COMPONENT_RADIO_BOOKMARK_DEL_RECENT, Msg::OnSubmit(Payload::Unsigned(index))) => {
                    // hide bookmark delete
                    self.umount_recent_del_dialog();
                    // Index must be 0 => YES
                    match *index {
                        0 => {
                            // Get selected bookmark
                            match self.view.get_value(COMPONENT_RECENTS_LIST) {
                                Some(Payload::Unsigned(index)) => {
                                    // Delete recent
                                    self.del_recent(index);
                                    // Update bookmarks
                                    self.view_recent_connections()
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    }
                }
                // <ESC> hide tab
                (COMPONENT_RADIO_BOOKMARK_DEL_RECENT, &MSG_KEY_ESC) => {
                    match self
                        .view
                        .get_props(COMPONENT_RADIO_BOOKMARK_DEL_RECENT)
                        .as_mut()
                    {
                        Some(props) => {
                            let msg = self.view.update(
                                COMPONENT_RADIO_BOOKMARK_DEL_RECENT,
                                props.hidden().build(),
                            );
                            self.update(msg)
                        }
                        None => None,
                    }
                }
                (COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK, &MSG_KEY_ESC) => {
                    match self
                        .view
                        .get_props(COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK)
                        .as_mut()
                    {
                        Some(props) => {
                            let msg = self.view.update(
                                COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK,
                                props.hidden().build(),
                            );
                            self.update(msg)
                        }
                        None => None,
                    }
                }
                // Help
                (_, &MSG_KEY_CTRL_H) => {
                    // Show help
                    self.mount_help();
                    None
                }
                (COMPONENT_TEXT_HELP, &MSG_KEY_ENTER) | (COMPONENT_TEXT_HELP, &MSG_KEY_ESC) => {
                    // Hide text help
                    self.umount_help();
                    None
                }
                // Enter setup
                (_, &MSG_KEY_CTRL_C) => {
                    self.exit_reason = Some(super::ExitReason::EnterSetup);
                    None
                }
                // Save bookmark; show popup
                (_, &MSG_KEY_CTRL_S) => {
                    // Show popup
                    self.mount_bookmark_save_dialog();
                    // Give focus to bookmark name
                    self.view.active(COMPONENT_INPUT_BOOKMARK_NAME);
                    None
                }
                (COMPONENT_INPUT_BOOKMARK_NAME, &MSG_KEY_DOWN) => {
                    // Give focus to pwd
                    self.view.active(COMPONENT_RADIO_BOOKMARK_SAVE_PWD);
                    None
                }
                (COMPONENT_RADIO_BOOKMARK_SAVE_PWD, &MSG_KEY_UP) => {
                    // Give focus to pwd
                    self.view.active(COMPONENT_INPUT_BOOKMARK_NAME);
                    None
                }
                // Save bookmark
                (COMPONENT_INPUT_BOOKMARK_NAME, Msg::OnSubmit(_))
                | (COMPONENT_RADIO_BOOKMARK_SAVE_PWD, Msg::OnSubmit(_)) => {
                    // Get values
                    let bookmark_name: String =
                        match self.view.get_value(COMPONENT_INPUT_BOOKMARK_NAME) {
                            Some(Payload::Text(s)) => s,
                            _ => String::new(),
                        };
                    let save_pwd: bool = matches!(
                        self.view.get_value(COMPONENT_RADIO_BOOKMARK_SAVE_PWD),
                        Some(Payload::Unsigned(0))
                    );
                    // Save bookmark
                    self.save_bookmark(bookmark_name, save_pwd);
                    // Umount popup
                    self.umount_bookmark_save_dialog();
                    // Reload bookmarks
                    self.view_bookmarks()
                }
                // Hide save bookmark
                (COMPONENT_INPUT_BOOKMARK_NAME, &MSG_KEY_ESC)
                | (COMPONENT_RADIO_BOOKMARK_SAVE_PWD, &MSG_KEY_ESC) => {
                    // Umount popup
                    self.umount_bookmark_save_dialog();
                    None
                }
                // Error message
                (COMPONENT_TEXT_ERROR, &MSG_KEY_ENTER) => {
                    // Umount text error
                    self.umount_error();
                    None
                }
                // Quit dialog
                (COMPONENT_RADIO_QUIT, Msg::OnSubmit(Payload::Unsigned(choice))) => {
                    // If choice is 0, quit termscp
                    if *choice == 0 {
                        self.exit_reason = Some(super::ExitReason::Quit);
                    }
                    self.umount_quit();
                    None
                }
                (COMPONENT_RADIO_QUIT, &MSG_KEY_ESC) => {
                    self.umount_quit();
                    None
                }
                // On submit on any unhandled (connect)
                (_, Msg::OnSubmit(_)) | (_, &MSG_KEY_ENTER) => {
                    // Match <ENTER> key for all other components
                    self.save_recent();
                    let (address, port, protocol, username, password) = self.get_input();
                    // Set file transfer params to context
                    let mut ft_params: &mut FileTransferParams =
                        &mut self.context.as_mut().unwrap().ft_params.as_mut().unwrap();
                    ft_params.address = address;
                    ft_params.port = port;
                    ft_params.protocol = protocol;
                    ft_params.username = match username.is_empty() {
                        true => None,
                        false => Some(username),
                    };
                    ft_params.password = match password.is_empty() {
                        true => None,
                        false => Some(password),
                    };
                    // Set exit reason
                    self.exit_reason = Some(super::ExitReason::Connect);
                    // Return None
                    None
                }
                // <ESC> => Quit
                (_, &MSG_KEY_ESC) => {
                    self.mount_quit();
                    None
                }
                (_, _) => None, // Ignore other events
            },
        }
    }
}
