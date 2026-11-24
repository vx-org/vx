load("@vx//stdlib:rez.star", "rez_bundle_source")

# provider.star - maturin provider
#
# maturin: Build and publish crates with pyo3, cffi and uniffi bindings
# as well as Rust binaries as Python packages.
# Releases: https://github.com/PyO3/maturin/releases
# Asset format: maturin-{triple}.{ext}  (no version in filename)
# Tag format:   v{version}
# Linux uses musl for static linking.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_rust_provider")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "maturin"
description = "maturin - Build and publish crates with pyo3, cffi and uniffi bindings"
homepage    = "https://www.maturin.rs"
repository  = "https://github.com/PyO3/maturin"
license     = "MIT OR Apache-2.0"
ecosystem   = "python"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("maturin",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "maturin \\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template - github_rust_provider
#
# Asset: maturin-{triple}.{ext}  (no version in filename)
# Tag:   v{version}
# Linux: musl (static binary)
# ---------------------------------------------------------------------------

_p = github_rust_provider(
    "PyO3", "maturin",
    asset      = "maturin-{triple}.{ext}",
    linux_libc = "musl",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
# ---------------------------------------------------------------------------
# Rez Next SDK provider descriptor
#
# The descriptor is ready for the provider's immutable Rez bundle release.
# Existing provider installation remains authoritative until that release is
# published, keeping migration independent from bundle publication order.
# ---------------------------------------------------------------------------

_REZ_SDK = rez_bundle_source(
    "vx-org",
    "vx-rez-packages",
    "maturin",
    published = False,
)
rez_fetch_versions = _REZ_SDK["fetch_versions"]
rez_download_url = _REZ_SDK["download_url"]
rez_install_layout = _REZ_SDK["install_layout"]
rez_bundle = _REZ_SDK["rez_bundle"]