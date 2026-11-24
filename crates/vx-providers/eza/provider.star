load("@vx//stdlib:rez.star", "rez_bundle_source")

# provider.star - eza provider
#
# eza: A modern replacement for ls (community fork of exa)
# Releases: https://github.com/eza-community/eza/releases
# Asset format: eza_{triple}.{ext}  (NO version in asset name)
# Windows:      eza.exe_{triple}.zip
# Tag format:   v{version}
#
# Note: eza does NOT publish macOS binaries.
#       Only Linux (x64, arm64) and Windows (x64) are available.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:layout.star", "archive_layout")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "eza"
description = "eza - A modern replacement for ls"
homepage    = "https://github.com/eza-community/eza"
repository  = "https://github.com/eza-community/eza"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("eza",
        version_pattern     = "\\d+\\.\\d+",
        platform_constraint = {"os": ["linux", "windows"]},
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Platform mapping
# Note: eza does NOT publish macOS binaries — macOS is not supported.
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("x86_64-pc-windows-gnu",      "zip"),
    "linux/x64":     ("x86_64-unknown-linux-gnu",   "tar.gz"),
    "linux/arm64":   ("aarch64-unknown-linux-gnu",  "tar.gz"),
}

# ---------------------------------------------------------------------------
# Provider functions
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("eza-community", "eza", tag_prefix = "v")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    p = _PLATFORMS.get(key)
    if not p:
        return None  # macOS and other platforms not supported
    triple, ext = p
    if ctx.platform.os == "windows":
        fname = "eza.exe_{}.{}".format(triple, ext)
    else:
        fname = "eza_{}.{}".format(triple, ext)
    return "https://github.com/eza-community/eza/releases/download/v{}/{}".format(version, fname)

install_layout = archive_layout("eza")

paths = path_fns("eza")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
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
    "eza",
    published = False,
)
rez_fetch_versions = _REZ_SDK["fetch_versions"]
rez_download_url = _REZ_SDK["download_url"]
rez_install_layout = _REZ_SDK["install_layout"]
rez_bundle = _REZ_SDK["rez_bundle"]