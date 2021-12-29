# Changelog

# [Unreleased]

# [0.2.0] - 2021-12-29

This release makes breaking changes to the API!

* Added `syslog` crate and `flexi-syslog` now publically depends on it.
* Added `syslog` implementation of `flexi-logger::LogWriter`.
* Removed libc implementation of `flexi-logger::LogWriter`.

# [0.1.0] - 2021-12-19

* Added `flexi-logger::LogWriter` targetting syslog through libc. 
