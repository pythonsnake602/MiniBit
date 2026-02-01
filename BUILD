load("@crates//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_binary")
load("@aspect_bazel_lib//lib:copy_file.bzl", "copy_file")

config_setting(
    name = "optimize",
    values = {"compilation_mode": "opt"}
)

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
    rustc_flags = select({
        ":optimize": ["--codegen=opt-level=3"],
        "//conditions:default": ["--codegen=opt-level=1"],
    }),
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
    rustc_flags = select({
        ":optimize": ["--codegen=opt-level=3"],
        "//conditions:default": ["--codegen=opt-level=1"],
    }) + select({
        "@platforms//os:macos": ["-L/opt/homebrew/opt/libpq/lib"],
        "//conditions:default": [],
    }),
    data = glob(["data/**/*"]),
)
