use std::{borrow::Cow, ffi::OsString, rc::Rc};

use crate::{Error, Metavar, Name, Named, utils::Vec1};
impl From<CompleteReply> for Error {
    fn from(value: CompleteReply) -> Self {
        Error::CompleteReply(Vec1::new(value))
    }
}

// this could be a method on `Command<T>`, but will it monomorphise?
pub(crate) fn complete_command(names: &[Cow<'static, str>], err: Error) -> Error {
    let Error::CompleteRequest(ref comp) = err else {
        return err;
    };
    let prefix = match comp {
        CompleteReq::Anything => None,
        CompleteReq::Literal { prefix } => Some(prefix.as_ref()),
        CompleteReq::Name { .. } | CompleteReq::Value(..) => return err,
    };
    for name in names {
        if prefix.map_or(true, |p| name.starts_with(p)) {
            return CompleteReply::Command {
                name: name.clone(),
                help: None,
            }
            .into();
        }
    }
    Error::Killed
}

impl Named {
    pub(crate) fn complete_name(&self, err: Error, meta: Option<Metavar>) -> Error {
        let Error::CompleteRequest(ref comp) = err else {
            return err;
        };
        let mut name = None;
        match comp {
            CompleteReq::Name { prefix: None } | CompleteReq::Anything => {
                // any name with a preference to long
                for n in self.names.iter() {
                    match n {
                        Name::Short(_) => {
                            if name.is_none() {
                                name = Some(n);
                            }
                        }
                        Name::Long(_) => {
                            name = Some(n);
                            break;
                        }
                    }
                }
            }
            CompleteReq::Name {
                prefix: Some(prefix),
            } => {
                // long that starts with a given prefix
                for n in self.names.iter() {
                    if let Name::Long(s) = n
                        && s.starts_with(prefix.as_ref())
                    {
                        name = Some(n);
                        break;
                    }
                }
            }
            CompleteReq::Literal { .. } | CompleteReq::Value(..) => return Error::Internal,
        };
        if let Some(name) = name.cloned() {
            let help = self.help.clone();
            CompleteReply::Named { name, meta, help }.into()
        } else {
            Error::Killed
        }
    }
}

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
pub(crate) enum CompleteReply {
    Value(OsString),
    Named {
        name: Name<'static>,
        meta: Option<Metavar>,
        help: Option<String>,
    },
    Command {
        name: Cow<'static, str>,
        help: Option<String>,
    },
}

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
pub(crate) enum CompleteReq {
    /// We are trying to complete a name, `Some` indicates a long name, `None` - a short name
    /// where user typed just `-`
    Name {
        prefix: Option<Rc<str>>,
    },

    //
    Anything,
    Literal {
        prefix: Rc<str>,
    },
    Value(OsString),
}

pub(crate) fn render_completions(items: Vec1<CompleteReply>) -> String {
    render_completions_int(items).unwrap()
}
pub(crate) fn render_completions_int(
    mut items: Vec1<CompleteReply>,
) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    items.sort();
    let mut out = String::new();
    for item in items.as_slice() {
        match item {
            CompleteReply::Named { name, meta, help } => write!(&mut out, "{name}")?,
            CompleteReply::Value(os_string) => todo!(),
            CompleteReply::Command { name, help } => write!(&mut out, "{name}")?,
        }
    }
    Ok(out)
}
