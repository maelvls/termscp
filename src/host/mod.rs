//! ## Host
//!
//! `host` is the module which provides functionalities to host file system

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
// dependencies
extern crate wildmatch;
// ext
use std::fs::{self, File, Metadata, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use wildmatch::WildMatch;
// Metadata ext
#[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
use std::fs::set_permissions;
#[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

// Locals
use crate::fs::{FsDirectory, FsEntry, FsFile};

/// ## HostErrorType
///
/// HostErrorType provides an overview of the specific host error
#[derive(PartialEq, std::fmt::Debug)]
pub enum HostErrorType {
    NoSuchFileOrDirectory,
    ReadonlyFile,
    DirNotAccessible,
    FileNotAccessible,
    FileAlreadyExists,
    CouldNotCreateFile,
    ExecutionFailed,
    DeleteFailed,
}

/// ### HostError
///
/// HostError is a wrapper for the error type and the exact io error

pub struct HostError {
    pub error: HostErrorType,
    pub ioerr: Option<std::io::Error>,
}

impl HostError {
    /// ### new
    ///
    /// Instantiates a new HostError
    pub(crate) fn new(error: HostErrorType, errno: Option<std::io::Error>) -> HostError {
        HostError {
            error,
            ioerr: errno,
        }
    }
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let code_str: &str = match self.error {
            HostErrorType::NoSuchFileOrDirectory => "No such file or directory",
            HostErrorType::ReadonlyFile => "File is readonly",
            HostErrorType::DirNotAccessible => "Could not access directory",
            HostErrorType::FileNotAccessible => "Could not access file",
            HostErrorType::FileAlreadyExists => "File already exists",
            HostErrorType::CouldNotCreateFile => "Could not create file",
            HostErrorType::ExecutionFailed => "Could not run command",
            HostErrorType::DeleteFailed => "Could not delete file",
        };
        match &self.ioerr {
            Some(err) => write!(f, "{}: {}", code_str, err),
            None => write!(f, "{}", code_str),
        }
    }
}

/// ## Localhost
///
/// Localhost is the entity which holds the information about the current directory and host.
/// It provides functions to navigate across the local host file system
pub struct Localhost {
    wrkdir: PathBuf,
    files: Vec<FsEntry>,
}

impl Localhost {
    /// ### new
    ///
    /// Instantiates a new Localhost struct
    pub fn new(wrkdir: PathBuf) -> Result<Localhost, HostError> {
        let mut host: Localhost = Localhost {
            wrkdir,
            files: Vec::new(),
        };
        // Check if dir exists
        if !host.file_exists(host.wrkdir.as_path()) {
            return Err(HostError::new(HostErrorType::NoSuchFileOrDirectory, None));
        }
        // Retrieve files for provided path
        host.files = match host.scan_dir(host.wrkdir.as_path()) {
            Ok(files) => files,
            Err(err) => return Err(err),
        };
        Ok(host)
    }

    /// ### pwd
    ///
    /// Print working directory
    pub fn pwd(&self) -> PathBuf {
        self.wrkdir.clone()
    }

    /// ### list_dir
    ///
    /// List files in current directory
    #[allow(dead_code)]
    pub fn list_dir(&self) -> Vec<FsEntry> {
        self.files.clone()
    }

    /// ### change_wrkdir
    ///
    /// Change working directory with the new provided directory
    pub fn change_wrkdir(&mut self, new_dir: &Path) -> Result<PathBuf, HostError> {
        let new_dir: PathBuf = self.to_abs_path(new_dir);
        // Check whether directory exists
        if !self.file_exists(new_dir.as_path()) {
            return Err(HostError::new(HostErrorType::NoSuchFileOrDirectory, None));
        }
        // Change directory
        if std::env::set_current_dir(new_dir.as_path()).is_err() {
            return Err(HostError::new(HostErrorType::NoSuchFileOrDirectory, None));
        }
        let prev_dir: PathBuf = self.wrkdir.clone(); // Backup location
                                                     // Update working directory
                                                     // Change dir
        self.wrkdir = new_dir;
        // Scan new directory
        self.files = match self.scan_dir(self.wrkdir.as_path()) {
            Ok(files) => files,
            Err(err) => {
                // Restore directory
                self.wrkdir = prev_dir;
                return Err(err);
            }
        };
        Ok(self.wrkdir.clone())
    }

    /// ### mkdir
    ///
    /// Make a directory at path and update the file list (only if relative)
    pub fn mkdir(&mut self, dir_name: &Path) -> Result<(), HostError> {
        self.mkdir_ex(dir_name, false)
    }

    /// ### mkdir_ex
    ///
    /// Extended option version of makedir.
    /// ignex: don't report error if directory already exists
    pub fn mkdir_ex(&mut self, dir_name: &Path, ignex: bool) -> Result<(), HostError> {
        let dir_path: PathBuf = self.to_abs_path(dir_name);
        // If dir already exists, return Error
        if dir_path.exists() {
            match ignex {
                true => return Ok(()),
                false => return Err(HostError::new(HostErrorType::FileAlreadyExists, None)),
            }
        }
        match std::fs::create_dir(dir_path) {
            Ok(_) => {
                // Update dir
                if dir_name.is_relative() {
                    self.files = self.scan_dir(self.wrkdir.as_path())?;
                }
                Ok(())
            }
            Err(err) => Err(HostError::new(HostErrorType::CouldNotCreateFile, Some(err))),
        }
    }

