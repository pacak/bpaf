#### Combinatoric API
Parse arguments without using proc macros

When making a parser in the Combinatoric style API you usually go through those steps

1. Design data type your application will receive
2. Design command line options user will have to pass
3. Create a set of simple parsers
4. Combine and transform simple parsers to create the final data type
5. Transform the resulting [`Parser`] into [`OptionParser`] to add extra header/footer and run it
