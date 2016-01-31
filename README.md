# mprovision
A tool that helps iOS developers to manage mobileprovision files. (WIP)

## Installation

For now only supported method is compilation from source. To compile you need to
install [Rust Compiler](https://www.rust-lang.org/downloads.html) and then
`clone` and `build`:

```bash
git clone https://github.com/zummenix/mprovision
cd mprovision
cargo build --release
```

Result binary will be `./target/release/mprovision`.

## Usage

We support two commands for now:
```bash
mprovision search <text> [<directory>]
mprovision remove <uuid> [<directory>]
```
Parameter `<directory>` is optional, all commands work on
`~/Library/MobileDevice/Provisioning Profiles` directory by default.

## License

MIT
