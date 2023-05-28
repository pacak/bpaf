//
use bpaf::*;
const DB: &str = "DATABASE_VAR";

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Use verbose output
    // No name annotation and name is not a single character:
    // `bpaf` uses it as a long name - `--verbose`
    pub verbose: bool,

    /// Compile in a release mode
    #[bpaf(short)]
    // Name is long, but explicit annotation for a short name
    // `bpaf` makes a short name from the first symbol: `-r`
    pub release: bool,

    /// Number of parallel jobs, defaults to # of CPUs
    // Explicit annotation with a short name: `-j`
    #[bpaf(short('j'))]
    pub threads: Option<usize>,

    /// Upload artifacts to the storage
    // Explicit annotation for a single suppresses the oher one,
    // but you can specify both of them. `-u` and `--upload`
    #[bpaf(short, long)]
    pub upload: bool,

    /// List of features to activate
    // you can mix explicit annotations with and without names
    // when convenient, here it's `-F` and `--features`
    #[bpaf(short('F'), long)]
    pub features: Vec<String>,

    /// Read information from the database
    #[bpaf(env(DB))]
    // Annotation for `env` does not affect annotation for names
    // this
    pub database: String,

    /// Only print essential information
    #[bpaf(short, long, long("essential"))]
    // `--essential` is a hidden ailias, `-q` and `--quiet` are visible
    pub quiet: bool,

    /// implicit long + env variable "USER"
    #[bpaf(env("USER"))]
    pub user: String,
}
