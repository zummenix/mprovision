# mprovision
A tool that helps iOS developers to manage local provisioning profiles.
Written in Rust.

## Installation

### Easy way

Download `mprovision.zip` from
[release page](https://github.com/zummenix/mprovision/releases) and unarchive.
This archive contains standalone executable that you can place in your `/usr/bin`

### Compilation from source

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
mprovision show-expired --days <days> [<directory>]
mprovision remove-expired [<directory>]
```
Parameter `<directory>` is optional, all commands work on
`~/Library/MobileDevice/Provisioning Profiles` directory by default.

## Use cases

### Searching and Removing

If you don't want to have a particular provisioning profile in your system,
just search it using `search` subcommand with text, find it in results and then
delete it using `remove` subcommand with uuid of the provisioning profile you
want to delete.

> WARNING: `remove` subcommand is dangerous since it removes profiles from the
system completely.

### View details of a provisioning profile

Use `show-xml` subcommand followed by uuid of a provisioning profile to see details
in xml format.

### View profiles that will expire soon

Use `show-expired` subcommand followed by `--days` parameter to see provisioning
profiles that already expired (`--days 0`) or will expire, for example, in a
next week (`--days 7`).

### Remove already expired profiles

Use `remove-expired` subcommand to remove expired provisioning profiles.

> NOTE: you can see provisioning profiles that will be removed using
`mprovision show-expired --days 0` command.

## Performance

I expect that main bottleneck will be your machine's disk and cpu.
And when you run this tool first time, performance probably will be slow.

For comparison, on my machine with ssd and 4 cpu cores I see the following results:
```
$ time mprovision search any
...
Found 4 of 789
mprovision search any  1.94s user 0.03s system 667% cpu 0.295 total
```
As you can see the execution time for almost 1000 profiles is reasonable.

## License

MIT
