# mprovision

A command line tool to manage local provisioning profiles. Mostly useful in iOS
development or for mobile CI/CD engineers.

## Usage

Type `mprovision help` in your terminal to see the list of subcommands and options.
Most of subcommands work on `~/Library/MobileDevice/Provisioning Profiles`
directory by default but you can specify a full path using a `--source`
argument.

## Use cases

### 1. See all profiles in your system

`mprovision list` will show the list of all provisioning profiles installed in
your system.

### 2. Search and Remove

- The `list` subcommand accepts an optional argument `-t` or `--text` that
allows you to filter the list of provisioning profiles by some text.
- The `remove` subcommand removes one or more profiles by their uuids or bundle
ids.

### 3. View details of a provisioning profile

The `show` subcommand followed by uuid of a provisioning profile allows you to
see details in xml format. Alternatively, you can use `show-file` subcommand if
you know exact path to a file.

### 4. View profiles that will expire soon

The `list` subcommand accepts an optional argument `-d` or `--expire-in-days`
followed by a number of days and shows the list of profiles that will expire.
For example, the `mprovision list -d 0` command will show profiles that have
already been expired.

### 5. Remove expired profiles

The `clean` subcommand removes expired provisioning profiles.

> NOTE: you can see provisioning profiles that will be removed using the
`mprovision list -d 0` command.

### 6. Number of profiles

There is no special command for that but you can use the following hack:

```bash
mprovision list --oneline | wc -l
```

### 7. Extract provisioning profiles from an ipa file

Use the `extract` subcommand and pass `source` and `destination`.

```bash
mprovision extract MyApp.ipa MyApp/
```

## License

MIT
