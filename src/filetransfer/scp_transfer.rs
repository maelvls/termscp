//! ## SCP_Transfer
//!
//! `scps_transfer` is the module which provides the implementation for the SCP file transfer

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
extern crate regex;
extern crate ssh2;

// Locals
use super::{FileTransfer, FileTransferError, FileTransferErrorType};
use crate::fs::{FsDirectory, FsEntry, FsFile};
use crate::system::sshkey_storage::SshKeyStorage;
use crate::utils::parser::parse_lstime;

// Includes
use regex::Regex;
use ssh2::{Channel, Session};
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// ## ScpFileTransfer
///
/// SCP file transfer structure
pub struct ScpFileTransfer {
    session: Option<Session>,
    wrkdir: PathBuf,
    key_storage: SshKeyStorage,
}

impl ScpFileTransfer {
    /// ### new
    ///
    /// Instantiates a new ScpFileTransfer
    pub fn new(key_storage: SshKeyStorage) -> ScpFileTransfer {
        ScpFileTransfer {
            session: None,
            wrkdir: PathBuf::from("~"),
            key_storage,
        }
    }

    /// ### parse_ls_output
    ///
    /// Parse a line of `ls -l` output and tokenize the output into a `FsEntry`
    fn parse_ls_output(&mut self, path: &Path, line: &str) -> Result<FsEntry, ()> {
        // Prepare list regex
        // NOTE: about this damn regex <https://stackoverflow.com/questions/32480890/is-there-a-regex-to-parse-the-values-from-an-ftp-directory-listing>
        lazy_static! {
            static ref LS_RE: Regex = Regex::new(r#"^([\-ld])([\-rwxs]{9})\s+(\d+)\s+(\w+)\s+(\w+)\s+(\d+)\s+(\w{3}\s+\d{1,2}\s+(?:\d{1,2}:\d{1,2}|\d{4}))\s+(.+)$"#).unwrap();
        }
        // Apply regex to result
        match LS_RE.captures(line) {
            // String matches regex
            Some(metadata) => {
                // NOTE: metadata fmt: (regex, file_type, permissions, link_count, uid, gid, filesize, mtime, filename)
                // Expected 7 + 1 (8) values: + 1 cause regex is repeated at 0
                if metadata.len() < 8 {
                    return Err(());
                }
                // Collect metadata
                // Get if is directory and if is symlink
                let (mut is_dir, is_symlink): (bool, bool) = match metadata.get(1).unwrap().as_str()
                {
                    "-" => (false, false),
                    "l" => (false, true),
                    "d" => (true, false),
                    _ => return Err(()), // Ignore special files
                };
                // Check string length (unix pex)
                if metadata.get(2).unwrap().as_str().len() < 9 {
                    return Err(());
                }

                let pex = |range: Range<usize>| {
                    let mut count: u8 = 0;
                    for (i, c) in metadata.get(2).unwrap().as_str()[range].chars().enumerate() {
                        match c {
                            '-' => {}
                            _ => {
                                count += match i {
                                    0 => 4,
                                    1 => 2,
                                    2 => 1,
                                    _ => 0,
                                }
                            }
                        }
                    }
                    count
                };

                // Get unix pex
                let unix_pex = (pex(0..3), pex(3..6), pex(6..9));

                // Parse mtime and convert to SystemTime
                let mtime: SystemTime = match parse_lstime(
                    metadata.get(7).unwrap().as_str(),
                    "%b %d %Y",
                    "%b %d %H:%M",
                ) {
                    Ok(t) => t,
                    Err(_) => SystemTime::UNIX_EPOCH,
                };
                // Get uid
                let uid: Option<u32> = match metadata.get(4).unwrap().as_str().parse::<u32>() {
                    Ok(uid) => Some(uid),
                    Err(_) => None,
                };
                // Get gid
                let gid: Option<u32> = match metadata.get(5).unwrap().as_str().parse::<u32>() {
                    Ok(gid) => Some(gid),
                    Err(_) => None,
                };
                // Get filesize
                let filesize: usize = metadata
                    .get(6)
                    .unwrap()
                    .as_str()
                    .parse::<usize>()
                    .unwrap_or(0);
                // Get link and name
                let (file_name, symlink_path): (String, Option<PathBuf>) = match is_symlink {
                    true => self.get_name_and_link(metadata.get(8).unwrap().as_str()),
                    false => (String::from(metadata.get(8).unwrap().as_str()), None),
                };
                // Check if symlink points to a directory
                if let Some(symlink_path) = symlink_path.as_ref() {
                    is_dir = symlink_path.is_dir();
                }
                // Get symlink; PATH mustn't be equal to filename
                let symlink: Option<Box<FsEntry>> = match symlink_path {
                    None => None,
                    Some(p) => match p.file_name().unwrap_or(&std::ffi::OsStr::new(""))
                        == file_name.as_str()
                    {
                        // If name is equal, don't stat path; otherwise it would get stuck
                        true => None,
                        false => match self.stat(p.as_path()) {
                            // If path match filename
                            Ok(e) => Some(Box::new(e)),
                            Err(_) => None, // Ignore errors
                        },
                    },
                };
                // Check if file_name is '.' or '..'
                if file_name.as_str() == "." || file_name.as_str() == ".." {
                    return Err(());
                }
                let mut abs_path: PathBuf = PathBuf::from(path);
                abs_path.push(file_name.as_str());
                // Get extension
                let extension: Option<String> = abs_path
                    .as_path()
                    .extension()
                    .map(|s| String::from(s.to_string_lossy()));
                // Return
                // Push to entries
                Ok(match is_dir {
                    true => FsEntry::Directory(FsDirectory {
                        name: file_name,
                        abs_path,
                        last_change_time: mtime,
                        last_access_time: mtime,
                        creation_time: mtime,
                        readonly: false,
                        symlink,
                        user: uid,
                        group: gid,
                        unix_pex: Some(unix_pex),
                    }),
                    false => FsEntry::File(FsFile {
                        name: file_name,
                        abs_path,
                        last_change_time: mtime,
                        last_access_time: mtime,
                        creation_time: mtime,
                        size: filesize,
                        ftype: extension,
                        readonly: false,
                        symlink,
                        user: uid,
                        group: gid,
                        unix_pex: Some(unix_pex),
                    }),
                })
            }
            None => Err(()),
        }
    }

    /// ### get_name_and_link
    ///
    /// Returns from a `ls -l` command output file name token, the name of the file and the symbolic link (if there is any)
    fn get_name_and_link(&self, token: &str) -> (String, Option<PathBuf>) {
        let tokens: Vec<&str> = token.split(" -> ").collect();
        let filename: String = String::from(*tokens.get(0).unwrap());
        let symlink: Option<PathBuf> = tokens.get(1).map(PathBuf::from);
        (filename, symlink)
    }

    /// ### perform_shell_cmd_with
    ///
    /// Perform a shell command, but change directory to specified path first
    fn perform_shell_cmd_with_path(
        &mut self,
        path: &Path,
        cmd: &str,
    ) -> Result<String, FileTransferError> {
        self.perform_shell_cmd(format!("cd \"{}\"; {}", path.display(), cmd).as_str())
    }

    /// ### perform_shell_cmd
    ///
    /// Perform a shell command and read the output from shell
    /// This operation is, obviously, blocking.
    fn perform_shell_cmd(&mut self, cmd: &str) -> Result<String, FileTransferError> {
        match self.session.as_mut() {
            Some(session) => {
                // Create channel
                let mut channel: Channel = match session.channel_session() {
                    Ok(ch) => ch,
                    Err(err) => {
                        return Err(FileTransferError::new_ex(
                            FileTransferErrorType::ProtocolError,
                            format!("Could not open channel: {}", err),
                        ))
                    }
                };
                // Execute command
                if let Err(err) = channel.exec(cmd) {
                    return Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("Could not execute command \"{}\": {}", cmd, err),
                    ));
                }
                // Read output
                let mut output: String = String::new();
                match channel.read_to_string(&mut output) {
                    Ok(_) => {
                        // Wait close
                        let _ = channel.wait_close();
                        Ok(output)
                    }
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("Could not read output: {}", err),
                    )),
                }
            }
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }
}

