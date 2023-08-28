use bpaf::config::*;
use bpaf::*;

struct DummyConfig(usize);

impl Config for DummyConfig {
    fn get(
        &self,
        path: &[(&'static str, usize)],
        name: &'static str,
        num: usize,
    ) -> Option<String> {
        use std::fmt::Write;
        let mut res = String::new();

        for (name, ix) in path {
            if *ix >= self.0 {
                return None;
            }
            write!(&mut res, "{}[{}].", name, ix).ok()?
        }
        if num >= self.0 {
            None
        } else {
            write!(&mut res, "{}[{}]", name, num).ok()?;
            Some(res)
        }
    }
}

#[test]
fn basic_config() {
    let cfg = [("name".into(), "Bob".into()), ("age".into(), "21".into())]
        .into_iter()
        .collect::<std::collections::BTreeMap<String, String>>();

    let name = long("name").key("name").argument::<String>("NAME");

    let age = long("age").key("age").argument::<usize>("AGE");
    let parser = construct!(name, age).to_options();

    let args = Args::from(&[]).with_config(cfg.clone());
    let r = parser.run_inner(args).unwrap();
    assert_eq!(r.0, "Bob");
    assert_eq!(r.1, 21);
}

#[test]
fn many_enter() {
    let parser = long("name")
        .key("name")
        .argument::<String>("NAME")
        .many()
        .enter("group")
        .to_options();

    let args = Args::from(&[]).with_config(DummyConfig(4));
    let r = parser.run_inner(args).unwrap();

    assert_eq!(
        r,
        [
            "group[0].name[0]",
            "group[0].name[1]",
            "group[0].name[2]",
            "group[0].name[3]"
        ]
    );
}

#[test]
fn enter_many() {
    let parser = long("name")
        .key("name")
        .argument::<String>("NAME")
        .enter("group")
        .many()
        .to_options();

    let args = Args::from(&[]).with_config(DummyConfig(4));
    let r = parser.run_inner(args).unwrap();

    assert_eq!(
        r,
        [
            "group[0].name[0]",
            "group[1].name[0]",
            "group[2].name[0]",
            "group[3].name[0]"
        ]
    );
}

#[test]
fn many_enter_many() {
    let parser = long("name")
        .key("name")
        .argument::<String>("NAME")
        .many()
        .enter("group")
        .many()
        .map(|x| x.into_iter().flatten().collect::<Vec<_>>())
        .to_options();

    let args = Args::from(&[]).with_config(DummyConfig(2));
    let r = parser.run_inner(args).unwrap();

    assert_eq!(
        r,
        [
            "group[0].name[0]",
            "group[0].name[1]",
            "group[1].name[0]",
            "group[1].name[1]",
        ]
    );
}
