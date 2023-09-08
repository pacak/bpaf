pub fn render_res<T: std::fmt::Debug>(res: Result<T, bpaf::ParseFailure>) -> String {
    match res {
        Ok(x) => format!("{x:?}\n"),
        Err(e) => match e {
            bpaf::ParseFailure::Stdout(doc, complete) => doc.monochrome(complete),
            bpaf::ParseFailure::Completion(d) => d,
            bpaf::ParseFailure::Stderr(d) => format!("Error: {}\n", d.monochrome(true)),
        },
    }
}

include!(concat!(env!("OUT_DIR"), "/lib.rs"));
