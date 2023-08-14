use bpaf::*;

pub fn render_res<T: std::fmt::Debug>(res: Result<T, bpaf::ParseFailure>) -> String {
    match res {
        Ok(x) => format!("{x:?}"),
        Err(e) => match e {
            bpaf::ParseFailure::Stdout(doc, complete) => doc.render_markdown(complete),
            bpaf::ParseFailure::Completion(d) => d,
            bpaf::ParseFailure::Stderr(d) => d.render_markdown(true),
        },
    }
}

include!(concat!(env!("OUT_DIR"), "/lib.rs"));
