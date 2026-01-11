genrule(
    name = "minibit",
    srcs = [
        "Cargo.toml",
        "Cargo.lock",
    ] + glob(["src/**"]),
    outs = ["minibit"],
    cmd = "./$(location bazel_build_minibit.sh) $@",
    tools = ["bazel_build_minibit.sh"],
    tags = ["no-sandbox"],
    executable = True,
)