# exactulator

`exactulator` is a simple arbitrary-precision calculator focused on rational
numbers. It provides exact results for basic arithmetic, with no floating-point
precision loss.

Built with [Rust](https://www.rust-lang.org/) and [Iced](https://iced.rs).

## Features

- **Exact rational arithmetic** — Addition, subtraction, multiplication, and
  division using arbitrary-precision rational numbers.
- **Computation history** — A scrollable history panel shows previous
  expressions and their results (up to the last 50 entries).
- **Answer recall** — Reuse the result of the last computation.
- **Negate** — Toggle the sign of the current value. When applied to a
  standalone result (no pending operation), the negation is recorded in the
  history.
- **Division-by-zero protection** — Displays an error rather than crashing.
  Press **C** to clear the error and continue.

## Building

```sh
cargo build --release
```

## Usage

Launch the application:

```sh
cargo run --release
```

### On-Screen Buttons

| Button | Description                                     |
| ------ | ----------------------------------------------- |
| 0–9    | Enter digits                                    |
| .      | Decimal point                                   |
| +      | Add                                             |
| −      | Subtract                                        |
| ×      | Multiply                                        |
| ÷      | Divide                                          |
| =      | Evaluate the pending operation                  |
| ±      | Negate the current value                        |
| Ans    | Recall the last result                          |
| C      | Clear all (display, pending operation, history) |
| CE     | Clear the current entry only                    |

### Keyboard Shortcuts

| Key              | Action                   |
| ---------------- | ------------------------ |
| `0`–`9`          | Enter digits             |
| `.`              | Decimal point            |
| `+`              | Add                      |
| `-`              | Subtract                 |
| `*`              | Multiply                 |
| `/`              | Divide                   |
| `=` or `Enter`   | Evaluate                 |
| `x`              | Recall last result (Ans) |
| `Backspace`      | Delete last digit        |
| `Delete`         | Clear entry (CE)         |
| `Escape`         | Clear all (C)            |
| `F9`             | Negate (±)               |

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE-2.0](LICENSE-APACHE-2.0) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
