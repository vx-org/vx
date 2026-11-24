load("@vx//stdlib:rez.star", "rez_bundle_source")

# provider.star - curl provider
#
# cURL is pre-installed on most modern systems (Windows 10+, macOS, Linux).
# vx only detects the system installation — no download managed.
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star", "runtime_def", "system_permissions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "curl"
description = "Command-line tool for transferring data with URLs"
homepage    = "https://curl.se"
repository  = "https://github.com/curl/curl"
license     = "MIT"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("curl",
        system_paths = [
            "C:/Windows/System32/curl.exe",
            "C:/Program Files/Git/mingw64/bin/curl.exe",
            "C:/msys64/usr/bin/curl.exe",
            "/usr/bin/curl",
            "/usr/local/bin/curl",
            "/opt/homebrew/bin/curl",
        ],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "curl \\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(exec_cmds = ["curl"])

# ---------------------------------------------------------------------------
# fetch_versions — system detection only
# ---------------------------------------------------------------------------

def fetch_versions(_ctx):
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — system tool, not managed by vx
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/curl"

def get_execute_path(ctx, _version):
    exe = "curl.exe" if ctx.platform.os == "windows" else "curl"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(_ctx, _version):
    return []

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
    "curl",
    published = False,
)
rez_fetch_versions = _REZ_SDK["fetch_versions"]
rez_download_url = _REZ_SDK["download_url"]
rez_install_layout = _REZ_SDK["install_layout"]
rez_bundle = _REZ_SDK["rez_bundle"]