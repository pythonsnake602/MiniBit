load("@aspect_bazel_lib//lib:expand_template.bzl", "expand_template")

expand_template(
    name = "build_constants",
    template = "src/main/templates/minibit/minibitvelocity/BuildConstants.java",
    substitutions = {
        "${version}": "1.0-SNAPSHOT",
    },
    visibility = ["//visibility:private"],
)

java_plugin(
    name = "velocity_plugin",
    deps = ["@velocity//jar"],
    processor_class = "com.velocitypowered.api.plugin.ap.PluginAnnotationProcessor",
    visibility = ["//visibility:private"],
)

java_library(
    name = "minibit_plugin",
    deps = ["@velocity//jar"],
    srcs = glob(["src/main/java/**/*.java"]) + [":build_constants"],
    plugins = [":velocity_plugin"],
    visibility = ["//visibility:public"],
)