impl FileTransfer for ScpFileTransfer {
    /// ### connect
    ///
    /// Connect to the remote server
    fn connect(
        &mut self,
        address: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<Option<String>, FileTransferError> {
        // Setup tcp stream
        let socket_addresses: Vec<SocketAddr> =
            match format!("{}:{}", address, port).to_socket_addrs() {
                Ok(s) => s.collect(),
                Err(err) => {
                    return Err(FileTransferError::new_ex(
                        FileTransferErrorType::BadAddress,
                        format!("{}", err),
                    ))
                }
            };
        let mut tcp: Option<TcpStream> = None;
        // Try addresses
        for socket_addr in socket_addresses.iter() {
            match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(30)) {
                Ok(stream) => {
                    tcp = Some(stream);
                    break;
                }
                Err(_) => continue,
            }
        }
        // If stream is None, return connection timeout
        let tcp: TcpStream = match tcp {
            Some(t) => t,
            None => {
                return Err(FileTransferError::new_ex(
                    FileTransferErrorType::ConnectionError,
                    String::from("Connection timeout"),
                ))
            }
        };
        // Create session
        let mut session: Session = match Session::new() {
            Ok(s) => s,
            Err(err) => {
                return Err(FileTransferError::new_ex(
                    FileTransferErrorType::ConnectionError,
                    format!("{}", err),
                ))
            }
        };
        // Set TCP stream
        session.set_tcp_stream(tcp);
        // Open connection
        if let Err(err) = session.handshake() {
            return Err(FileTransferError::new_ex(
                FileTransferErrorType::ConnectionError,
                format!("{}", err),
            ));
        }
        let username: String = match username {
            Some(u) => u,
            None => String::from(""),
        };
        // Check if it is possible to authenticate using a RSA key
        match self
            .key_storage
            .resolve(address.as_str(), username.as_str())
        {
            Some(rsa_key) => {
                // Authenticate with RSA key
                if let Err(err) = session.userauth_pubkey_file(
                    username.as_str(),
                    None,
                    rsa_key.as_path(),
                    password.as_deref(),
                ) {
                    return Err(FileTransferError::new_ex(
                        FileTransferErrorType::AuthenticationFailed,
                        format!("{}", err),
                    ));
                }
            }
            None => {
                // Proceeed with username/password authentication
                if let Err(err) = session.userauth_password(
                    username.as_str(),
                    password.unwrap_or_else(|| String::from("")).as_str(),
                ) {
                    return Err(FileTransferError::new_ex(
                        FileTransferErrorType::AuthenticationFailed,
                        format!("{}", err),
                    ));
                }
            }
        }
        // Get banner
        let banner: Option<String> = session.banner().map(String::from);
        // Set session
        self.session = Some(session);
        // Get working directory
        match self.perform_shell_cmd("pwd") {
            Ok(output) => self.wrkdir = PathBuf::from(output.as_str().trim()),
            Err(err) => return Err(err),
        }
        Ok(banner)
    }

