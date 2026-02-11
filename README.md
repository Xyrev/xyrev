# xyrev

A command-line utility for optimizing various file formats.

## Features

- **pretty-json**: Automatically format JSON documents (minify or prettify)

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/xyrev`.

## Usage

### Pretty JSON

Format JSON files with prettification or minification:

```bash
# Prettify JSON
xyrev pretty-json prettify -i input.json -o output.json

# Minify JSON
xyrev pretty-json minify -i input.json -o output.json

# Sort keys while prettifying
xyrev pretty-json prettify --sorted-keys -i input.json -o output.json
```

### Global Options

- `--silent`: Show nothing
- `--verbose`: Show more debugging information

## Development

Build the project:
```bash
cargo build
```

Run tests:
```bash
cargo test
```

## License

MIT

## Author

Zhiwei Sun (@szwathub)
- E-mail: szwathub@gmail.com
