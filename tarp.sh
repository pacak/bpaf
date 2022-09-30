cargo tarpaulin --target-dir tarp \
    --workspace \
    --all-features \
    --exclude-files tests/*.rs \
    --exclude-files docs/*/*.rs \
    --exclude-files docs/*/*/*.rs \
    --exclude-files docs/build.rs \
    --exclude-files bpaf_cauwugo/src/*.rs \
    --exclude-files bpaf_derive/src/*_tests.rs \
    --exclude-files src/tests.rs
