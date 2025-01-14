//! ## Context
//!
//! `Context` is the module which provides all the functionalities related to the UI data holder, called Context

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
// Dependencies
extern crate crossterm;
extern crate tui;

// Locals
use super::input::InputHandler;
use super::store::Store;
use crate::filetransfer::FileTransferProtocol;
use crate::host::Localhost;
use crate::system::config_client::ConfigClient;

// Includes
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use std::io::{stdout, Stdout};
use std::path::PathBuf;
use tui::backend::CrosstermBackend;
use tui::Terminal;

/// ## Context
///
/// Context holds data structures used by the ui
pub struct Context {
    pub local: Localhost,
    pub ft_params: Option<FileTransferParams>,
    pub(crate) config_client: Option<ConfigClient>,
    pub(crate) store: Store,
    pub(crate) input_hnd: InputHandler,
    pub(crate) terminal: Terminal<CrosstermBackend<Stdout>>,
    error: Option<String>,
}

/// ### FileTransferParams
///
/// Holds connection parameters for file transfers
pub struct FileTransferParams {
    pub address: String,
    pub port: u16,
    pub protocol: FileTransferProtocol,
    pub username: Option<String>,
    pub password: Option<String>,
    pub entry_directory: Option<PathBuf>,
}

impl Context {
    /// ### new
    ///
    /// Instantiates a new Context
    pub fn new(
        local: Localhost,
        config_client: Option<ConfigClient>,
        error: Option<String>,
    ) -> Context {
        // Create terminal
        let mut stdout = stdout();
        assert!(execute!(stdout, EnterAlternateScreen).is_ok());
        Context {
            local,
            ft_params: None,
            config_client,
            store: Store::init(),
            input_hnd: InputHandler::new(),
            terminal: Terminal::new(CrosstermBackend::new(stdout)).unwrap(),
            error,
        }
    }

    /* NOTE: in case is necessary
    /// ### set_error
    ///
    /// Set context error
    pub fn set_error(&mut self, err: String) {
        self.error = Some(err);
    }
    */

    /// ### get_error
    ///
    /// Get error message and remove it from the context
    pub fn get_error(&mut self) -> Option<String> {
        self.error.take()
    }

    /// ### enter_alternate_screen
    ///
    /// Enter alternate screen (gui window)
    pub fn enter_alternate_screen(&mut self) {
        let _ = execute!(
            self.terminal.backend_mut(),
            EnterAlternateScreen,
            DisableMouseCapture
        );
    }

    /// ### leave_alternate_screen
    ///
    /// Go back to normal screen (gui window)
    pub fn leave_alternate_screen(&mut self) {
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
    }

    /// ### clear_screen
    ///
    /// Clear terminal screen
    pub fn clear_screen(&mut self) {
        let _ = self.terminal.clear();
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // Re-enable terminal stuff
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
    }
}

impl Default for FileTransferParams {
    fn default() -> Self {
        Self {
            address: String::new(),
            port: 22,
            protocol: FileTransferProtocol::Sftp,
            username: None,
            password: None,
            entry_directory: None,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_ui_context_ft_params() {
        let params: FileTransferParams = FileTransferParams::default();
        assert_eq!(params.address.as_str(), "");
        assert_eq!(params.port, 22);
        assert_eq!(params.protocol, FileTransferProtocol::Sftp);
        assert!(params.username.is_none());
        assert!(params.password.is_none());
    }

    //use crate::filetransfer::sftp_transfer::SftpFileTransfer;
    //use std::path::PathBuf;

    /*
    #[test]
    fn test_ui_context_new() {
        // Prepare stuff
        Context::new(
            build_sftp_client(),
            Localhost::new(PathBuf::from("/")).ok().unwrap(),
        );
    }

    fn build_sftp_client() -> Box<dyn FileTransfer> {
        let mut sftp_client: SftpFileTransfer = SftpFileTransfer::new();
        // Connect to remote
        assert!(sftp_client
            .connect(
                String::from("test.rebex.net"),
                22,
                Some(String::from("demo")),
                Some(String::from("password"))
            )
            .is_ok());
        Box::new(sftp_client)
    }
    */
}
