![Voyage](https://github.com/clickswave/voyage/blob/main/readme-cover.png?raw=true)

**Voyage is a subdomain enumeration tool built in Rust that combines active and passive discovery methods. It keeps track of progress using SQLite, so you can stop and resume scans without repeating work. The tool features a terminal user interface (TUI) for real-time monitoring and is designed to be fast and efficient, leveraging multi-threading to handle large-scale reconnaissance.**



## Screenshots
![Voyage SS1](https://github.com/clickswave/voyage/blob/main/voyage-ss1.png?raw=true)
![Voyage SS2](https://github.com/clickswave/voyage/blob/main/voyage-ss2.png?raw=true)

## Installation

### Linux and MacOS
**If you are feeling brave**
```bash
curl https://raw.githubusercontent.com/clickswave/voyage/refs/heads/main/install.sh | bash
```
**Recommended method**
```bash
curl https://raw.githubusercontent.com/clickswave/voyage/refs/heads/main/install.sh -o voyage-install.sh
# read the script to see what it does
bash voyage-install.sh
```

### Windows
**Recommended method**
```powershell
# inside powershell terminal
git clone https://github.com/clickswave/voyage
cd voyage
.\install.ps1
```

### From Source

```
git clone https://github.com/clickswave/voyage.git
cd voyage
cargo build --release
```

### Using Prebuilt Binaries

Download the latest release from the Releases page and extract it.

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
