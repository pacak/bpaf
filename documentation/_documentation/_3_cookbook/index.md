#### Parsing cookbook
How to parse less frequent combinations

While `bpaf`'s design tries to cover the most common use cases, mostly
[posix conventions](https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/basedefs/V1_chap12.html),
it can also handle some more unusual requirements. It might come at the cost of having to write
more code, more confusing error messages or worse performance, but it will get the job done.
