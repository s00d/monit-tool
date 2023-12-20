[![Crates.io](https://img.shields.io/crates/v/monit-tool.svg)](https://crates.io/crates/monit-tool)
[![GitHub issues](https://img.shields.io/github/issues/s00d/monit-tool.svg)](https://github.com/s00d/monit-tool/issues)
[![GitHub license](https://img.shields.io/github/license/s00d/monit-tool.svg)](https://github.com/s00d/monit-tool/blob/main/LICENSE)

# Process Monitor in Rust

![img](https://github.com/s00d/monit-tool/blob/main/assets/main.gif?raw=true)

This Rust application provides a simple interface to monitor CPU and memory usage of processes running on your system. It allows users to filter and select a process from the list and displays real-time CPU and memory usage using colored graphs.

## Features

- **Process Filtering**: Filter processes by name.
- **Interactive Selection**: Choose a process to monitor from a filtered list.
- **Real-time Monitoring**: View real-time CPU and memory usage of the selected process.
- **Graphical Display**: CPU and memory usage are displayed as colored lines on a graph.

## Requirements

- Rust programming environment.
- Dependencies: `sysinfo`, `dialoguer`, `rgb`, `textplots`.

## Installation

1. Clone the repository:
   ```
   git clone https://github.com/s00d/monit-tool
   ```
2. Navigate to the directory:
   ```
   cd ./monit-tool
   ```
3. Build the project:
   ```
   cargo build --release
   ```

## crates.io

Before installing the `monit-tool` package, you need to install Rust. Rust is a programming language that the package is built with. Here are the steps to install Rust:

1. Open a terminal or command prompt.

2. Visit the official Rust website at [https://www.rust-lang.org/](https://www.rust-lang.org/).

3. Follow the instructions on the website to download and install Rust for your operating system.

4. After the installation is complete, verify that Rust is installed correctly by running the following command in your terminal:

```shell
rustc --version
```

You can install the `monit-tool` package using the `cargo` utility. Make sure you have Rust compiler and `cargo` tool installed.

1. Open a terminal or command prompt.

2. Run the following command to install the package:

```shell
cargo install monit-tool
```

## Usage

1. Run the program:
   ```
   monit-tool [parameter]
   ```
2. Enter a filter to search for a specific process or leave it blank to list all processes.
3. Select a process from the list to monitor.
4. The program will display real-time CPU and memory usage on a graphical chart.

Where [parameter] is an optional argument you can pass when starting the application. If no parameter is specified, the application will display information about all processes. If a parameter is provided, the application will only display information about processes whose names contain the specified parameter.

## Customization

- Modify the graph dimensions or the refresh rate in the main loop for different display preferences.

## Contributing

Contributions, issues, and feature requests are welcome. Feel free to check [issues page](link-to-issues-page) if you want to contribute.

## License

Distributed under the MIT License. See `LICENSE` for more information.

## Acknowledgements

- [sysinfo](https://crates.io/crates/sysinfo)
- [dialoguer](https://crates.io/crates/dialoguer)
- [rgb](https://crates.io/crates/rgb)
- [textplots](https://crates.io/crates/textplots)
