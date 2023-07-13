//! This example implements parsers for a few coreutils tools - they tend to have complicated CLI
//! rules due to historical reasons
use bpaf::*;

mod boilerplate {
    use super::*;
    pub trait ExtraParsers<T> {
        /// Assuming parser can consume one of several values - try to consume them all and return
        /// last one
        fn my_last(self) -> ParseLast<T>;
    }

    impl<T> Parser<T> for ParseLast<T> {
        fn eval(&self, args: &mut State) -> Result<T, Error> {
            self.inner.eval(args)
        }

        fn meta(&self) -> Meta {
            self.inner.meta()
        }
    }

    pub struct ParseLast<T> {
        inner: Box<dyn Parser<T>>,
    }

    impl<T, P> ExtraParsers<T> for P
    where
        P: Parser<T> + 'static,
        T: 'static,
    {
        fn my_last(self) -> ParseLast<T> {
            let p = self
                .some("need to specify at least once")
                .map(|mut xs| xs.pop().unwrap());
            ParseLast { inner: p.boxed() }
        }
    }
}
pub mod shared {
    use super::boilerplate::*;
    use bpaf::*;

    #[derive(Debug, Clone, Copy, Bpaf)]
    pub enum Verbosity {
        /// Display warnings
        #[bpaf(short, long)]
        Warn,
        /// Display only diagnostics
        #[bpaf(short, long)]
        Quiet,
        /// Display status only
        #[bpaf(short, long)]
        Status,
    }

    pub fn parse_verbosity() -> impl Parser<Verbosity> {
        verbosity().my_last().fallback(Verbosity::Status)
    }

    pub fn parse_binary() -> impl Parser<bool> {
        #[derive(Debug, Clone, Copy, Bpaf, Eq, PartialEq)]
        enum Mode {
            /// Use binary mode
            #[bpaf(short, long)]
            Binary,
            /// Use text mode
            #[bpaf(short, long)]
            Text,
        }
        mode()
            .last()
            .fallback(Mode::Text)
            .debug_fallback()
            .map(|mode| mode == Mode::Binary)
    }
}

mod arch {
    use bpaf::*;

    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(command)]
    /// Print machine architecture.
    pub struct Arch;
}

mod b2sum {
    use super::shared::*;
    use bpaf::*;
    use std::path::PathBuf;

    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(command("b2sum"))]
    /// Print or check BLAKE2 (512-bit) checksums.
    pub struct B2Sum {
        #[bpaf(external(parse_binary))]
        pub binary: bool,

        /// read BLAKE2 sums from the FILEs and check them
        #[bpaf(short, long)]
        pub check: bool,

        /// create a BSD-style checksum
        pub tag: bool,

        #[bpaf(external(parse_verbosity))]
        pub check_output: Verbosity,

        /// exit non-zero for improperly formatted checksum lines
        pub strict: bool,

        #[bpaf(positional("FILE"))]
        pub files: Vec<PathBuf>,
    }
}

mod base32 {
    use bpaf::*;
    use std::path::PathBuf;

    fn non_zero(val: Option<usize>) -> Option<usize> {
        val.and_then(|v| (v > 0).then_some(v))
    }

    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(command)]
    /// Base32 encode or decode FILE, or standard input, to standard output.
    pub struct Base32 {
        /// decode data
        #[bpaf(long, short)]
        pub decode: bool,
        #[bpaf(long, short)]
        /// when decoding, ignore non-alphabet characters
        pub ignore_garbage: bool,

        #[bpaf(
            long,
            short,
            argument("COLS"),
            optional,
            map(non_zero),
            fallback(Some(76)),
            debug_fallback
        )]
        /// wrap encoded lines after COLS character
        ///  Use 0 to disable line wrapping
        pub wrap: Option<usize>,

        #[bpaf(positional("FILE"))]
        /// With no FILE, or when FILE is -, read standard input.
        pub file: Option<PathBuf>,
    }
}

mod basename {
    use bpaf::*;

    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(command)]
    pub struct Basename {
        /// support multiple arguments and treat each as a NAME
        #[bpaf(short('a'), long)]
        pub multiple: bool,

        /// remove a trailing SUFFIX; implies -a
        #[bpaf(short, long, argument("SUFFIX"), optional)]
        pub suffix: Option<String>,

        ///  end each output line with NUL, not newline
        #[bpaf(short, long)]
        pub zero: bool,

        /// Print NAME with any leading directory components removed.
        #[bpaf(positional("NAME"), many)]
        pub names: Vec<String>,
    }

    pub fn parse_basename() -> impl Parser<Basename> {
        basename().map(|mut b| {
            if b.suffix.is_some() {
                b.multiple = true;
            }
            b
        })
    }
}

mod cat {
    use std::path::PathBuf;

    use bpaf::*;

    #[derive(Debug, Clone, Bpaf)]
    struct Extra {
        #[bpaf(short('A'), long)]
        /// equivalent to -vET
        show_all: bool,

        #[bpaf(short('b'), long)]
        /// number nonempty output lines, overrides -n
        number_nonblank: bool,

        #[bpaf(short('e'))]
        /// equivalent to -vE
        show_non_printing_ends: bool,
    }

    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(fallback(NumberingMode::None))]
    pub enum NumberingMode {
        #[bpaf(hide)]
        /// Don't number lines, default behavior
        None,

        /// Number nonempty output lines, overrides -n
        #[bpaf(short('b'), long("number-nonblank"))]
        NonEmpty,

        /// Number all output lines
        #[bpaf(short('n'), long("number"))]
        All,
    }

    #[derive(Debug, Clone, Bpaf)]
    pub struct Cat {
        #[bpaf(short('T'), long)]
        /// display TAB characters as ^I
        pub show_tabs: bool,

        /// display $ at end of each line
        #[bpaf(short('E'))]
        pub show_ends: bool,

        /// use ^ and M- notation, except for LFD and TAB
        #[bpaf(short('n'), long("number"))]
        show_nonprinting: bool,

        #[bpaf(external(numbering_mode))]
        pub number: NumberingMode,

        #[bpaf(short('s'), long)]
        /// suppress repeated empty output lines
        pub squeeze_blank: bool,

        #[bpaf(positional("FILE"), many)]
        /// Concatenate FILE(s) to standard output.
        pub files: Vec<PathBuf>,
    }

    pub fn parse_cat() -> impl Parser<Cat> {
        construct!(extra(), cat())
            .map(|(extra, mut cat)| {
                if extra.show_all {
                    cat.show_tabs = true;
                    cat.show_ends = true;
                    cat.show_nonprinting = true;
                }
                if extra.show_non_printing_ends {
                    cat.show_nonprinting = true;
                    cat.show_ends = true;
                }
                if extra.number_nonblank {
                    cat.number = NumberingMode::NonEmpty;
                }
                cat
            })
            .to_options()
            .command("cat")
    }
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum Options {
    Arch(#[bpaf(external(arch::arch))] arch::Arch),
    B2Sum(#[bpaf(external(b2sum::b2_sum))] b2sum::B2Sum),
    Base32(#[bpaf(external(base32::base32))] base32::Base32),
    Basename(#[bpaf(external(basename::parse_basename))] basename::Basename),
    Cat(#[bpaf(external(cat::parse_cat))] cat::Cat),
}

fn main() {
    let parser = options();

    println!("{:?}", parser.run());
}
