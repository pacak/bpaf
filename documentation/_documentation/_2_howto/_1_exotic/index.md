#### Parsing exotic options
You can parse a lot of unusual types of options - for legacy compatibility or other reasons


# Some of the more unusual examples

While `bpaf`'s design tries to cover most common use cases, mostly
[posix conventions](https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/basedefs/V1_chap12.html),
it can also handle some more unusual requirements. It might come at a cost of having to write
more code, more confusing error messages or worse performance, but it will get the job done.
