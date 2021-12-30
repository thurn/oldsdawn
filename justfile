code-review: clean build outdated clippy check-format check-docs test udeps

fix: fix-format fix-lints

clean:
    cargo clean
    find . -name "*.profraw" -delete
    mkdir target
    xattr -w com.dropbox.ignored 1 target

check:
    cargo check --all-targets --all-features

build:
    cargo build --all-targets --all-features

run:
    cargo run

test:
    cargo test

doc:
    cargo doc --open

clippy:
    cargo clippy --workspace --exclude "protos" -- \
        -D warnings \
        -D clippy::all \
        -D clippy::cast_lossless \
        -D clippy::cloned_instead_of_copied \
        -D clippy::copy_iterator \
        -D clippy::default_trait_access \
        -D clippy::if_then_some_else_none \
        -D clippy::inconsistent_struct_constructor \
        -D clippy::inefficient_to_string \
        -D clippy::integer_division \
        -D clippy::let_underscore_drop \
        -D clippy::let_underscore_must_use \
        -D clippy::manual_ok_or \
        -D clippy::map_flatten \
        -D clippy::map_unwrap_or \
        -D clippy::match_same_arms \
        -D clippy::multiple_inherent_impl \
        -D clippy::needless_continue \
        -D clippy::needless_for_each \
        -D clippy::option_if_let_else \
        -D clippy::redundant_closure_for_method_calls \
        -D clippy::ref_option_ref \
        -D clippy::string_to_string \
        -D clippy::trait_duplication_in_bounds \
        -D clippy::unnecessary_self_imports \
        -D clippy::unnested_or_patterns \
        -D clippy::unused_self \
        -D clippy::unwrap_in_result \
        -D clippy::used_underscore_binding \
        -D clippy::useless_let_if_seq \
        -D clippy::use_self

# Reformats code. Requires nightly because several useful options (e.g. imports_granularity) are
# nightly-only
fix-format:
    cargo +nightly fmt

check-format:
    cargo +nightly fmt -- --check

fix-lints:
    cargo fix --all-features

# Checks documentation lints, haven't figured out how to do this with a single command
check-docs:
    #!/usr/bin/env sh
    set -euxo pipefail
    for file in `ls crates | grep -v 'spelldawn'`; do
        echo "Checking $file";
        cargo rustdoc --lib -p $file -- \
            -D rustdoc::broken-intra-doc-links \
            -D rustdoc::private-intra-doc-links \
            -D rustdoc::missing-crate-level-docs \
            -D rustdoc::bare-urls;
    done

outdated:
    cargo outdated --exit-code 1

udeps: clean
    # Currently seems to panic sometimes without running clean first clean
    cargo +nightly udeps

time-passes: clean
    cargo +nightly rustc -p spelldawn --bin spelldawn -- -Z time-passes

timings: clean
    cargo +nightly build -p spelldawn --bin spelldawn -Z timings --release

gen-gcda: clean
    #!/usr/bin/env sh
    set -euxo pipefail
    RUSTC_BOOTSTRAP=1 # Causes the compiler to behave like nightly
    LLVM_PROFILE_FILE='spelldawn-%p-%m.profraw'
    RUSTFLAGS='-Zinstrument-coverage'
    cargo build
    cargo test
    CARGO_INCREMENTAL=0
    RUSTFLAGS='-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort'
    RUSTDOCFLAGS="-Cpanic=abort"
    cargo build
    cargo test

# Displays test coverage information in a web browser
coverage: gen-gcda
    grcov . -s . \
        --binary-path ./target/debug/ \
        -t html \
        --branch \
        --ignore-not-existing \
        -o ./target/debug/coverage
    open target/debug/coverage/index.html
