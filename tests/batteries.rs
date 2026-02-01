use bpaf::batteries::toggle_flag;
use bpaf::*;

#[test]
fn test_toggle_flag() {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    enum Flag {
        Y,
        N,
    }

    let parser = toggle_flag(short('y'), Flag::Y, short('n'), Flag::N).to_options();
    // let parser = short('y').req_flag(Flag::Y).or_else(short('n').req_flag(Flag::N)).last();

    let r = parser.run_inner(&[]).unwrap();
    assert_eq!(r, None);

    let r = parser.run_inner(&["-y", "-y", "-n"]).unwrap();
    assert_eq!(r, Some(Flag::N));

    let r = parser.run_inner(&["-y", "-y", "-n", "-y"]).unwrap();
    assert_eq!(r, Some(Flag::Y));
}
