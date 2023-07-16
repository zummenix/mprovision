# mprovision

A tool that helps iOS developers to manage local provisioning profiles.

## Usage

Type `mprovision help` in your terminal to see the list of subcommands and options.
Most of subcommands work on `~/Library/MobileDevice/Provisioning Profiles` directory by default but you can specify a
full path using a `--source` argument.

## Use cases

### See all profiles in your system

It's very simple: `mprovision list`

### Searching and Removing

1. The `list` subcommand accepts an optional argument `-t` or `--text` that allows you to filter the list of 
provisioning profiles by some text. 
2. The `remove` subcommand allows you to remove one or more profiles by their uuids or bundle ids.

### View details of a provisioning profile

The `show` subcommand followed by uuid of a provisioning profile allows you to see details
in xml format.

### View profiles that will expire soon

The `list` subcommand accepts an optional argument `-d` or `--expire-in-days` followed by a number of days and shows the
list of profiles that will expire. For example the `mprovision list -d 0` command will show profiles that have already 
been expired.

### Remove expired profiles

The `clean` subcommand removes expired provisioning profiles.

> NOTE: you can see provisioning profiles that will be removed using the
`mprovision list -d 0` command.

### Number of profiles

Use the `list` subcommand with options you want, set `--oneline` flag and pipe into `wc`,
like that:

```bash
mprovision list -d 30 --oneline | wc -l
```

### Extract provisioning profiles from an ipa file

Use the `extract` subcommand and pass `source` and `destination`.

```bash
mprovision extract MyApp.ipa MyApp/
```

## License

MIT
