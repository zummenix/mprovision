# mprovision
A tool that helps iOS developers to manage mobileprovision files. (WIP)

## Installation

For now only supported method of installation is compilation from source.
To compile you need to install
[Rust Compiler](https://www.rust-lang.org/downloads.html) and then `clone`
and `build`:

```bash
git clone https://github.com/zummenix/mprovision
cd mprovision
cargo build --release
```

Result binary will be `./target/release/mprovision`.

## Usage

The following commands are supported:
```bash
mprovision search <text> [<directory>]
mprovision remove <uuid> [<directory>]
mprovision show-xml <uuid> [<directory>]
mprovision show-expired [<directory>]
mprovision remove-expired [<directory>]
```
Parameter `<directory>` is optional, all commands work on
`~/Library/MobileDevice/Provisioning Profiles` directory by default.

## Performance

I expect that main bottleneck will be your machine's disk and cpu.
And when you run this tool first time performance probably will be slow.

For comparison, on my machine with ssd and 4 cpu cores I see the following results:
```
$ time mprovision search any
Found 4 of 798 profiles.
...
mprovision search any  1.96s user 0.04s system 713% cpu 0.280 total
```
As you see performance for almost 1000 profiles is reasonable.

## License

MIT