    /// ### remove
    ///
    /// Remove file entry
    pub fn remove(&mut self, entry: &FsEntry) -> Result<(), HostError> {
        match entry {
            FsEntry::Directory(dir) => {
                // If file doesn't exist; return error
                if !dir.abs_path.as_path().exists() {
                    return Err(HostError::new(HostErrorType::NoSuchFileOrDirectory, None));
                }
                // Remove
                match std::fs::remove_dir_all(dir.abs_path.as_path()) {
                    Ok(_) => {
                        // Update dir
                        self.files = self.scan_dir(self.wrkdir.as_path())?;
                        Ok(())
                    }
                    Err(err) => Err(HostError::new(HostErrorType::DeleteFailed, Some(err))),
                }
            }
            FsEntry::File(file) => {
                // If file doesn't exist; return error
                if !file.abs_path.as_path().exists() {
                    return Err(HostError::new(HostErrorType::NoSuchFileOrDirectory, None));
                }
                // Remove
                match std::fs::remove_file(file.abs_path.as_path()) {
                    Ok(_) => {
                        // Update dir
                        self.files = self.scan_dir(self.wrkdir.as_path())?;
                        Ok(())
                    }
                    Err(err) => Err(HostError::new(HostErrorType::DeleteFailed, Some(err))),
                }
            }
        }
    }

    /// ### rename
    ///
    /// Rename file or directory to new name
    pub fn rename(&mut self, entry: &FsEntry, dst_path: &Path) -> Result<(), HostError> {
        let abs_path: PathBuf = entry.get_abs_path();
        match std::fs::rename(abs_path.as_path(), dst_path) {
            Ok(_) => {
                // Scan dir
                self.files = self.scan_dir(self.wrkdir.as_path())?;
                Ok(())
            }
            Err(err) => Err(HostError::new(HostErrorType::CouldNotCreateFile, Some(err))),
        }
    }

    /// ### copy
    ///
    /// Copy file to destination path
    pub fn copy(&mut self, entry: &FsEntry, dst: &Path) -> Result<(), HostError> {
        // Get absolute path of dest
        let dst: PathBuf = self.to_abs_path(dst);
        // Match entry
        match entry {
            FsEntry::File(file) => {
                // Copy file
                // If destination path is a directory, push file name
                let dst: PathBuf = match dst.as_path().is_dir() {
                    true => {
                        let mut p: PathBuf = dst.clone();
                        p.push(file.name.as_str());
                        p
                    }
                    false => dst.clone(),
                };
                // Copy entry path to dst path
                if let Err(err) = std::fs::copy(file.abs_path.as_path(), dst.as_path()) {
                    return Err(HostError::new(HostErrorType::CouldNotCreateFile, Some(err)));
                }
            }
            FsEntry::Directory(dir) => {
                // If destination path doesn't exist, create destination
                if !dst.exists() {
                    self.mkdir(dst.as_path())?;
                }
                // Scan dir
                let dir_files: Vec<FsEntry> = self.scan_dir(dir.abs_path.as_path())?;
                // Iterate files
                for dir_entry in dir_files.iter() {
                    // Calculate dst
                    let mut sub_dst: PathBuf = dst.clone();
                    sub_dst.push(dir_entry.get_name());
                    // Call function recursively
                    self.copy(dir_entry, sub_dst.as_path())?;
                }
            }
        }
        // Reload directory if dst is pwd
        match dst.is_dir() {
            true => {
                if dst == self.pwd().as_path() {
                    self.files = self.scan_dir(self.wrkdir.as_path())?;
                } else if let Some(parent) = dst.parent() {
                    // If parent is pwd, scan directory
                    if parent == self.pwd().as_path() {
                        self.files = self.scan_dir(self.wrkdir.as_path())?;
                    }
                }
            }
            false => {
                if let Some(parent) = dst.parent() {
                    // If parent is pwd, scan directory
                    if parent == self.pwd().as_path() {
                        self.files = self.scan_dir(self.wrkdir.as_path())?;
                    }
                }
            }
        }
        Ok(())
    }

