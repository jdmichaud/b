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

Then, in `Browsing` mode, use the:
<kbd>&darr;</kbd> or
<kbd>j</kbd> to select down.

<kbd>&uarr;</kbd> or
<kbd>k</kbd> to select down.

<kbd>&rarr;</kbd> or
<kbd>Enter</kbd> or
<kbd>l</kbd> to enter a folder.

<kbd>&larr;</kbd> or
<kbd>h</kbd> to go back to the parent folder.

<kbd>Escape</kbd> or
<kbd>q</kbd> to quit.

<kbd>d</kbd> to switch details on/off.

<kbd>.</kbd> to show/hide hidden files.

<kbd>\`</kbd> to enter roaming mode.

# Modes

In `Browsing` mode, you use your keyboard to move around the directory structure.

In `Roaming` mode, you type folder name which are then filtered. You can still
browse with the arrow and enter keys.
