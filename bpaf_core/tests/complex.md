# complex

## Synopsis

* [`complex`](#complex) -- Complex program, long words
* [`complex subcommand`](#complex-subcommand) -- A subcommand with a short alias

## `complex`

`complex` -- Complex program, long words

### Usage

**`complex`** _`COMMAND`_ ... _`COMMAND`_ ... **`-m`** `{[`**`-v`**`]}` `[`**`-x`**`]` **`-a`** **`-b`** `[`**`--timeout`**=_`SEC`_`]` **`-p`**=_`PAT`_ `[`**`-i`**`]` _`EXTRA`_ **`--config`**=_`FILE`_ _`COMMAND`_ ... `[`**`-M`**`]`

### Description

Long words and multi paragraphs

This parser exercises many bpaf features including custom sections, nested commands, and environment variables.

### Available positional items:

* _`EXTRA`_

### Custom Section

* -x\
  Custom flag rendered via literal

### Search options:

* **`-p`**=_`PAT`_\
  Search pattern

* **`-i`**\
  Case insensitive

### Available options:

* **`-m`**, **`--mode`** [**`-v`**]\
  Mode options

* **`-v`**\
  Verbose mode

* **`-a`**\
  Flag A

* **`-b`**\
  Flag B

* **`--timeout`**=_`SEC`_\
  Timeout in seconds\
  [default: 30]

* **`--config`**=_`FILE`_\
  Path to config file\
  Uses environment variable **`CONFIG_PATH`**

* **`-M`**\
  line1\
  line2

  line4

* **`-h`**, **`--help`**\
  Prints help information

### Available commands:

* **`action`**\
  Perform an action

* **`set`** _`KEY`_ _`VAL`_\
  Set a key=value pair

* _`KEY`_\
  Name of an option to set

* _`VAL`_\
  Value to set

* **`subcommand`**, **`s`**\
  A subcommand with a short alias

Beware of edge cases!

## `complex subcommand`

`complex subcommand` -- A subcommand with a short alias

### Usage

**`complex`** **`subcommand`**

### Available options:

* **`-h`**, **`--help`**\
  Prints help information