    /// ### stat
    ///
    /// Stat file and create a FsEntry
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    pub fn stat(&self, path: &Path) -> Result<FsEntry, HostError> {
        let path: PathBuf = self.to_abs_path(path);
        let attr: Metadata = match fs::metadata(path.as_path()) {
            Ok(metadata) => metadata,
            Err(err) => return Err(HostError::new(HostErrorType::FileNotAccessible, Some(err))),
        };
        let file_name: String = String::from(path.file_name().unwrap().to_str().unwrap_or(""));
        // Match dir / file
        Ok(match path.is_dir() {
            true => FsEntry::Directory(FsDirectory {
                name: file_name,
                abs_path: path.clone(),
                last_change_time: attr.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                last_access_time: attr.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
                creation_time: attr.created().unwrap_or(SystemTime::UNIX_EPOCH),
                readonly: attr.permissions().readonly(),
                symlink: match fs::read_link(path.as_path()) {
                    Ok(p) => match self.stat(p.as_path()) {
                        Ok(entry) => Some(Box::new(entry)),
                        Err(_) => None,
                    },
                    Err(_) => None,
                },
                user: Some(attr.uid()),
                group: Some(attr.gid()),
                unix_pex: Some(self.u32_to_mode(attr.mode())),
            }),
            false => {
                // Is File
                let extension: Option<String> = path
                    .extension()
                    .map(|s| String::from(s.to_str().unwrap_or("")));
                FsEntry::File(FsFile {
                    name: file_name,
                    abs_path: path.clone(),
                    last_change_time: attr.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    last_access_time: attr.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
                    creation_time: attr.created().unwrap_or(SystemTime::UNIX_EPOCH),
                    readonly: attr.permissions().readonly(),
                    size: attr.len() as usize,
                    ftype: extension,
                    symlink: match fs::read_link(path.as_path()) {
                        Ok(p) => match self.stat(p.as_path()) {
                            Ok(entry) => Some(Box::new(entry)),
                            Err(_) => None,
                        },
                        Err(_) => None, // Ignore errors
                    },
                    user: Some(attr.uid()),
                    group: Some(attr.gid()),
                    unix_pex: Some(self.u32_to_mode(attr.mode())),
                })
            }
        })
    }

    /// ### stat
    ///
    /// Stat file and create a FsEntry
    #[cfg(target_os = "windows")]
    #[cfg(not(tarpaulin_include))]
    pub fn stat(&self, path: &Path) -> Result<FsEntry, HostError> {
        let path: PathBuf = self.to_abs_path(path);
        let attr: Metadata = match fs::metadata(path.as_path()) {
            Ok(metadata) => metadata,
            Err(err) => return Err(HostError::new(HostErrorType::FileNotAccessible, Some(err))),
        };
        let file_name: String = String::from(path.file_name().unwrap().to_str().unwrap_or(""));
        // Match dir / file
        Ok(match path.is_dir() {
            true => FsEntry::Directory(FsDirectory {
                name: file_name,
                abs_path: path.clone(),
                last_change_time: attr.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                last_access_time: attr.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
                creation_time: attr.created().unwrap_or(SystemTime::UNIX_EPOCH),
                readonly: attr.permissions().readonly(),
                symlink: match fs::read_link(path.as_path()) {
                    Ok(p) => match self.stat(p.as_path()) {
                        Ok(entry) => Some(Box::new(entry)),
                        Err(_) => None, // Ignore errors
                    },
                    Err(_) => None,
                },
                user: None,
                group: None,
                unix_pex: None,
            }),
            false => {
                // Is File
                let extension: Option<String> = match path.extension() {
                    Some(s) => Some(String::from(s.to_str().unwrap_or(""))),
                    None => None,
                };
                FsEntry::File(FsFile {
                    name: file_name,
                    abs_path: path.clone(),
                    last_change_time: attr.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    last_access_time: attr.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
                    creation_time: attr.created().unwrap_or(SystemTime::UNIX_EPOCH),
                    readonly: attr.permissions().readonly(),
                    size: attr.len() as usize,
                    ftype: extension,
                    symlink: match fs::read_link(path.as_path()) {
                        Ok(p) => match self.stat(p.as_path()) {
                            Ok(entry) => Some(Box::new(entry)),
                            Err(_) => None,
                        },
                        Err(_) => None,
                    },
                    user: None,
                    group: None,
                    unix_pex: None,
                })
            }
        })
    }

    /// ### exec
    ///
    /// Execute a command on localhost
    pub fn exec(&self, cmd: &str) -> Result<String, HostError> {
        // Make command
        let args: Vec<&str> = cmd.split(' ').collect();
        let cmd: &str = args.first().unwrap();
        let argv: &[&str] = &args[1..];
        match std::process::Command::new(cmd).args(argv).output() {
            Ok(output) => match std::str::from_utf8(&output.stdout) {
                Ok(s) => Ok(s.to_string()),
                Err(_) => Ok(String::new()),
            },
            Err(err) => Err(HostError::new(HostErrorType::ExecutionFailed, Some(err))),
        }
    }

