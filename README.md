# Voyage

**Stateful subdomain enumeration TUI toolkit**

![Voyage SS](https://github.com/clickswave/voyage/blob/main/voyage-ss1.png?raw=true)

## Installation

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
voyage -d example.com
```

#### Using a custom wordlist:

```
voyage -d example.com -w wordlist.txt
```

#### Adjusting concurrency and request interval:

```
voyage -d example.com -t 10 -i 500
```

#### Saving output to a file:

```
voyage -d example.com -o results.txt
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
