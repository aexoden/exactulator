# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-15

Initial public release of exactulator.

### Added

- Basic calculator GUI built with Iced.
- Exact rational arithmetic using arbitrary-precision rational numbers.
- Addition, subtraction, multiplication, and division operators.
- Decimal point input for fractional values.
- Negate (±) to toggle the sign of the current value, with standalone negations
  recorded in history.
- Answer recall (Ans) to reuse the result of the last computation.
- Clear (C) to reset display, pending operation, and history.
- Clear entry (CE) to clear the current input only.
- Scrollable computation history panel (up to 50 entries).
- Division-by-zero error handling with a clear-to-continue workflow.
- Keyboard shortcuts for all operations, including digit entry, operators,
  backspace, delete, escape, and answer recall.

[0.1.0]: https://github.com/aexoden/exactulator/releases/tag/v0.1.0