    /// ### chmod
    ///
    /// Change file mode to file, according to UNIX permissions
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    pub fn chmod(&self, path: &Path, pex: (u8, u8, u8)) -> Result<(), HostError> {
        let path: PathBuf = self.to_abs_path(path);
        // Get metadta
        match fs::metadata(path.as_path()) {
            Ok(metadata) => {
                let mut mpex = metadata.permissions();
                mpex.set_mode(self.mode_to_u32(pex));
                match set_permissions(path.as_path(), mpex) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(HostError::new(HostErrorType::FileNotAccessible, Some(err))),
                }
            }
            Err(err) => Err(HostError::new(HostErrorType::FileNotAccessible, Some(err))),
        }
    }

    /// ### open_file_read
    ///
    /// Open file for read
    pub fn open_file_read(&self, file: &Path) -> Result<File, HostError> {
        let file: PathBuf = self.to_abs_path(file);
        if !self.file_exists(file.as_path()) {
            return Err(HostError::new(HostErrorType::NoSuchFileOrDirectory, None));
        }
        match OpenOptions::new()
            .create(false)
            .read(true)
            .write(false)
            .open(file.as_path())
        {
            Ok(f) => Ok(f),
            Err(err) => Err(HostError::new(HostErrorType::FileNotAccessible, Some(err))),
        }
    }

    /// ### open_file_write
    ///
    /// Open file for write
    pub fn open_file_write(&self, file: &Path) -> Result<File, HostError> {
        let file: PathBuf = self.to_abs_path(file);
        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file.as_path())
        {
            Ok(f) => Ok(f),
            Err(err) => match self.file_exists(file.as_path()) {
                true => Err(HostError::new(HostErrorType::ReadonlyFile, Some(err))),
                false => Err(HostError::new(HostErrorType::FileNotAccessible, Some(err))),
            },
        }
    }

    /// ### file_exists
    ///
    /// Returns whether provided file path exists
    pub fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    /// ### scan_dir
    ///
    /// Get content of the current directory as a list of fs entry (Windows)
    pub fn scan_dir(&self, dir: &Path) -> Result<Vec<FsEntry>, HostError> {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(err) => return Err(HostError::new(HostErrorType::DirNotAccessible, Some(err))),
        };
        let mut fs_entries: Vec<FsEntry> = Vec::new();
        for entry in entries.flatten() {
            fs_entries.push(match self.stat(entry.path().as_path()) {
                Ok(entry) => entry,
                Err(err) => return Err(err),
            });
        }
        Ok(fs_entries)
    }

    /// ### find
    ///
    /// Find files matching `search` on localhost starting from current directory. Search supports recursive search of course.
    /// The `search` argument supports wilcards ('*', '?')
    pub fn find(&self, search: &str) -> Result<Vec<FsEntry>, HostError> {
        self.iter_search(self.wrkdir.as_path(), &WildMatch::new(search))
    }

    // -- privates

    /// ### iter_search
    ///
    /// Recursive call for `find` method.
    /// Search in current directory for files which match `filter`.
    /// If a directory is found in current directory, `iter_search` will be called using that dir as argument.
    fn iter_search(&self, dir: &Path, filter: &WildMatch) -> Result<Vec<FsEntry>, HostError> {
        // Scan directory
        let mut drained: Vec<FsEntry> = Vec::new();
        match self.scan_dir(dir) {
            Err(err) => Err(err),
            Ok(entries) => {
                // Iter entries
                /* For each entry:
                - if is dir: call iter_search with `dir`
                    - push `iter_search` result to `drained`
                - if is file: check if it matches `filter`
                    - if it matches `filter`: push to to filter
                */
                for entry in entries.iter() {
                    match entry {
                        FsEntry::Directory(dir) => {
                            // If directory matches; push directory to drained
                            if filter.is_match(dir.name.as_str()) {
                                drained.push(FsEntry::Directory(dir.clone()));
                            }
                            match self.iter_search(dir.abs_path.as_path(), filter) {
                                Ok(mut filtered) => drained.append(&mut filtered),
                                Err(err) => return Err(err),
                            }
                        }
                        FsEntry::File(file) => {
                            if filter.is_match(file.name.as_str()) {
                                drained.push(FsEntry::File(file.clone()));
                            }
                        }
                    }
                }
                Ok(drained)
            }
        }
    }

    /// ### u32_to_mode
    ///
    /// Return string with format xxxxxx to tuple of permissions (user, group, others)
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn u32_to_mode(&self, mode: u32) -> (u8, u8, u8) {
        let user: u8 = ((mode >> 6) & 0x7) as u8;
        let group: u8 = ((mode >> 3) & 0x7) as u8;
        let others: u8 = (mode & 0x7) as u8;
        (user, group, others)
    }

    /// mode_to_u32
    ///
    /// Convert owner,group,others to u32
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn mode_to_u32(&self, mode: (u8, u8, u8)) -> u32 {
        ((mode.0 as u32) << 6) + ((mode.1 as u32) << 3) + mode.2 as u32
    }

    /// ### to_abs_path
    ///
    /// Convert path to absolute path
    fn to_abs_path(&self, p: &Path) -> PathBuf {
        // Convert to abs path
        match p.is_relative() {
            true => {
                let mut path: PathBuf = self.wrkdir.clone();
                path.push(p);
                path
            }
            false => PathBuf::from(p),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    use std::os::unix::fs::{symlink, PermissionsExt};

    #[test]
    fn test_host_error_new() {
        let error: HostError = HostError::new(HostErrorType::CouldNotCreateFile, None);
        assert_eq!(error.error, HostErrorType::CouldNotCreateFile);
        assert!(error.ioerr.is_none());
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn test_host_localhost_new() {
        let host: Localhost = Localhost::new(PathBuf::from("/dev")).ok().unwrap();
        assert_eq!(host.wrkdir, PathBuf::from("/dev"));
        // Scan dir
        let entries = std::fs::read_dir(PathBuf::from("/dev").as_path()).unwrap();
        let mut counter: usize = 0;
        for _ in entries {
            counter = counter + 1;
        }
        assert_eq!(host.files.len(), counter);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_host_localhost_new() {
        let host: Localhost = Localhost::new(PathBuf::from("C:\\users")).ok().unwrap();
        assert_eq!(host.wrkdir, PathBuf::from("C:\\users"));
        // Scan dir
        let entries = std::fs::read_dir(PathBuf::from("C:\\users").as_path()).unwrap();
        let mut counter: usize = 0;
        for _ in entries {
            counter = counter + 1;
        }
        assert_eq!(host.files.len(), counter);
    }

    #[test]
    #[should_panic]
    fn test_host_localhost_new_bad() {
        Localhost::new(PathBuf::from("/omargabber/123/345"))
            .ok()
            .unwrap();
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn test_host_localhost_pwd() {
        let host: Localhost = Localhost::new(PathBuf::from("/dev")).ok().unwrap();
        assert_eq!(host.pwd(), PathBuf::from("/dev"));
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn test_host_localhost_list_files() {
        let host: Localhost = Localhost::new(PathBuf::from("/dev")).ok().unwrap();
        // Scan dir
        let entries = std::fs::read_dir(PathBuf::from("/dev").as_path()).unwrap();
        let mut counter: usize = 0;
        for _ in entries {
            counter = counter + 1;
        }
        assert_eq!(host.list_dir().len(), counter);
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn test_host_localhost_change_dir() {
        let mut host: Localhost = Localhost::new(PathBuf::from("/dev")).ok().unwrap();
        let new_dir: PathBuf = PathBuf::from("/dev");
        assert!(host.change_wrkdir(new_dir.as_path()).is_ok());
        // Verify new files
        // Scan dir
        let entries = std::fs::read_dir(PathBuf::from(new_dir).as_path()).unwrap();
        let mut counter: usize = 0;
        for _ in entries {
            counter = counter + 1;
        }
        assert_eq!(host.files.len(), counter);
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    #[should_panic]
    fn test_host_localhost_change_dir_failed() {
        let mut host: Localhost = Localhost::new(PathBuf::from("/dev")).ok().unwrap();
        let new_dir: PathBuf = PathBuf::from("/omar/gabber/123/456");
        assert!(host.change_wrkdir(new_dir.as_path()).is_ok());
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn test_host_localhost_open_read() {
        let host: Localhost = Localhost::new(PathBuf::from("/dev")).ok().unwrap();
        // Create temp file
        let file: tempfile::NamedTempFile = create_sample_file();
        assert!(host.open_file_read(file.path()).is_ok());
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    #[should_panic]
    fn test_host_localhost_open_read_err_no_such_file() {
        let host: Localhost = Localhost::new(PathBuf::from("/dev")).ok().unwrap();
        assert!(host
            .open_file_read(PathBuf::from("/bin/foo-bar-test-omar-123-456-789.txt").as_path())
            .is_ok());
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn test_host_localhost_open_read_err_not_accessible() {
        let host: Localhost = Localhost::new(PathBuf::from("/dev")).ok().unwrap();
        let file: tempfile::NamedTempFile = create_sample_file();
        //let mut perms = fs::metadata(file.path())?.permissions();
        fs::set_permissions(file.path(), PermissionsExt::from_mode(0o222)).unwrap();
        //fs::set_permissions(file.path(), perms)?;
        assert!(host.open_file_read(file.path()).is_err());
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn test_host_localhost_open_write() {
        let host: Localhost = Localhost::new(PathBuf::from("/dev")).ok().unwrap();
        // Create temp file
        let file: tempfile::NamedTempFile = create_sample_file();
        assert!(host.open_file_write(file.path()).is_ok());
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn test_host_localhost_open_write_err() {
        let host: Localhost = Localhost::new(PathBuf::from("/dev")).ok().unwrap();
        let file: tempfile::NamedTempFile = create_sample_file();
        //let mut perms = fs::metadata(file.path())?.permissions();
        fs::set_permissions(file.path(), PermissionsExt::from_mode(0o444)).unwrap();
        //fs::set_permissions(file.path(), perms)?;
        assert!(host.open_file_write(file.path()).is_err());
    }
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    #[test]
    fn test_host_localhost_symlinks() {
        let tmpdir: tempfile::TempDir = tempfile::TempDir::new().unwrap();
        // Create sample file
        assert!(File::create(format!("{}/foo.txt", tmpdir.path().display()).as_str()).is_ok());
        // Create symlink
        assert!(symlink(
            format!("{}/foo.txt", tmpdir.path().display()),
            format!("{}/bar.txt", tmpdir.path().display())
        )
        .is_ok());
        // Get dir
        let host: Localhost = Localhost::new(PathBuf::from(tmpdir.path())).ok().unwrap();
        let files: Vec<FsEntry> = host.list_dir();
        println!("Entries {:?}", files);
        // Verify files
        let file_0: &FsEntry = files.get(0).unwrap();
        match file_0 {
            FsEntry::File(file_0) => {
                if file_0.name == String::from("foo.txt") {
                    assert!(file_0.symlink.is_none());
                } else {
                    assert_eq!(
                        *file_0.symlink.as_ref().unwrap().get_abs_path(),
                        PathBuf::from(format!("{}/foo.txt", tmpdir.path().display()))
                    );
                }
            }
            _ => panic!("expected entry 0 to be file: {:?}", file_0),
        };
        // Verify simlink
        let file_1: &FsEntry = files.get(1).unwrap();
        match file_1 {
            FsEntry::File(file_1) => {
                if file_1.name == String::from("bar.txt") {
                    assert_eq!(
                        *file_1.symlink.as_ref().unwrap().get_abs_path(),
                        PathBuf::from(format!("{}/foo.txt", tmpdir.path().display()))
                    );
                } else {
                    assert!(file_1.symlink.is_none());
                }
            }
            _ => panic!("expected entry 0 to be file: {:?}", file_1),
        };
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn test_host_localhost_mkdir() {
        let tmpdir: tempfile::TempDir = tempfile::TempDir::new().unwrap();
        let mut host: Localhost = Localhost::new(PathBuf::from(tmpdir.path())).ok().unwrap();
        let files: Vec<FsEntry> = host.list_dir();
        assert_eq!(files.len(), 0); // There should be 0 files now
        assert!(host.mkdir(PathBuf::from("test_dir").as_path()).is_ok());
        let files: Vec<FsEntry> = host.list_dir();
        assert_eq!(files.len(), 1); // There should be 1 file now
                                    // Try to re-create directory
        assert!(host.mkdir(PathBuf::from("test_dir").as_path()).is_err());
        // Try abs path
        assert!(host
            .mkdir_ex(PathBuf::from("/tmp/test_dir_123456789").as_path(), true)
            .is_ok());
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn test_host_localhost_remove() {
        let tmpdir: tempfile::TempDir = tempfile::TempDir::new().unwrap();
        // Create sample file
        assert!(File::create(format!("{}/foo.txt", tmpdir.path().display()).as_str()).is_ok());
        let mut host: Localhost = Localhost::new(PathBuf::from(tmpdir.path())).ok().unwrap();
        let files: Vec<FsEntry> = host.list_dir();
        assert_eq!(files.len(), 1); // There should be 1 file now
                                    // Remove file
        assert!(host.remove(files.get(0).unwrap()).is_ok());
        // There should be 0 files now
        let files: Vec<FsEntry> = host.list_dir();
        assert_eq!(files.len(), 0); // There should be 0 files now
                                    // Create directory
        assert!(host.mkdir(PathBuf::from("test_dir").as_path()).is_ok());
        // Delete directory
        let files: Vec<FsEntry> = host.list_dir();
        assert_eq!(files.len(), 1); // There should be 1 file now
        assert!(host.remove(files.get(0).unwrap()).is_ok());
    }

    #[test]
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn test_host_localhost_rename() {
        let tmpdir: tempfile::TempDir = tempfile::TempDir::new().unwrap();
        // Create sample file
        let src_path: PathBuf =
            PathBuf::from(format!("{}/foo.txt", tmpdir.path().display()).as_str());
        assert!(File::create(src_path.as_path()).is_ok());
        let mut host: Localhost = Localhost::new(PathBuf::from(tmpdir.path())).ok().unwrap();
        let files: Vec<FsEntry> = host.list_dir();
        assert_eq!(files.len(), 1); // There should be 1 file now
        assert_eq!(get_filename(files.get(0).unwrap()), String::from("foo.txt"));
        // Rename file
        let dst_path: PathBuf =
            PathBuf::from(format!("{}/bar.txt", tmpdir.path().display()).as_str());
        assert!(host
            .rename(files.get(0).unwrap(), dst_path.as_path())
            .is_ok());
        // There should be still 1 file now, but named bar.txt
        let files: Vec<FsEntry> = host.list_dir();
        assert_eq!(files.len(), 1); // There should be 0 files now
        assert_eq!(get_filename(files.get(0).unwrap()), String::from("bar.txt"));
        // Fail
        let bad_path: PathBuf = PathBuf::from("/asdailsjoidoewojdijow/ashdiuahu");
        assert!(host
            .rename(files.get(0).unwrap(), bad_path.as_path())
            .is_err());
    }

    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    #[test]
    fn test_host_chmod() {
        let tmpdir: tempfile::TempDir = tempfile::TempDir::new().unwrap();
        let file: tempfile::NamedTempFile = create_sample_file();
        let host: Localhost = Localhost::new(PathBuf::from(tmpdir.path())).ok().unwrap();
        // mode_to_u32
        assert_eq!(host.mode_to_u32((6, 4, 4)), 0o644);
        assert_eq!(host.mode_to_u32((7, 7, 5)), 0o775);
        // Chmod to file
        assert!(host.chmod(file.path(), (7, 7, 5)).is_ok());
        // Chmod to dir
        assert!(host.chmod(tmpdir.path(), (7, 5, 0)).is_ok());
        // Error
        assert!(host
            .chmod(Path::new("/tmp/krgiogoiegj/kwrgnoerig"), (7, 7, 7))
            .is_err());
    }

    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    #[test]
    fn test_host_copy_file_absolute() {
        let tmpdir: tempfile::TempDir = tempfile::TempDir::new().unwrap();
        // Create file in tmpdir
        let mut file1_path: PathBuf = PathBuf::from(tmpdir.path());
        file1_path.push("foo.txt");
        // Write file 1
        let mut file1: File = File::create(file1_path.as_path()).ok().unwrap();
        assert!(file1.write_all(b"Hello world!\n").is_ok());
        // Get file 2 path
        let mut file2_path: PathBuf = PathBuf::from(tmpdir.path());
        file2_path.push("bar.txt");
        // Create host
        let mut host: Localhost = Localhost::new(PathBuf::from(tmpdir.path())).ok().unwrap();
        let file1_entry: FsEntry = host.files.get(0).unwrap().clone();
        assert_eq!(file1_entry.get_name(), String::from("foo.txt"));
        // Copy
        assert!(host.copy(&file1_entry, file2_path.as_path()).is_ok());
        // Verify host has two files
        assert_eq!(host.files.len(), 2);
    }

    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    #[test]
    fn test_host_copy_file_relative() {
        let tmpdir: tempfile::TempDir = tempfile::TempDir::new().unwrap();
        // Create file in tmpdir
        let mut file1_path: PathBuf = PathBuf::from(tmpdir.path());
        file1_path.push("foo.txt");
        // Write file 1
        let mut file1: File = File::create(file1_path.as_path()).ok().unwrap();
        assert!(file1.write_all(b"Hello world!\n").is_ok());
        // Get file 2 path
        let file2_path: PathBuf = PathBuf::from("bar.txt");
        // Create host
        let mut host: Localhost = Localhost::new(PathBuf::from(tmpdir.path())).ok().unwrap();
        let file1_entry: FsEntry = host.files.get(0).unwrap().clone();
        assert_eq!(file1_entry.get_name(), String::from("foo.txt"));
        // Copy
        assert!(host.copy(&file1_entry, file2_path.as_path()).is_ok());
        // Verify host has two files
        assert_eq!(host.files.len(), 2);
    }

    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    #[test]
    fn test_host_copy_directory_absolute() {
        let tmpdir: tempfile::TempDir = tempfile::TempDir::new().unwrap();
        // Create directory in tmpdir
        let mut dir_src: PathBuf = PathBuf::from(tmpdir.path());
        dir_src.push("test_dir/");
        assert!(std::fs::create_dir(dir_src.as_path()).is_ok());
        // Create file in src dir
        let mut file1_path: PathBuf = dir_src.clone();
        file1_path.push("foo.txt");
        // Write file 1
        let mut file1: File = File::create(file1_path.as_path()).ok().unwrap();
        assert!(file1.write_all(b"Hello world!\n").is_ok());
        // Copy dir src to dir ddest
        let mut dir_dest: PathBuf = PathBuf::from(tmpdir.path());
        dir_dest.push("test_dest_dir/");
        // Create host
        let mut host: Localhost = Localhost::new(PathBuf::from(tmpdir.path())).ok().unwrap();
        let dir_src_entry: FsEntry = host.files.get(0).unwrap().clone();
        assert_eq!(dir_src_entry.get_name(), String::from("test_dir"));
        // Copy
        assert!(host.copy(&dir_src_entry, dir_dest.as_path()).is_ok());
        // Verify host has two files
        assert_eq!(host.files.len(), 2);
        // Verify dir_dest contains foo.txt
        let mut test_file_path: PathBuf = dir_dest.clone();
        test_file_path.push("foo.txt");
        assert!(host.stat(test_file_path.as_path()).is_ok());
    }

    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    #[test]
    fn test_host_copy_directory_relative() {
        let tmpdir: tempfile::TempDir = tempfile::TempDir::new().unwrap();
        // Create directory in tmpdir
        let mut dir_src: PathBuf = PathBuf::from(tmpdir.path());
        dir_src.push("test_dir/");
        assert!(std::fs::create_dir(dir_src.as_path()).is_ok());
        // Create file in src dir
        let mut file1_path: PathBuf = dir_src.clone();
        file1_path.push("foo.txt");
        // Write file 1
        let mut file1: File = File::create(file1_path.as_path()).ok().unwrap();
        assert!(file1.write_all(b"Hello world!\n").is_ok());
        // Copy dir src to dir ddest
        let dir_dest: PathBuf = PathBuf::from("test_dest_dir/");
        // Create host
        let mut host: Localhost = Localhost::new(PathBuf::from(tmpdir.path())).ok().unwrap();
        let dir_src_entry: FsEntry = host.files.get(0).unwrap().clone();
        assert_eq!(dir_src_entry.get_name(), String::from("test_dir"));
        // Copy
        assert!(host.copy(&dir_src_entry, dir_dest.as_path()).is_ok());
        // Verify host has two files
        assert_eq!(host.files.len(), 2);
        // Verify dir_dest contains foo.txt
        let mut test_file_path: PathBuf = dir_dest.clone();
        test_file_path.push("foo.txt");
        println!("{:?}", host.scan_dir(tmpdir.path()).ok().unwrap());
        assert!(host.stat(test_file_path.as_path()).is_ok());
    }

    #[test]
    fn test_host_exec() {
        let tmpdir: tempfile::TempDir = tempfile::TempDir::new().unwrap();
        let host: Localhost = Localhost::new(PathBuf::from(tmpdir.path())).ok().unwrap();
        // Execute
        #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
        assert_eq!(host.exec("echo 5").ok().unwrap().as_str(), "5\n");
        #[cfg(target_os = "windows")]
        assert_eq!(host.exec("echo 5").ok().unwrap().as_str(), "5\r\n");
    }

    #[test]
    fn test_host_find() {
        let tmpdir: tempfile::TempDir = tempfile::TempDir::new().unwrap();
        let dir_path: &Path = tmpdir.path();
        // Make files
        assert!(make_sample_file(dir_path, "pippo.txt").is_ok());
        assert!(make_sample_file(dir_path, "foo.jpg").is_ok());
        // Make nested struct
        assert!(make_dir(dir_path, "examples").is_ok());
        let mut subdir: PathBuf = PathBuf::from(dir_path);
        subdir.push("examples/");
        assert!(make_sample_file(subdir.as_path(), "omar.txt").is_ok());
        assert!(make_sample_file(subdir.as_path(), "errors.txt").is_ok());
        assert!(make_sample_file(subdir.as_path(), "screenshot.png").is_ok());
        assert!(make_sample_file(subdir.as_path(), "examples.csv").is_ok());
        let host: Localhost = Localhost::new(PathBuf::from(dir_path)).ok().unwrap();
        // Find txt files
        let mut result: Vec<FsEntry> = host.find("*.txt").ok().unwrap();
        result.sort_by_key(|x: &FsEntry| x.get_name().to_lowercase());
        // There should be 3 entries
        assert_eq!(result.len(), 3);
        // Check names (they should be sorted alphabetically already; NOTE: examples/ comes before pippo.txt)
        assert_eq!(result[0].get_name(), "errors.txt");
        assert_eq!(result[1].get_name(), "omar.txt");
        assert_eq!(result[2].get_name(), "pippo.txt");
        // Search for directory
        let mut result: Vec<FsEntry> = host.find("examples*").ok().unwrap();
        result.sort_by_key(|x: &FsEntry| x.get_name().to_lowercase());
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].get_name(), "examples");
        assert_eq!(result[1].get_name(), "examples.csv");
    }

    #[test]
    fn test_host_fmt_error() {
        let err: HostError = HostError::new(
            HostErrorType::CouldNotCreateFile,
            Some(std::io::Error::from(std::io::ErrorKind::AddrInUse)),
        );
        assert_eq!(
            format!("{}", err),
            String::from("Could not create file: address in use")
        );
        assert_eq!(
            format!("{}", HostError::new(HostErrorType::DeleteFailed, None)),
            String::from("Could not delete file")
        );
        assert_eq!(
            format!("{}", HostError::new(HostErrorType::ExecutionFailed, None)),
            String::from("Could not run command")
        );
        assert_eq!(
            format!("{}", HostError::new(HostErrorType::DirNotAccessible, None)),
            String::from("Could not access directory")
        );
        assert_eq!(
            format!(
                "{}",
                HostError::new(HostErrorType::NoSuchFileOrDirectory, None)
            ),
            String::from("No such file or directory")
        );
        assert_eq!(
            format!("{}", HostError::new(HostErrorType::ReadonlyFile, None)),
            String::from("File is readonly")
        );
        assert_eq!(
            format!("{}", HostError::new(HostErrorType::FileNotAccessible, None)),
            String::from("Could not access file")
        );
        assert_eq!(
            format!("{}", HostError::new(HostErrorType::FileAlreadyExists, None)),
            String::from("File already exists")
        );
    }

    /// ### make_sample_file
    ///
    /// Make a file with `name` in the current directory
    fn make_sample_file(dir: &Path, filename: &str) -> std::io::Result<()> {
        let mut p: PathBuf = PathBuf::from(dir);
        p.push(filename);
        let mut file: File = File::create(p.as_path())?;
        write!(
            file,
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nMauris ultricies consequat eros,\nnec scelerisque magna imperdiet metus.\n"
        )?;
        Ok(())
    }

    /// ### make_dir
    ///
    /// Make a directory in `dir`
    fn make_dir(dir: &Path, dirname: &str) -> std::io::Result<()> {
        let mut p: PathBuf = PathBuf::from(dir);
        p.push(dirname);
        std::fs::create_dir(p.as_path())
    }

    /// ### create_sample_file
    ///
    /// Create a sample file
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn create_sample_file() -> tempfile::NamedTempFile {
        // Write
        let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        write!(
            tmpfile,
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nMauris ultricies consequat eros,\nnec scelerisque magna imperdiet metus.\n"
        )
        .unwrap();
        tmpfile
    }

    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn get_filename(entry: &FsEntry) -> String {
        match entry {
            FsEntry::Directory(d) => d.name.clone(),
            FsEntry::File(f) => f.name.clone(),
        }
    }
}
