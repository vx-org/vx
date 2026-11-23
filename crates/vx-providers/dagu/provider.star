load("@vx//stdlib:rez.star", "rez_bundle_source")

# provider.star - Dagu provider
#
# Dagu is a powerful DAG (Directed Acyclic Graph) workflow engine.
# Asset naming: dagu_{version}_{os}_{arch}.tar.gz  (Go-style, all platforms tar.gz)
#
# Uses github_go_provider template from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "github_go_provider", "runtime_def", "github_permissions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "dagu"
description = "Dagu - A powerful DAG workflow engine with a Web UI"
homepage    = "https://dagu.run"
repository  = "https://github.com/dagu-org/dagu"
license     = "GPL-3.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("dagu",
        version_cmd     = "{executable} version",
        version_pattern = "\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template — github_go_provider
#
# Asset: dagu_{version}_{os}_{arch}.tar.gz
# Tag:   v{version}
# All platforms use tar.gz (no zip on Windows for dagu)
# ---------------------------------------------------------------------------

_p = github_go_provider(
    "dagu-org", "dagu",
    asset = "dagu_{version}_{os}_{arch}.tar.gz",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]
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
    "dagu",
    published = False,
)
rez_fetch_versions = _REZ_SDK["fetch_versions"]
rez_download_url = _REZ_SDK["download_url"]
rez_install_layout = _REZ_SDK["install_layout"]
rez_bundle = _REZ_SDK["rez_bundle"]