    /// ### disconnect
    ///
    /// Disconnect from the remote server
    fn disconnect(&mut self) -> Result<(), FileTransferError> {
        match self.session.as_ref() {
            Some(session) => {
                // Disconnect (greet server with 'Mandi' as they do in Friuli)
                match session.disconnect(None, "Mandi!", None) {
                    Ok(()) => {
                        // Set session to none
                        self.session = None;
                        Ok(())
                    }
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ConnectionError,
                        format!("{}", err),
                    )),
                }
            }
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### is_connected
    ///
    /// Indicates whether the client is connected to remote
    fn is_connected(&self) -> bool {
        self.session.as_ref().is_some()
    }

    /// ### pwd
    ///
    /// Print working directory

    fn pwd(&mut self) -> Result<PathBuf, FileTransferError> {
        match self.is_connected() {
            true => Ok(self.wrkdir.clone()),
            false => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### change_dir
    ///
    /// Change working directory

    fn change_dir(&mut self, dir: &Path) -> Result<PathBuf, FileTransferError> {
        match self.is_connected() {
            true => {
                let p: PathBuf = self.wrkdir.clone();
                let remote_path: PathBuf = match dir.is_absolute() {
                    true => PathBuf::from(dir),
                    false => {
                        let mut p: PathBuf = PathBuf::from(".");
                        p.push(dir);
                        p
                    }
                };
                // Change directory
                match self.perform_shell_cmd_with_path(
                    p.as_path(),
                    format!("cd \"{}\"; echo $?; pwd", remote_path.display()).as_str(),
                ) {
                    Ok(output) => {
                        // Trim
                        let output: String = String::from(output.as_str().trim());
                        // Check if output starts with 0; should be 0{PWD}
                        match output.as_str().starts_with('0') {
                            true => {
                                // Set working directory
                                self.wrkdir = PathBuf::from(&output.as_str()[1..].trim());
                                Ok(self.wrkdir.clone())
                            }
                            false => Err(FileTransferError::new_ex(
                                // No such file or directory
                                FileTransferErrorType::NoSuchFileOrDirectory,
                                format!("\"{}\"", dir.display()),
                            )),
                        }
                    }
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("{}", err),
                    )),
                }
            }
            false => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### copy
    ///
    /// Copy file to destination
    fn copy(&mut self, src: &FsEntry, dst: &Path) -> Result<(), FileTransferError> {
        match self.is_connected() {
            true => {
                // Run `cp -rf`
                let p: PathBuf = self.wrkdir.clone();
                match self.perform_shell_cmd_with_path(
                    p.as_path(),
                    format!(
                        "cp -rf \"{}\" \"{}\"; echo $?",
                        src.get_abs_path().display(),
                        dst.display()
                    )
                    .as_str(),
                ) {
                    Ok(output) =>
                    // Check if output is 0
                    {
                        match output.as_str().trim() == "0" {
                            true => Ok(()), // File copied
                            false => Err(FileTransferError::new_ex(
                                // Could not copy file
                                FileTransferErrorType::FileCreateDenied,
                                format!("\"{}\"", dst.display()),
                            )),
                        }
                    }
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("{}", err),
                    )),
                }
            }
            false => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### list_dir
    ///
    /// List directory entries

    fn list_dir(&mut self, path: &Path) -> Result<Vec<FsEntry>, FileTransferError> {
        match self.is_connected() {
            true => {
                // Send ls -l to path
                let p: PathBuf = self.wrkdir.clone();
                match self.perform_shell_cmd_with_path(
                    p.as_path(),
                    format!("unset LANG; ls -la \"{}\"", path.display()).as_str(),
                ) {
                    Ok(output) => {
                        // Split output by (\r)\n
                        let lines: Vec<&str> = output.as_str().lines().collect();
                        let mut entries: Vec<FsEntry> = Vec::with_capacity(lines.len());
                        for line in lines.iter() {
                            // First line must always be ignored
                            // Parse row, if ok push to entries
                            if let Ok(entry) = self.parse_ls_output(path, line) {
                                entries.push(entry);
                            }
                        }
                        Ok(entries)
                    }
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("{}", err),
                    )),
                }
            }
            false => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### mkdir
    ///
    /// Make directory
    /// You must return error in case the directory already exists
    fn mkdir(&mut self, dir: &Path) -> Result<(), FileTransferError> {
        match self.is_connected() {
            true => {
                let p: PathBuf = self.wrkdir.clone();
                // Mkdir dir && echo 0
                match self.perform_shell_cmd_with_path(
                    p.as_path(),
                    format!("mkdir \"{}\"; echo $?", dir.display()).as_str(),
                ) {
                    Ok(output) => {
                        // Check if output is 0
                        match output.as_str().trim() == "0" {
                            true => Ok(()), // Directory created
                            false => Err(FileTransferError::new_ex(
                                // Could not create directory
                                FileTransferErrorType::FileCreateDenied,
                                format!("\"{}\"", dir.display()),
                            )),
                        }
                    }
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("{}", err),
                    )),
                }
            }
            false => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### remove
    ///
    /// Remove a file or a directory
    fn remove(&mut self, file: &FsEntry) -> Result<(), FileTransferError> {
        // Yay, we have rm -rf here :D
        match self.is_connected() {
            true => {
                // Get path
                let path: PathBuf = file.get_abs_path();
                let p: PathBuf = self.wrkdir.clone();
                match self.perform_shell_cmd_with_path(
                    p.as_path(),
                    format!("rm -rf \"{}\"; echo $?", path.display()).as_str(),
                ) {
                    Ok(output) => {
                        // Check if output is 0
                        match output.as_str().trim() == "0" {
                            true => Ok(()), // Directory created
                            false => Err(FileTransferError::new_ex(
                                // Could not create directory
                                FileTransferErrorType::PexError,
                                format!("\"{}\"", path.display()),
                            )),
                        }
                    }
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("{}", err),
                    )),
                }
            }
            false => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### rename
    ///
    /// Rename file or a directory
    fn rename(&mut self, file: &FsEntry, dst: &Path) -> Result<(), FileTransferError> {
        match self.is_connected() {
            true => {
                // Get path
                let path: PathBuf = file.get_abs_path();
                let p: PathBuf = self.wrkdir.clone();
                match self.perform_shell_cmd_with_path(
                    p.as_path(),
                    format!(
                        "mv -f \"{}\" \"{}\"; echo $?",
                        path.display(),
                        dst.display()
                    )
                    .as_str(),
                ) {
                    Ok(output) => {
                        // Check if output is 0
                        match output.as_str().trim() == "0" {
                            true => Ok(()), // File renamed
                            false => Err(FileTransferError::new_ex(
                                // Could not move file
                                FileTransferErrorType::PexError,
                                format!("\"{}\"", path.display()),
                            )),
                        }
                    }
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("{}", err),
                    )),
                }
            }
            false => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### stat
    ///
    /// Stat file and return FsEntry
    fn stat(&mut self, path: &Path) -> Result<FsEntry, FileTransferError> {
        if path.is_dir() {
            return Err(FileTransferError::new_ex(
                FileTransferErrorType::UnsupportedFeature,
                String::from("stat is not supported for directories"),
            ));
        }
        let path: PathBuf = match path.is_absolute() {
            true => PathBuf::from(path),
            false => {
                let mut p: PathBuf = self.wrkdir.clone();
                p.push(path);
                p
            }
        };
        match self.is_connected() {
            true => {
                let p: PathBuf = self.wrkdir.clone();
                match self.perform_shell_cmd_with_path(
                    p.as_path(),
                    format!("ls -l \"{}\"", path.display()).as_str(),
                ) {
                    Ok(line) => {
                        // Parse ls line
                        let parent: PathBuf = match path.as_path().parent() {
                            Some(p) => PathBuf::from(p),
                            None => {
                                return Err(FileTransferError::new_ex(
                                    FileTransferErrorType::UnsupportedFeature,
                                    String::from("Path has no parent"),
                                ))
                            }
                        };
                        match self.parse_ls_output(parent.as_path(), line.as_str().trim()) {
                            Ok(entry) => Ok(entry),
                            Err(_) => Err(FileTransferError::new(
                                FileTransferErrorType::NoSuchFileOrDirectory,
                            )),
                        }
                    }
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("{}", err),
                    )),
                }
            }
            false => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### exec
    ///
    /// Execute a command on remote host
    fn exec(&mut self, cmd: &str) -> Result<String, FileTransferError> {
        match self.is_connected() {
            true => {
                let p: PathBuf = self.wrkdir.clone();
                match self.perform_shell_cmd_with_path(p.as_path(), cmd) {
                    Ok(output) => Ok(output),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("{}", err),
                    )),
                }
            }
            false => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### send_file
    ///
    /// Send file to remote
    /// File name is referred to the name of the file as it will be saved
    /// Data contains the file data
    /// Returns file and its size
    fn send_file(
        &mut self,
        local: &FsFile,
        file_name: &Path,
    ) -> Result<Box<dyn Write>, FileTransferError> {
        match self.session.as_ref() {
            Some(session) => {
                // Set blocking to true
                session.set_blocking(true);
                // Calculate file mode
                let mode: i32 = match local.unix_pex {
                    None => 0o644,
                    Some((u, g, o)) => ((u as i32) << 6) + ((g as i32) << 3) + (o as i32),
                };
                // Calculate mtime, atime
                let times: (u64, u64) = {
                    let mtime: u64 = match local
                        .last_change_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                    {
                        Ok(durr) => durr.as_secs() as u64,
                        Err(_) => 0,
                    };
                    let atime: u64 = match local
                        .last_access_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                    {
                        Ok(durr) => durr.as_secs() as u64,
                        Err(_) => 0,
                    };
                    (mtime, atime)
                };
                // We need to get the size of local; NOTE: don't use the `size` attribute, since might be out of sync
                let file_size: u64 = match std::fs::metadata(local.abs_path.as_path()) {
                    Ok(metadata) => metadata.len(),
                    Err(_) => local.size as u64, // NOTE: fallback to fsentry size
                };
                // Send file
                match session.scp_send(file_name, mode, file_size, Some(times)) {
                    Ok(channel) => Ok(Box::new(BufWriter::with_capacity(65536, channel))),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("{}", err),
                    )),
                }
            }
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### recv_file
    ///
    /// Receive file from remote with provided name
    /// Returns file and its size
    fn recv_file(&mut self, file: &FsFile) -> Result<Box<dyn Read>, FileTransferError> {
        match self.session.as_ref() {
            Some(session) => {
                // Set blocking to true
                session.set_blocking(true);
                match session.scp_recv(file.abs_path.as_path()) {
                    Ok(reader) => Ok(Box::new(BufReader::with_capacity(65536, reader.0))),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("{}", err),
                    )),
                }
            }
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### on_sent
    ///
    /// Finalize send method.
    /// This method must be implemented only if necessary; in case you don't need it, just return `Ok(())`
    /// The purpose of this method is to finalize the connection with the peer when writing data.
    /// This is necessary for some protocols such as FTP.
    /// You must call this method each time you want to finalize the write of the remote file.
    fn on_sent(&mut self, _writable: Box<dyn Write>) -> Result<(), FileTransferError> {
        // Nothing to do
        Ok(())
    }

    /// ### on_recv
    ///
    /// Finalize recv method.
    /// This method must be implemented only if necessary; in case you don't need it, just return `Ok(())`
    /// The purpose of this method is to finalize the connection with the peer when reading data.
    /// This mighe be necessary for some protocols.
    /// You must call this method each time you want to finalize the read of the remote file.
    fn on_recv(&mut self, _readable: Box<dyn Read>) -> Result<(), FileTransferError> {
        // Nothing to do
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_filetransfer_scp_new() {
        let client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client.session.is_none());
        assert_eq!(client.is_connected(), false);
    }

    #[test]
    fn test_filetransfer_scp_connect() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert_eq!(client.is_connected(), false);
        assert!(client
            .connect(
                String::from("test.rebex.net"),
                22,
                Some(String::from("demo")),
                Some(String::from("password"))
            )
            .is_ok());
        // Check session and scp
        assert!(client.session.is_some());
        assert_eq!(client.is_connected(), true);
        // Disconnect
        assert!(client.disconnect().is_ok());
        assert_eq!(client.is_connected(), false);
    }
    #[test]
    fn test_filetransfer_scp_bad_auth() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("test.rebex.net"),
                22,
                Some(String::from("demo")),
                Some(String::from("badpassword"))
            )
            .is_err());
    }

    #[test]
    fn test_filetransfer_scp_no_credentials() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(String::from("test.rebex.net"), 22, None, None)
            .is_err());
    }

    #[test]
    fn test_filetransfer_scp_bad_server() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("mybadserver.veryverybad.awful"),
                22,
                None,
                None
            )
            .is_err());
    }
    #[test]
    fn test_filetransfer_scp_pwd() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("test.rebex.net"),
                22,
                Some(String::from("demo")),
                Some(String::from("password"))
            )
            .is_ok());
        // Check session and scp
        assert!(client.session.is_some());
        // Pwd
        assert_eq!(client.pwd().ok().unwrap(), PathBuf::from("/"));
        // Disconnect
        assert!(client.disconnect().is_ok());
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn test_filetransfer_scp_cwd() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("test.rebex.net"),
                22,
                Some(String::from("demo")),
                Some(String::from("password"))
            )
            .is_ok());
        // Check session and scp
        assert!(client.session.is_some());
        // Cwd (relative)
        assert!(client.change_dir(PathBuf::from("pub/").as_path()).is_ok());
        // Cwd (absolute)
        assert!(client.change_dir(PathBuf::from("/pub").as_path()).is_ok());
        // Disconnect
        assert!(client.disconnect().is_ok());
    }

    #[test]
    fn test_filetransfer_scp_cwd_error() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("test.rebex.net"),
                22,
                Some(String::from("demo")),
                Some(String::from("password"))
            )
            .is_ok());
        // Cwd (abs)
        assert!(client
            .change_dir(PathBuf::from("/omar/gabber").as_path())
            .is_err());
        // Cwd (rel)
        assert!(client
            .change_dir(PathBuf::from("gomar/pett").as_path())
            .is_err());
        // Disconnect
        assert!(client.disconnect().is_ok());
    }

    #[test]
    fn test_filetransfer_scp_ls() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("test.rebex.net"),
                22,
                Some(String::from("demo")),
                Some(String::from("password"))
            )
            .is_ok());
        // Check session and scp
        assert!(client.session.is_some());
        // List dir
        let pwd: PathBuf = client.pwd().ok().unwrap();
        let files: Vec<FsEntry> = client.list_dir(pwd.as_path()).ok().unwrap();
        assert_eq!(files.len(), 3); // There are 3 files
                                    // Disconnect
        assert!(client.disconnect().is_ok());
    }

    #[test]
    fn test_filetransfer_scp_stat() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("test.rebex.net"),
                22,
                Some(String::from("demo")),
                Some(String::from("password"))
            )
            .is_ok());
        // Check session and scp
        assert!(client.session.is_some());
        let file: FsEntry = client
            .stat(PathBuf::from("readme.txt").as_path())
            .ok()
            .unwrap();
        if let FsEntry::File(file) = file {
            assert_eq!(file.abs_path, PathBuf::from("/readme.txt"));
        } else {
            panic!("Expected readme.txt to be a file");
        }
    }

    #[test]
    fn test_filetransfer_scp_exec() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("test.rebex.net"),
                22,
                Some(String::from("demo")),
                Some(String::from("password"))
            )
            .is_ok());
        // Check session and scp
        assert!(client.session.is_some());
        // Exec
        assert_eq!(client.exec("echo 5").ok().unwrap().as_str(), "5\n");
        // Disconnect
        assert!(client.disconnect().is_ok());
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn test_filetransfer_scp_find() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("test.rebex.net"),
                22,
                Some(String::from("demo")),
                Some(String::from("password"))
            )
            .is_ok());
        // Check session and scp
        assert!(client.session.is_some());
        // Search for file (let's search for pop3-*.png); there should be 2
        let search_res: Vec<FsEntry> = client.find("pop3-*.png").ok().unwrap();
        assert_eq!(search_res.len(), 2);
        // verify names
        assert_eq!(search_res[0].get_name(), "pop3-browser.png");
        assert_eq!(search_res[1].get_name(), "pop3-console-client.png");
        // Search directory
        let search_res: Vec<FsEntry> = client.find("pub").ok().unwrap();
        assert_eq!(search_res.len(), 1);
        // Disconnect
        assert!(client.disconnect().is_ok());
        // Verify err
        assert!(client.find("pippo").is_err());
    }

    #[test]
    fn test_filetransfer_scp_recv() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("test.rebex.net"),
                22,
                Some(String::from("demo")),
                Some(String::from("password"))
            )
            .is_ok());
        // Check session and scp
        assert!(client.session.is_some());
        let file: FsFile = FsFile {
            name: String::from("readme.txt"),
            abs_path: PathBuf::from("/readme.txt"),
            last_change_time: SystemTime::UNIX_EPOCH,
            last_access_time: SystemTime::UNIX_EPOCH,
            creation_time: SystemTime::UNIX_EPOCH,
            size: 0,
            ftype: Some(String::from("txt")), // File type
            readonly: true,
            symlink: None,             // UNIX only
            user: Some(0),             // UNIX only
            group: Some(0),            // UNIX only
            unix_pex: Some((6, 4, 4)), // UNIX only
        };
        // Receive file
        assert!(client.recv_file(&file).is_ok());
        // Disconnect
        assert!(client.disconnect().is_ok());
    }
    #[test]
    fn test_filetransfer_scp_recv_failed_nosuchfile() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("test.rebex.net"),
                22,
                Some(String::from("demo")),
                Some(String::from("password"))
            )
            .is_ok());
        // Check session and scp
        assert!(client.session.is_some());
        // Receive file
        let file: FsFile = FsFile {
            name: String::from("omar.txt"),
            abs_path: PathBuf::from("/omar.txt"),
            last_change_time: SystemTime::UNIX_EPOCH,
            last_access_time: SystemTime::UNIX_EPOCH,
            creation_time: SystemTime::UNIX_EPOCH,
            size: 0,
            ftype: Some(String::from("txt")), // File type
            readonly: true,
            symlink: None,             // UNIX only
            user: Some(0),             // UNIX only
            group: Some(0),            // UNIX only
            unix_pex: Some((6, 4, 4)), // UNIX only
        };
        assert!(client.recv_file(&file).is_err());
        // Disconnect
        assert!(client.disconnect().is_ok());
    }
    // NOTE: other functions doesn't work with this test scp server

    /* NOTE: the server doesn't allow you to create directories
    #[test]
    fn test_filetransfer_scp_mkdir() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client.connect(String::from("test.rebex.net"), 22, Some(String::from("demo")), Some(String::from("password"))).is_ok());
        let dir: String = String::from("foo");
        // Mkdir
        assert!(client.mkdir(dir).is_ok());
        // cwd
        assert!(client.change_dir(PathBuf::from("foo/").as_path()).is_ok());
        assert_eq!(client.wrkdir, PathBuf::from("/foo"));
        // Disconnect
        assert!(client.disconnect().is_ok());
    }
    */

    #[test]
    fn test_filetransfer_scp_uninitialized() {
        let file: FsFile = FsFile {
            name: String::from("omar.txt"),
            abs_path: PathBuf::from("/omar.txt"),
            last_change_time: SystemTime::UNIX_EPOCH,
            last_access_time: SystemTime::UNIX_EPOCH,
            creation_time: SystemTime::UNIX_EPOCH,
            size: 0,
            ftype: Some(String::from("txt")), // File type
            readonly: true,
            symlink: None,             // UNIX only
            user: Some(0),             // UNIX only
            group: Some(0),            // UNIX only
            unix_pex: Some((6, 4, 4)), // UNIX only
        };
        let mut scp: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(scp.change_dir(Path::new("/tmp")).is_err());
        assert!(scp.disconnect().is_err());
        assert!(scp.list_dir(Path::new("/tmp")).is_err());
        assert!(scp.mkdir(Path::new("/tmp")).is_err());
        assert!(scp.pwd().is_err());
        assert!(scp.stat(Path::new("/tmp")).is_err());
        assert!(scp.recv_file(&file).is_err());
        assert!(scp.send_file(&file, Path::new("/tmp/omar.txt")).is_err());
    }
}
