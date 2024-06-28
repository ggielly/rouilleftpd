# rouilleFTPd
 **rouilleftpd** (WIP) is a highly configurable and robust FTP server written in Rust.

## Features

- Configuration via `rouilleftpd.conf` file (TOML syntax)
- Shared memory for inter-process communication (IPC) : rouillespy will help you to monitore the users in console.
- Asynchronous I/O operations using `tokio`
- Command-line argument handling
- Chrooted by default
- site [args] commands for managing the ftpd : site adduser, site deluser, site user...


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


## Contributing

Contributions are welcome! Please submit a pull request or open an issue to discuss what you would like to change.

## Acknowledgments

Special thanks to the Rust community and the authors of the crates used in this project.

