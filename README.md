![Voyage](https://github.com/clickswave/voyage/blob/main/static/readme-cover.png?raw=true)

> **voyage is now developed in the [Crossfyre toolchain](https://github.com/clickswave/crossfyre-toolchain).** This repository keeps the history of the standalone tool and its last release, v0.0.1. New versions, fixes and issues belong in the toolchain repository.


**Voyage is a subdomain enumeration tool built in Rust that combines active and passive discovery methods. It keeps track of progress using SQLite, so you can stop and resume scans without repeating work. The tool features a terminal user interface (TUI) for real-time monitoring and is designed to be fast and efficient, leveraging multi-threading to handle large-scale reconnaissance.**

<p align="center">
<a>
<img src="https://img.shields.io/badge/Language-Rust-%23B7410E.svg?style=for-the-badge&logo=rust&logoColor=white">
</a>
<a>
<img src="https://img.shields.io/badge/OS-WINDOWS%20%7C%20MAC%20%7C%20LINUX-2ECC71?style=for-the-badge&logo=databricks&logoColor=white&labelColor=555555">
</a>
<a href="https://discord.gg/QMuyWSY5sa?utm_source=github&utm_medium=web">
<img src="https://img.shields.io/badge/COMMUNITY-DISCORD-%235865F2.svg?style=for-the-badge&logo=discord&logoColor=white">
</a>
</p>
<hr>

## Key Features

- **Stateful Enumeration Engine**  
  Voyage keeps track of progress, enabling seamless resumable scans without redundant checks.

- **Hybrid Subdomain Enumeration**  
  Utilizes both passive intelligence gathering and active brute-force techniques for comprehensive coverage.

- **Configurable Performance**  
  Adjust threads, request intervals and other parameters mid-scan to balance speed against the load on the target.

- **Selective Enumeration**  
  Disable active or passive enumeration modes, or exclude specific sources and techniques.

- **Per-User Local Database**  
  Scan data is stored per user, maintaining isolation and personalized history.

- **Fine-Grained Customizations**  
  Wide variety of customizations for your scan. Explore with `voyage --help`.


## Screenshots
![Screenshot 1](https://github.com/clickswave/voyage/blob/main/static/voyage-ss1.png?raw=true)
![Screenshot 2](https://github.com/clickswave/voyage/blob/main/static/voyage-ss2.png?raw=true)

## Installation

Install the current voyage from the toolchain:

```bash
cargo install --git https://github.com/clickswave/crossfyre-toolchain.git voyage
```

or install the whole toolchain, voyage included, with the installer in the [toolchain README](https://github.com/clickswave/crossfyre-toolchain#install).

### This repository's last version

To build v0.0.1 from this repository:

```
git clone https://github.com/clickswave/voyage.git
cd voyage
cargo build --release
```

Prebuilt v0.0.1 binaries are on the Releases page.

## Usage

```
voyage [OPTIONS] --domain <DOMAIN>
```

### Example Commands

#### Basic enumeration:
```
voyage -d example.com -w ./path/to/wordlist.txt
```

#### Chain multiple domains:
```
voyage -d example.com -d example2.com -w ./path/to/wordlist.txt
```

#### Adjusting concurrency and request interval in milliseconds:
```
voyage -d example.com -w ./path/to/wordlist.txt -t 10 -i 500 
```

#### Saving output to a file:
```
voyage -d example.com -w ./path/to/wordlist.txt -o results.txt
```

#### Launch a fresh scan (deletes cache for current scan):
```
voyage -d example.com -w ./path/to/wordlist.txt --fresh-start
```

#### Full list of options:
```
voyage --help
```

## Output Formats

Voyage supports exporting results in different formats:
* **Text:** Default format
* **CSV:** Machine-readable format

### Example:

```
voyage -d example.com --output-format csv -o results.csv
```

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.

## License

Voyage is licensed under the **GNU General Public License v3.0 (GPLv3)**. See LICENSE for details on your rights and obligations under this license.

## Links

**Website:** voyage.vg

