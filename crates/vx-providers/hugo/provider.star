load("@vx//stdlib:rez.star", "rez_bundle_source")

# provider.star - hugo provider (github_smart_provider)
#
# Hugo is a fast and flexible static site generator (written in Go).
# Smart detection auto-selects the best GitHub release asset per platform.
# macOS: .pkg is hard-excluded by smart_detect → falls through to brew.

load("@vx//stdlib:provider_templates.star", "github_smart_provider")
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions",
     "system_install_strategies", "winget_install", "brew_install", "apt_install")

name        = "hugo"
description = "Hugo - Fast and flexible static site generator"
homepage    = "https://gohugo.io"
repository  = "https://github.com/gohugoio/hugo"
license     = "Apache-2.0"
ecosystem   = "devtools"

runtimes = [runtime_def("hugo",
    version_cmd     = "{executable} version",
    version_pattern = "hugo v\\d+\\.\\d+\\.\\d+")]

permissions = github_permissions()

_p = github_smart_provider("gohugoio", "hugo")
fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]

system_install = system_install_strategies([
    winget_install("Hugo.Hugo.Extended", priority = 90),
    brew_install("hugo",                 priority = 90),
    apt_install("hugo",                  priority = 70),
])
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
    "hugo",
    published = False,
)
rez_fetch_versions = _REZ_SDK["fetch_versions"]
rez_download_url = _REZ_SDK["download_url"]
rez_install_layout = _REZ_SDK["install_layout"]
rez_bundle = _REZ_SDK["rez_bundle"]