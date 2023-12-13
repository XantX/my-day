# Logbook application

This application is a small cli for making a daily log.

The application records a description, the number of hours in the day and the url of the assigned task.

# Changelog

### RELEASE # 0.2.0

- Added listing by date with system timezone
- Added show create date with current system timezone

### RELEASE # 0.1.0

- Added create, list and list with date

## CLI commands

*Create a new record*
```
$myday new
```

*List records for a current date*
```
$myday list 
```

*List of specific date records*
```
$myday list YYYY-MM-DD
```

Rust version 1.74.0
