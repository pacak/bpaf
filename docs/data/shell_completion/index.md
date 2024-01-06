# Dynamic shell completion

`bpaf` implements shell completion to allow to automatically fill in not only flag and command
names, but also argument and positional item values.

1. Enable `autocomplete` feature:

    ```toml
    bpaf = { version = "0.9", features = ["autocomplete"] }
	```

2. Decorate [`argument`](SimpleParser::argument) and [`positional`] parsers with
   [`Parser::complete`] to provide completion functions for arguments


3. Depending on your shell generate appropriate completion file and place it to whereever your
   shell is going to look for it, name of the file should correspond in some way to name of
   your program. Consult manual for your shell for the location and named conventions:

	 1. **bash**
		```console
		$ your_program --bpaf-complete-style-bash >> ~/.bash_completion
		```

	 1. **zsh**: note `_` at the beginning of the filename
		```console
		$ your_program --bpaf-complete-style-zsh > ~/.zsh/_your_program
		```

	 1. **fish**
		```console
		$ your_program --bpaf-complete-style-fish > ~/.config/fish/completions/your_program.fish
		```

	 1. **elvish**
		```console
		$ your_program --bpaf-complete-style-elvish >> ~/.config/elvish/rc.elv
		```

4. Restart your shell. Regenerating scripts and restarting the shell is needed only on the
   first attempt and between some major versions of `bpaf`: generated completion files contain
   only instructions how to ask your program for completions and not the completions themselves
   so files don't change even if options are.


5. Generated scripts rely on your program being accessible in `$PATH`
