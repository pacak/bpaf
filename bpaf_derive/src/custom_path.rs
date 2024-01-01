use syn::{
    punctuated::Punctuated,
    token::{self, PathSep},
    visit_mut::{self, VisitMut},
    PathSegment, UseName, UsePath, UseRename, UseTree,
};

/// Implements [`syn::visit_mut::VisitMut`] to find
/// those [`Path`](syn::Path)s which match
/// [`query`](Self::target) and replace them with [`target`](Self::target).
pub(crate) struct CratePathReplacer {
    /// The prefix to search for within an input path.
    query: syn::Path,
    /// The prefix we wish the input path to have.
    target: syn::Path,
}

impl CratePathReplacer {
    pub(crate) fn new(target: syn::Path, replacement: syn::Path) -> Self {
        CratePathReplacer {
            query: target,
            target: replacement,
        }
    }

    /// Check if both [`query`](Self::query) and `input` have the same leading
    /// path segment (`::`) responsible for marking [a path as
    /// "global"](https://doc.rust-lang.org/reference/procedural-macros.html#procedural-macro-hygiene).
    ///
    /// If these do not match, no replacement will be performed.
    fn path_global_match(&self, input: &mut syn::Path) -> bool {
        self.query.leading_colon.is_some() && input.leading_colon.is_some()
    }

    /// Check if the initial segments of `input` match [`query`](Self::query).
    ///
    /// If these do not match, no replacement will be performed.
    fn path_segments_match(&self, input: &mut syn::Path) -> bool {
        self.query
            .segments
            .iter()
            .zip(input.segments.iter())
            .all(|(f, o)| f == o)
    }

    /// Replaces the prefix of `input` with those of [`target`](Self::target) if
    /// the `input` path's prefix matches [`query`](Self::query).
    fn replace_path_if_match(&self, input: &mut syn::Path) {
        if self.path_global_match(input) && self.path_segments_match(input) {
            input.leading_colon = self.target.leading_colon;
            input.segments = self
                .target
                .segments
                .clone()
                .into_iter()
                .chain(
                    input
                        .segments
                        .iter()
                        .skip(self.query.segments.iter().count())
                        .cloned(),
                )
                .collect::<Punctuated<_, _>>();
        }
    }

    fn item_use_global_match(&self, input: &syn::ItemUse) -> bool {
        self.query.leading_colon == input.leading_colon
    }

    fn item_use_segments_match<'a, Q: Iterator<Item = &'a PathSegment>>(
        input: &'a UseTree,
        query_len: usize,
        mut query_iter: Q,
        mut matched_parts: Vec<&'a UseTree>,
    ) -> Option<(Vec<&'a UseTree>, Option<UseTree>)> {
        if let Some(next_to_match) = query_iter.next() {
            match input {
                UseTree::Path(path) => {
                    if next_to_match.ident == path.ident {
                        matched_parts.push(input);
                        return Self::item_use_segments_match(
                            path.tree.as_ref(),
                            query_len,
                            query_iter,
                            matched_parts,
                        );
                    }
                }
                UseTree::Name(name) => {
                    if next_to_match.ident == name.ident {
                        if query_iter.next().is_some() {
                            return None;
                        } else {
                            matched_parts.push(input);
                        }
                    }
                }
                UseTree::Rename(rename) => {
                    if next_to_match.ident == rename.ident {
                        if query_iter.next().is_some() {
                            return None;
                        } else {
                            matched_parts.push(input);
                        }
                    }
                }
                UseTree::Glob(_) => {}
                UseTree::Group(_) => {}
            }
        }

        if query_len == matched_parts.len() {
            Some((matched_parts, Some(input.clone())))
        } else {
            None
        }
    }

    fn append_suffix_to_target(
        &self,
        matched_parts: Vec<&UseTree>,
        suffix: Option<UseTree>,
    ) -> UseTree {
        let last_input_match = matched_parts
            .last()
            .expect("If a match exists, then it the matched prefix must be non-empty.");
        let mut rev_target_ids = self.target.segments.iter().map(|s| s.ident.clone()).rev();
        let mut result_tree = match last_input_match {
            UseTree::Path(_) => {
                if let Some(suffix_tree) = suffix {
                    UseTree::Path(UsePath {
                        ident: rev_target_ids.next().expect(
                            "error while making a `UseTree::Path`: target should not be empty",
                        ),
                        colon2_token: PathSep::default(),
                        tree: Box::new(suffix_tree),
                    })
                } else {
                    unreachable!("If the last part of the matched input was a path, then there must be some suffix left to attach to complete it.")
                }
            }
            UseTree::Name(_) => {
                assert!(suffix.is_none(), "If the last part of the matched input was a syn::UseTree::Name, then there shouldn't be any suffix left to attach to the prefix.");
                UseTree::Name(UseName {
                    ident: rev_target_ids
                        .next()
                        .expect("error while making a `UseTree::Name`: target should not be empty"),
                })
            }
            UseTree::Rename(original_rename) => {
                assert!(suffix.is_none(), "If the last part of the matched input was a syn::UseTree::Rename, then there shouldn't be any suffix left to attach to the prefix.");
                UseTree::Rename(UseRename {
                    ident: rev_target_ids.next().expect(
                        "error while making a `UseTree::Rename`: target should not be empty",
                    ),
                    as_token: token::As::default(),
                    rename: original_rename.rename.clone(),
                })
            }
            UseTree::Glob(_) => unreachable!(
                "There is no functionality for matching against a syn::UseTree::Group."
            ),
            UseTree::Group(_) => unreachable!(
                "There is no functionality for matching against a syn::UseTree::Group."
            ),
        };
        for id in rev_target_ids {
            result_tree = UseTree::Path(UsePath {
                ident: id,
                colon2_token: PathSep::default(),
                tree: Box::new(result_tree),
            })
        }
        result_tree
    }

    /// Replaces the prefix of `input` with those of [`target`](Self::target) if
    /// the `input` path's prefix matches [`query`](Self::query).
    fn replace_item_use_if_match(&self, input: &mut syn::ItemUse) {
        if self.item_use_global_match(input) {
            if let Some((matched_prefix, suffix)) = Self::item_use_segments_match(
                &input.tree,
                self.query.segments.len(),
                self.query.segments.iter(),
                vec![],
            ) {
                input.leading_colon = self.target.leading_colon;
                input.tree = self.append_suffix_to_target(matched_prefix, suffix);
            }
        }
    }
}

impl VisitMut for CratePathReplacer {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        self.replace_path_if_match(path);
        visit_mut::visit_path_mut(self, path);
    }

    fn visit_item_use_mut(&mut self, item_use: &mut syn::ItemUse) {
        self.replace_item_use_if_match(item_use);
        visit_mut::visit_item_use_mut(self, item_use);
    }
}
