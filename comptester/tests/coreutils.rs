use comptester::*;
use pretty_assertions::assert_eq;

#[test]
fn all_options_zsh() {
    let buf = zsh_comptest("coreutils \t").unwrap();
    let expected = r"% coreutils
arch                     -- Print machine architecture.
b2sum                    -- Print or check BLAKE2 (512-bit) checksums.
base32                   -- Base32 encode or decode FILE, or standard input, to standard output.
basename
cat";
    assert_eq!(buf, expected);
}

#[test]
fn all_options_bash() {
    let buf = bash_comptest("coreutils \t\t").unwrap();
    let expected = r"%
arch                     -- Print machine architecture.
b2sum                    -- Print or check BLAKE2 (512-bit) checksums.
base32                   -- Base32 encode or decode FILE, or standard input, to standard output.
basename
cat";
    assert_eq!(buf, expected);
}

#[test]
fn cat_zsh() {
    let buf = zsh_comptest("coreutils cat -- \t").unwrap();
    assert_eq!(
        buf,
        r"% coreutils cat --
      FILE"
    );
}

#[test]
fn cat_bash() {
    let buf = bash_comptest("coreutils cat -- \t\t").unwrap();
    assert_eq!(buf, "%\nFILE");
}
