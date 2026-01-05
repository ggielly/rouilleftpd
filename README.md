# rouilleFTPd

 **rouilleftpd** (WIP) is a PoC for a configurable FTP server, written in Rust.

## Features

- Configuration via `rouilleftpd.conf` file (TOML syntax)
- Shared memory for inter-process communication (IPC) : rouillespy will help you to monitore the users in console.
- Asynchronous I/O operations using `tokio`
- Command-line argument handling
- Chrooted by default
- site [args] commands for managing the ftpd : site adduser
- IPv4/IPv6 support
- TLS support

## Requirements

- Rust (https://www.rust-lang.org/tools/install)

## Installation

1. Clone the repository :

    ```sh
    git clone https://github.com/ggielly/rouilleftpd.git
    cd rouilleftpd
    ```

2. Build the project :

    ```sh
    cargo build --release
    ```

## License

This project is licensed under the GPLv3 License. See the LICENSE file for details.

