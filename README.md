# TermSCP

[![License: MIT](https://img.shields.io/badge/License-MIT-teal.svg)](https://opensource.org/licenses/MIT) [![Stars](https://img.shields.io/github/stars/veeso/termscp.svg)](https://github.com/veeso/termscp) [![Downloads](https://img.shields.io/crates/d/termscp.svg)](https://crates.io/crates/termscp) [![Crates.io](https://img.shields.io/badge/crates.io-v0.4.0-orange.svg)](https://crates.io/crates/termscp) [![Docs](https://docs.rs/termscp/badge.svg)](https://docs.rs/termscp)  

[![Build](https://github.com/veeso/termscp/workflows/Linux/badge.svg)](https://github.com/veeso/termscp/actions) [![Build](https://github.com/veeso/termscp/workflows/MacOS/badge.svg)](https://github.com/veeso/termscp/actions) [![Build](https://github.com/veeso/termscp/workflows/Windows/badge.svg)](https://github.com/veeso/termscp/actions) [![codecov](https://codecov.io/gh/veeso/termscp/branch/main/graph/badge.svg?token=au67l7nQah)](https://codecov.io/gh/veeso/termscp)

~ Basically, WinSCP on a terminal ~  
Developed by Christian Visintin  
Current version: 0.4.0 (27/03/2021)

---

- [TermSCP](#termscp)
  - [About TermSCP 🖥](#about-termscp-)
    - [Why TermSCP 🤔](#why-termscp-)
  - [Features 🎁](#features-)
  - [Installation 🛠](#installation-)
    - [Cargo 🦀](#cargo-)
    - [Deb package 📦](#deb-package-)
    - [RPM package 📦](#rpm-package-)
    - [AUR Package 🔼](#aur-package-)
    - [Chocolatey 🍫](#chocolatey-)
    - [Brew 🍻](#brew-)
  - [Usage ❓](#usage-)
    - [Address argument 🌎](#address-argument-)
      - [How Password can be provided 🔐](#how-password-can-be-provided-)
  - [Bookmarks ⭐](#bookmarks-)
    - [Are my passwords Safe 😈](#are-my-passwords-safe-)
  - [Text Editor ✏](#text-editor-)
    - [How do I configure the text editor 🦥](#how-do-i-configure-the-text-editor-)
  - [Configuration ⚙️](#configuration-️)
    - [SSH Key Storage 🔐](#ssh-key-storage-)
    - [File Explorer Format](#file-explorer-format)
  - [Keybindings ⌨](#keybindings-)
  - [Documentation 📚](#documentation-)
  - [Known issues 🧻](#known-issues-)
  - [Upcoming Features 🧪](#upcoming-features-)
  - [Contributions 🤝🏻](#contributions-)
  - [Changelog ⏳](#changelog-)
  - [Powered by 🚀](#powered-by-)
  - [Gallery 🎬](#gallery-)
  - [License 📃](#license-)

---

## About TermSCP 🖥

TermSCP is basically a porting of WinSCP to terminal. So basically is a terminal utility with an TUI to connect to a remote server to retrieve and upload files and to interact with the local file system. It works both on **Linux**, **MacOS**, **BSD** and **Windows** and supports SFTP, SCP, FTP and FTPS.

![Explorer](assets/images/explorer.gif)

---

### Why TermSCP 🤔

It happens quite often to me, when using SCP at work to forget the path of a file on a remote machine, which forces me to connect through SSH, gather the file path and finally download it through SCP. I could use WinSCP, but I use Linux and I pratically use the terminal for everything, so I wanted something like WinSCP on my terminal. Yeah, I know there is midnight commander too, but actually I don't like it very much tbh (and hasn't a decent support for scp).

## Features 🎁

- Different communication protocols support
  - SFTP
  - SCP
  - FTP and FTPS
- Compatible with Windows, Linux, BSD and MacOS
- Handy user interface to explore and operate on the remote and on the local machine file system
  - Create, remove, rename, search, view and edit files
- Bookmarks and recent connections can be saved to access quickly to your favourite hosts
- Supports text editors to view and edit text files
- Supports both SFTP/SCP authentication through SSH keys and username/password
- Customizations:
  - Custom file explorer format
  - Customizable text editor
  - Customizable file sorting
- SSH key storage
- Written in Rust
- Easy to extend with new file transfers protocols
- Developed keeping an eye on performance

---

## Installation 🛠

If you're considering to install TermSCP I want to thank you 💛 ! I hope you will enjoy TermSCP!  
If you want to contribute to this project, don't forget to check out our contribute guide. [Read More](CONTRIBUTING.md)

### Cargo 🦀

```sh
# Install termscp through cargo
cargo install termscp
```

Requirements:

- Linux
  - pkg-config
  - libssh2
  - openssl

### Deb package 📦

Get `deb` package from [HERE](https://github.com/veeso/termscp/releases/latest/download/termscp_0.4.0_amd64.deb)
or run `wget https://github.com/veeso/termscp/releases/latest/download/termscp_0.4.0_amd64.deb`

then install through dpkg:

```sh
dpkg -i termscp_*.deb
# Or even better with gdebi
gdebi termscp_*.deb
```

### RPM package 📦

Get `rpm` package from [HERE](https://github.com/veeso/termscp/releases/latest/download/termscp-0.4.0-1.x86_64.rpm)
or run `wget https://github.com/veeso/termscp/releases/latest/download/termscp-0.4.0-1.x86_64.rpm`

then install through rpm:

```sh
rpm -U termscp_*.rpm
```

### AUR Package 🔼

On Arch Linux based distribution, you can install termscp using for istance [yay](https://github.com/Jguer/yay), which I recommend to install AUR packages.

```sh
yay -S termscp
```

### Chocolatey 🍫

You can install TermSCP on Windows using [chocolatey](https://chocolatey.org/)

Start PowerShell as administrator and run

```ps
choco install termscp
```

Alternatively you can download the ZIP file from [HERE](https://github.com/veeso/termscp/releases/latest/download/termscp.0.4.0.nupkg)

and then with PowerShell started with administrator previleges, run:

```ps
choco install termscp -s .
```

### Brew 🍻

You can install TermSCP on MacOS using [brew](https://brew.sh/)

From your terminal run

```sh
brew tap veeso/termscp
brew install termscp
```

---

## Usage ❓

TermSCP can be started with the following options:

`termscp [options]... [protocol://user@address:port:wrkdir] [local-wrkdir]`

- `-P, --password <password>` if address is provided, password will be this argument
- `-v, --version` Print version info
- `-h, --help` Print help page

TermSCP can be started in two different mode, if no extra arguments is provided, TermSCP will show the authentication form, where the user will be able to provide the parameters required to connect to the remote peer.

Alternatively, the user can provide an address as argument to skip the authentication form and starting directly the connection to the remote server.

If address argument is provided you can also provide the start working directory for local host

### Address argument 🌎

The address argument has the following syntax:

```txt
[protocol://][username@]<address>[:port][:wrkdir]
```

Let's see some example of this particular syntax, since it's very comfortable and you'll probably going to use this instead of the other one...

- Connect using default protocol (*defined in configuration*) to 192.168.1.31, port if not provided is default for the selected protocol (in this case depends on your configuration); username is current user's name

    ```sh
    termscp 192.168.1.31
    ```

- Connect using default protocol (*defined in configuration*) to 192.168.1.31; username is `root`

    ```sh
    termscp root@192.168.1.31
    ```

- Connect using scp to 192.168.1.31, port is 4022; username is `omar`

    ```sh
    termscp scp://omar@192.168.1.31:4022
    ```

- Connect using scp to 192.168.1.31, port is 4022; username is `omar`. You will start in directory `/tmp`

    ```sh
    termscp scp://omar@192.168.1.31:4022:/tmp
    ```

#### How Password can be provided 🔐

You have probably noticed, that, when providing the address as argument, there's no way to provide the password.
Password can be basically provided through 3 ways when address argument is provided:

- `-P, --password` option: just use this CLI option providing the password. I strongly unrecommend this method, since it's very unsecure (since you might keep the password in the shell history)
- Via `sshpass`: you can provide password via `sshpass`, e.g. `sshpass -f ~/.ssh/topsecret.key termscp cvisintin@192.168.1.31`
- You will be prompted for it: if you don't use any of the previous methods, you will be prompted for the password, as happens with the more classics tools such as `scp`, `ssh`, etc.

---

## Bookmarks ⭐

In TermSCP it is possible to save favourites hosts, which can be then loaded quickly from the main layout of termscp.
TermSCP will also save the last 16 hosts you connected to.
This feature allows you to load all the parameters required to connect to a certain remote, simply selecting the bookmark in the tab under the authentication form.

Bookmarks will be saved, if possible at:

- `$HOME/.config/termscp/` on Linux/BSD
- `$HOME/Library/Application Support/termscp` on MacOs
- `FOLDERID_RoamingAppData\termscp\` on Windows

For bookmarks only (this won't apply to recent hosts) it is also possible to save the password used to authenticate. The password is not saved by default and must be specified through the prompt when saving a new Bookmark.

> I was very undecided about storing passwords in termscp. The reason? Saving a password on your computer might give access to a hacker to any server you've registered. But I must admit by myself that for many machines typing the password everytime is really boring, also many times I have to work with machines in LAN, which wouldn't provide any advantage to an attacker, So I came out with a good compromise for passwords.

I warmly suggest you to follow these guidelines in order to decide whether you should or you shouldn't save passwords:

- **DON'T** save passwords for machines which are exposed on the internet, save passwords only for machines in LAN
- Make sure your machine is protected by attackers. If possible encrypt your disk and don't leave your PC unlocked while you're away.
- Preferably, save passwords only when a compromising of the target machine wouldn't be a problem.

To create a bookmark, just fulfill the authentication form and then input `<CTRL+S>`; you'll then be asked to give a name to your bookmark, and tadah, the bookmark has been created.
If you go to [gallery](#gallery-), there is a GIF showing how bookmarks work 💪.

### Are my passwords Safe 😈

Well, kinda.
As said before, bookmarks are saved in your configuration directory along with passwords. Passwords are obviously not plain text, they are encrypted with **AES-128**. Does this make them safe? Well, depends on your operating system:

On Windows and MacOS the passwords are stored, if possible (but should be), in respectively the Windows Vault and the Keychain. This is actually super-safe and is directly managed by your operating system.

On Linux and BSD, on the other hand, the key used to encrypt your passwords is stored on your drive (at $HOME/.config/termscp). It is then, still possible to retrieve the key to decrypt passwords. Luckily, the location of the key guarantees your key can't be read by users different from yours, but yeah, I still wouldn't save the password for a server exposed on the internet 😉.
Actually [keyring-rs](https://github.com/hwchen/keyring-rs), supports Linux, but for different reasons I preferred not to make it available for this configuration. If you want to read more about my decision read [this issue](https://github.com/veeso/termscp/issues/2), while if you think this might have been implemented differently feel free to open an issue with your proposal.

---

## Text Editor ✏

TermSCP has, as you might have noticed, many features, one of these is the possibility to view and edit text file. It doesn't matter if the file is located on the local host or on the remote host, termscp provides the possibility to open a file in your favourite text editor.
In case the file is located on remote host, the file will be first downloaded into your temporary file directory and then, **only** if changes were made to the file, re-uploaded to the remote host. TermSCP checks if you made changes to the file verifying the last modification time of the file.

Just a reminder: **you can edit only textual file**; binary files are not supported.

### How do I configure the text editor 🦥

Text editor is automatically found using this [awesome crate](https://github.com/milkey-mouse/edit), if you want to change the text editor to use, change it in termscp configuration. [View more](#configuration-️)

---

## Configuration ⚙️

TermSCP supports some user defined parameters, which can be defined in the configuration.
Underhood termscp has a TOML file and some other directories where all the parameters will be saved, but don't worry, you won't touch any of these files, since I made possible to configure termscp from its user interface entirely.

termscp, like for bookmarks, just requires to have these paths accessible:

- `$HOME/.config/termscp/` on Linux/BSD
- `$HOME/Library/Application Support/termscp` on MacOs
- `FOLDERID_RoamingAppData\termscp\` on Windows

To access configuration, you just have to press `<CTRL+C>` from the home of termscp.

These parameters can be changed:

- **Default Protocol**: the default protocol is the default value for the file transfer protocol to be used in termscp. This applies for the login page and for the address CLI argument.
- **Text Editor**: the text editor to use. By default termscp will find the default editor for you; with this option you can force an editor to be used (e.g. `vim`). **Also GUI editors are supported**, unless they `nohup` from the parent process so if you ask: yes, you can use `notepad.exe`, and no: **Visual Studio Code doesn't work**.
- **Show Hidden Files**: select whether hidden files shall be displayed by default. You will be able to decide whether to show or not hidden files at runtime pressing `A` anyway.
- **Check for updates**: if set to `yes`, termscp will fetch the Github API to check if there is a new version of termscp available.
- **Group Dirs**: select whether directories should be groupped or not in file explorers. If `Display first` is selected, directories will be sorted using the configured method but displayed before files, viceversa if `Display last` is selected.

### SSH Key Storage 🔐

Along with configuration, termscp provides also an **essential** feature for **SFTP/SCP clients**: the SSH key storage.

You can access the SSH key storage, from configuration moving to the `SSH Keys` tab, once there you can:

- **Add a new key**: just press `<CTRL+N>` and you will be prompted to create a new key. Provide the hostname/ip address and the username associated to the key and finally a text editor will open up: paste the **PRIVATE** ssh key into the text editor, save and quit.
- **Remove an existing key**: just press `<DEL>` or `<CTRL+E>` on the key you want to remove, to delete persistently the key from termscp.
- **Edit an existing key**: just press `<ENTER>` on the key you want to edit, to change the private key.

> Q: Wait, my private key is protected with password, can I use it?  
> A: Of course you can. The password provided for authentication in termscp, is valid both for username/password authentication and for RSA key authentication.

### File Explorer Format

It is possible through configuration to define a custom format for the file explorer. This field, with name `File formatter syntax` will define how the file entries will be displayed in the file explorer.
The syntax for the formatter is the following `{KEY1}... {KEY2:LENGTH}... {KEY3:LENGTH:EXTRA} {KEYn}...`.
Each key in bracket will be replaced with the related attribute, while everything outside brackets will be left unchanged.

- The key name is mandatory and must be one of the keys below
- The length describes the length reserved to display the field. Static attributes doesn't support this (GROUP, PEX, SIZE, USER)
- Extra is supported only by some parameters and is an additional options. See keys to check if extra is supported.

These are the keys supported by the formatter:

- `ATIME`: Last access time (with default syntax `%b %d %Y %H:%M`); Extra might be provided as the time syntax (e.g. `{ATIME:8:%H:%M}`)
- `CTIME`: Creation time (with syntax `%b %d %Y %H:%M`); Extra might be provided as the time syntax (e.g. `{CTIME:8:%H:%M}`)
- `GROUP`: Owner group
- `MTIME`: Last change time (with syntax `%b %d %Y %H:%M`); Extra might be provided as the time syntax (e.g. `{MTIME:8:%H:%M}`)
- `NAME`: File name (Elided if longer than 24)
- `PEX`: File permissions (UNIX format)
- `SIZE`: File size (omitted for directories)
- `SYMLINK`: Symlink (if any `-> {FILE_PATH}`)
- `USER`: Owner user

If left empty, the default formatter syntax will be used: `{NAME:24} {PEX} {USER} {SIZE} {MTIME:17:%b %d %Y %H:%M}`

---

## Keybindings ⌨

| Key           | Command                                               | Reminder    |
|---------------|-------------------------------------------------------|-------------|
| `<ESC>`       | Disconnect from remote; return to authentication page |             |
| `<TAB>`       | Switch between log tab and explorer                   |             |
| `<BACKSPACE>` | Go to previous directory in stack                     |             |
| `<RIGHT>`     | Move to remote explorer tab                           |             |
| `<LEFT>`      | Move to local explorer tab                            |             |
| `<UP>`        | Move up in selected list                              |             |
| `<DOWN>`      | Move down in selected list                            |             |
| `<PGUP>`      | Move up in selected list by 8 rows                    |             |
| `<PGDOWN>`    | Move down in selected list by 8 rows                  |             |
| `<ENTER>`     | Enter directory                                       |             |
| `<SPACE>`     | Upload / download selected file                       |             |
| `<A>`         | Toggle hidden files                                   | All         |
| `<B>`         | Sort files by                                         | Bubblesort? |
| `<C>`         | Copy file/directory                                   | Copy        |
| `<D>`         | Make directory                                        | Directory   |
| `<E>`         | Delete file (Same as `DEL`)                           | Erase       |
| `<F>`         | Search for files (wild match is supported)            | Find        |
| `<G>`         | Go to supplied path                                   | Go to       |
| `<H>`         | Show help                                             | Help        |
| `<I>`         | Show info about selected file or directory            | Info        |
| `<L>`         | Reload current directory's content                    | List        |
| `<N>`         | Create new file with provided name                    | New         |
| `<O>`         | Edit file; see [Text editor](#text-editor-)           | Open        |
| `<Q>`         | Quit TermSCP                                          | Quit        |
| `<R>`         | Rename file                                           | Rename      |
| `<S>`         | Save file as...                                       | Save        |
| `<U>`         | Go to parent directory                                | Upper       |
| `<X>`         | Execute a command                                     | eXecute     |
| `<DEL>`       | Delete file                                           |             |
| `<CTRL+C>`    | Abort file transfer process                           |             |

---

## Documentation 📚

The developer documentation can be found on Rust Docs at <https://docs.rs/termscp>

---

## Known issues 🧻

- `NoSuchFileOrDirectory` on connect (WSL): I know about this issue and it's a glitch of WSL I guess. Don't worry about it, just move the termscp executable into another PATH location, such as `/usr/bin`, or install it through the appropriate package format (e.g. deb).

---

## Upcoming Features 🧪

- **Themes provider**: I'm still thinking about how I will implement this, but basically the idea is to have a configuration file where it will be possible
    to define the color schema for the entire application. I haven't planned this release yet

No other new feature is planned at the moment. I actually think that termscp is getting mature and now I should focus upcoming updates more on bug fixing and
code/performance improvements than on new features.
Anyway there are some ideas which I'd like to implement. If you want to start working on them, feel free to open a PR:

- Amazon S3 support
- Samba support
- Themes provider

---

## Contributions 🤝🏻

Contributions are welcome! 😉

If you think you can contribute to TermSCP, please follow [TermSCP's contributions guide](CONTRIBUTING.md)

## Changelog ⏳

View TermSCP's changelog [HERE](CHANGELOG.md)

---

## Powered by 🚀

TermSCP is powered by these aweseome projects:

- [bytesize](https://github.com/hyunsik/bytesize)
- [crossterm](https://github.com/crossterm-rs/crossterm)
- [edit](https://github.com/milkey-mouse/edit)
- [keyring-rs](https://github.com/hwchen/keyring-rs)
- [rpassword](https://github.com/conradkleinespel/rpassword)
- [rust-ftp](https://github.com/mattnenterprise/rust-ftp)
- [ssh2-rs](https://github.com/alexcrichton/ssh2-rs)
- [textwrap](https://github.com/mgeisler/textwrap)
- [tui-rs](https://github.com/fdehau/tui-rs)
- [whoami](https://github.com/libcala/whoami)

---

## Gallery 🎬

> Termscp Home

![Auth](assets/images/auth.gif)

> Bookmarks

![Bookmarks](assets/images/bookmarks.gif)

> Setup

![Setup](assets/images/config.gif)

> Text editor

![TextEditor](assets/images/text-editor.gif)

---

## License 📃

termscp is licensed under the MIT license since version 0.4.0.

You can read the entire license [HERE](LICENSE)

*Older version (0.1.0 ~ 0.3.3) are licensed under the [GPL-V3](http://www.gnu.org/licenses/gpl-3.0.txt) license*
