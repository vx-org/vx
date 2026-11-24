load("@vx//stdlib:rez.star", "rez_bundle_source")

# provider.star - pre-commit provider
#
# pre-commit is a Python tool run via `uvx pre-commit`.
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star", "runtime_def", "dep_def", "github_permissions")
load("@vx//stdlib:github.star",   "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name          = "pre-commit"
description   = "A framework for managing and maintaining multi-language pre-commit hooks"
homepage      = "https://pre-commit.com"
repository    = "https://github.com/pre-commit/pre-commit"
license       = "MIT"
ecosystem     = "python"

# RFC 0033: route `vx pre-commit` → `vx uvx pre-commit`
package_alias = {"ecosystem": "uvx", "package": "pre-commit"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("pre-commit",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "pre-commit \\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("pre-commit", "pre-commit")

# ---------------------------------------------------------------------------
# download_url — installed via uvx
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries
# ---------------------------------------------------------------------------

def store_root(_ctx):
    return None

def get_execute_path(_ctx, _version):
    return None

def post_install(_ctx, _version):
    return None

def environment(_ctx, _version):
    return []

# ---------------------------------------------------------------------------
# deps — requires uv
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [dep_def("uv", reason = "pre-commit is installed and run via uv")]
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
    "pre-commit",
    published = False,
)
rez_fetch_versions = _REZ_SDK["fetch_versions"]
rez_download_url = _REZ_SDK["download_url"]
rez_install_layout = _REZ_SDK["install_layout"]
rez_bundle = _REZ_SDK["rez_bundle"]