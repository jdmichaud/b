# b

A minimalistic file browser.

# Installation

## Dependencies

You need rust (at least v1.6.0) installed.
You also need the ncurses dev library (at least v5).
For ubuntu, to install the ncurses library:
```bash
apt install libncurses5-dev
```

## Installation through git

```bash
git clone
cd b
cargo build --release
```

The executable will be generated in `target/release/b`

Copy the executable somewhere in your path and gun it!

# Usage

```bash
b
```
or
```bash
b /some/path
```

Then use the:
<kbd>&rightarrow</kbd> or
<kbd>Enter</kbd> or
<kbd>l</kbd> to enter a folder

<kbd>&leftarrow</kbd> to go back to the parent folder

<kbd>Escape</kbd> or
<kbd>q</kbd> to quit
