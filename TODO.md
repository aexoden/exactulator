# Exactulator TODO

## Immediate Tasks

- Update README

## Features

This is a potential list of features. There is no guarantee that any of them will
be implemented, and some of them may in fact be undesirable or infeasible.

### High Value, Low Complexity

- **Backspace** — Keyboard is available, but a button would also be useful.
- **Result display formatting** — Truncate long decimals, show a rounding indicator
  (e.g. `≈`), and use the Unicode minus sign. Critical for usability with rational
  results like `1/3`. Probably needs a `HistoryEntry` refactor as well.
- **Reciprocal operator** — Add a `1/x` button.
- **Copy result to clipboard** — Click-to-copy or Ctrl+C for the displayed result.

### High Value, Medium Complexity

- **Repeat operation** - Repeatedly pressing enter/equals should repeat the last
  operation.
- **Parentheses / expression chain** — Currently only single pending operations
  are supported (`2 + 3 × 4` evaluates left-to-right). Proper operator precedence
  would be a big improvement.
- **Fraction display mode** — Toggle between decimal and fraction (`p/q`) display.
- **Percentage operator** — Common calculator feature (`%`), useful for everyday
  math.
- **Click history entry to reuse** — Clicking a history expression or result could
  load it as the current value. The `HistoryEntry` struct is already there.
- **Persistent history** — Save/load history across sessions (e.g. to a file).

### Medium Value, Low Complexity

- **Responsive/resizable layout** — `KEYPAD_HEIGHT` is a fixed constant. Making
  the layout adapt to window resizing would improve the feel.
- **Configurable theme** — Currently hardcoded to `Theme::TokyoNightStorm`. Could
  offer light/dark or system-preference detection.
- **Tooltip / hover feedback** — Minor polish for discoverability.
- **Error recovery** — All input is blocked after an error until Clear is pressed.
  Allowing operators after error (to start fresh) would reduce friction.

### High Value, High Complexity

- **Expression/formula input mode** — A text input for full expressions (e.g.
  `(3/7 + 1/2) * 14`) with parsing and evaluation. Would unlock the full power
  of arbitrary-precision rational math.
- **Scientific functions** — Square root, powers, modulo, factorial. Some (like
  integer power) stay exact; others (like sqrt of non-perfect-squares) need a
  policy decision on representation.
- **Unit conversions** — Exact rational conversions between units (e.g. inches
  to cm).
- **Variable/memory slots** — Named variables or M+/M-/MR memory registers.
- **Undo/redo** — State history with Ctrl+Z / Ctrl+Y support.

### Lower Priority

- **Thousands separators** — Grouping digits for readability in large numbers.
- **Configurable precision cap** — Let users pick max display digits for
  repeating decimals.
- **Export history** — Copy or save history as text/CSV.
- **Accessibility** — Screen reader support, high-contrast mode, keyboard focus
  indicators.
