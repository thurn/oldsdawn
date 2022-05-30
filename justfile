code-review: git-status check-format build clippy test check-docs

clean:
    rm -f target/.rustc_info.json
    cargo clean
    mkdir target
    xattr -w com.dropbox.ignored 1 target
    find . -name "*.profraw" -delete

check:
    cargo check --all-targets --all-features

build:
    cargo build --all-targets --all-features

run:
    # Use +nightly in order to get backtraces for anyhow errors
    RUST_BACKTRACE=1 && cargo +nightly run

test:
    cargo test

test-backtrace:
    # Use +nightly in order to get backtraces for anyhow errors
    RUST_BACKTRACE=1 && cargo +nightly test
doc:
    cargo doc

fix: git-status fix-lints fmt fix-clippy

fix-amend: git-status fix-lints git-amend1 fmt git-amend2 fix-clippy git-amend3

tournament:
    cargo run --bin tournament

clippy:
    cargo clippy --workspace --exclude "protos" -- \
        -D warnings \
        -D clippy::all \
        -A clippy::needless-update \
        -D clippy::cast_lossless \
        -D clippy::cloned_instead_of_copied \
        -D clippy::copy_iterator \
        -D clippy::default_trait_access \
        -D clippy::inconsistent_struct_constructor \
        -D clippy::inefficient_to_string \
        -D clippy::integer_division \
        -D clippy::let_underscore_drop \
        -D clippy::let_underscore_must_use \
        -D clippy::manual_ok_or \
        -D clippy::map_flatten \
        -D clippy::map_unwrap_or \
        -D clippy::multiple_inherent_impl \
        -D clippy::needless_continue \
        -D clippy::needless_for_each \
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

# Reformats code. Requires nightly because several useful options (e.g. imports_granularity) are
# nightly-only
fmt:
    cargo +nightly fmt

check-format:
    cargo +nightly fmt -- --check

fix-lints:
    cargo fix --all-features

fix-clippy:
    cargo clippy --fix

clippy-fix:
    cargo clippy --fix -- \
        -D warnings \
        -D clippy::all \
        -A clippy::needless-update \
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

snapshots:
    cargo insta review

benchmark:
    # The 'inventory' and 'linkme' crates have both been semi-broken since august 2021,
    # this works around those issues.
    # See https://github.com/dtolnay/inventory/issues/32
    RUSTFLAGS="-C codegen-units=1" cargo criterion -p spelldawn

# Checks documentation lints, haven't figured out how to do this with a single command
check-docs:
    #!/usr/bin/env sh
    set -euxo pipefail
    for file in `ls crates | grep -v 'spelldawn'`; do
        echo "Checking rustdoc for $file";
        cargo rustdoc --lib -p $file -- \
            -D rustdoc::broken-intra-doc-links \
            -D rustdoc::private-intra-doc-links \
            -D rustdoc::missing-crate-level-docs \
            -D rustdoc::bare-urls;
    done

outdated:
    # Check for outdated dependencies, consider running 'cargo update' if this fails
    cargo outdated --exit-code 1

udeps: clean
    # Currently seems to panic if you don't clean first
    cargo +nightly udeps

time-passes: clean
    cargo +nightly rustc -p spelldawn --bin spelldawn -- -Z time-passes

timings: clean
    cargo +nightly build -p spelldawn --bin spelldawn -Z timings --release

# Builds .gcda files used for code coverage
gen-gcda: clean
    #!/usr/bin/env sh
    set -euxo pipefail
    export LLVM_PROFILE_FILE='spelldawn-%p-%m.profraw'
    export RUSTFLAGS='-Zinstrument-coverage'
    cargo +nightly build
    cargo +nightly test # Generates .profraw files in the current working directory
    export CARGO_INCREMENTAL=0
    export RUSTFLAGS='-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort'
    export RUSTDOCFLAGS="-Cpanic=abort"
    cargo +nightly build # Build .gcno files in ./target/debug/deps/
    cargo +nightly test # Build .gcda files in ./target/debug/deps/

# Displays test coverage information in a web browser
coverage: gen-gcda
    grcov . -s . \
        --binary-path ./target/debug/ \
        -t html \
        --branch \
        --ignore-not-existing \
        -o ./target/debug/coverage
    open target/debug/coverage/index.html

# Checks for uncommitted repository changes
git-status:
    git diff-index --quiet HEAD --
    git ls-files --exclude-standard --others

git-amend1:
    git commit -a --amend -C HEAD

git-amend2:
    git commit -a --amend -C HEAD

git-amend3:
    git commit -a --amend -C HEAD
