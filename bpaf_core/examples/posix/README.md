A regular [`Positional`] parser accepts the next operand wherever it appears on
the command line. Calling [`Positional::posix`](Positional::posix) switches the
parser into POSIX mode: `bpaf` accepts named items only before it and treats
everything after it as an operand.
