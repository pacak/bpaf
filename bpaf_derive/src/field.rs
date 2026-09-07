use syn::{PathArguments, Type};

#[derive(Debug, Copy, Clone)]
pub(crate) enum Shape {
    Optional,
    Multiple,
    Bool,
    Unit,
    Plain,
}

impl Shape {
    pub(crate) fn make(ty: &Type) -> Self {
        fn one_fish(x: &PathArguments) -> Option<&Type> {
            if let PathArguments::AngleBracketed(arg) = x
                && arg.args.len() == 1
                && let Some(syn::GenericArgument::Type(ty)) = arg.args.first()
            {
                Some(ty)
            } else {
                None
            }
        }

        fn try_make(ty: &Type) -> Option<Shape> {
            if let Type::Tuple(syn::TypeTuple { elems, .. }) = ty
                && elems.is_empty()
            {
                return Some(Shape::Unit);
            }

            let last = match ty {
                Type::Path(p) => p.path.segments.last()?,
                _ => return None,
            };
            if last.ident == "Vec" {
                one_fish(&last.arguments)?;
                Some(Shape::Multiple)
            } else if last.ident == "Option" {
                one_fish(&last.arguments)?;
                Some(Shape::Optional)
            } else if last.ident == "bool" {
                Some(Shape::Bool)
            } else {
                None
            }
        }

        try_make(ty).unwrap_or(Shape::Plain)
    }
}
