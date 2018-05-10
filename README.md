# b

A minimalistic file browser.

[![demo](https://asciinema.org/a/fvJQ39iaAzqAU0sTgVd2DlsQv.png)](https://asciinema.org/a/fvJQ39iaAzqAU0sTgVd2DlsQv?autoplay=1)

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
git clone https://github.com/jdmichaud/b.git
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
<kbd>&rarr;</kbd> or
<kbd>Enter</kbd> or
<kbd>l</kbd> to enter a folder.

<kbd>&larr;</kbd> to go back to the parent folder.

<kbd>Escape</kbd> or
<kbd>q</kbd> to quit.
