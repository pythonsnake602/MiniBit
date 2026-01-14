load("@crates//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_binary")

rust_library(
    name = "minibit_lib",
    srcs = glob(["src/lib/**/*.rs"]),
    crate_root = "src/lib/mod.rs",
    compile_data = ["//:Cargo.toml"],
    aliases = aliases(),
    deps = all_crate_deps(
        normal = True,
    ),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
)

rust_binary(
    name = "minibit_server",
    srcs = glob(["src/bin/**/*.rs"]),
    compile_data = ["//:Cargo.toml"],
    aliases = aliases(),
    deps = [":minibit_lib"] + all_crate_deps(
        normal = True,
    ),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
)

load("@aspect_bazel_lib//lib:run_binary.bzl", "run_binary")

run_binary(
    name = "config",
    tool = "config/generate_all.sh",
    srcs = ["config/run", "config/proxy", "@velocity//jar", "//velocity:minibit_plugin"],
    args = ["$(location config/run)", "$(location config/proxy)", "$(location @velocity//jar)", "$(location //velocity:minibit_plugin)", "$(RULEDIR)"],
    out_dirs = ["run", "proxy"],
)

