load("@vx//stdlib:rez.star", "rez_bundle_source")

# provider.star - Meson provider
#
# Meson is a Python-based build system distributed via PyPI.
# Executed via `uvx meson` — each version runs in its own isolated Python env.
# vx meson  ==  vx uvx:meson  (RFC 0033 package_alias routing)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star", "runtime_def", "dep_def", "system_permissions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name          = "meson"
description   = "Meson - An extremely fast and user friendly build system"
homepage      = "https://mesonbuild.com"
repository    = "https://github.com/mesonbuild/meson"
license       = "Apache-2.0"
ecosystem     = "python"
aliases       = ["mesonbuild"]

# RFC 0033: route `vx meson` → `vx uvx:meson`
package_alias = {"ecosystem": "uvx", "package": "meson"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("meson",
        aliases         = ["mesonbuild"],
        version_pattern = "^\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(
    extra_hosts = ["pypi.org"],
    exec_cmds   = ["uvx", "uv"],
)

# ---------------------------------------------------------------------------
# download_url — not applicable (runs via uvx)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/meson"

def get_execute_path(_ctx, _version):
    return None

def post_install(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# deps — requires uv
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("uv", reason = "Meson is installed and run via uv pip"),
    ]
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
    "meson",
    published = False,
)
rez_fetch_versions = _REZ_SDK["fetch_versions"]
rez_download_url = _REZ_SDK["download_url"]
rez_install_layout = _REZ_SDK["install_layout"]
rez_bundle = _REZ_SDK["rez_bundle"]