load("@vx//stdlib:rez.star", "rez_bundle_source")

load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_go_provider")
load("@vx//stdlib:system_install.star", "cross_platform_install")

# Metadata
name = "grype"
description = "Grype - A vulnerability scanner for container images and filesystems"
homepage = "https://github.com/anchore/grype"
repository = "https://github.com/anchore/grype"
license = "Apache-2.0"
ecosystem = "security"

# Runtimes
runtimes = [
    runtime_def("grype"),
]

# Permissions
permissions = github_permissions()

# Use github_go_provider template
# grype asset naming: grype_0.111.1_linux_amd64.tar.gz  (underscore, NO v prefix in asset)
_p = github_go_provider("anchore", "grype",
    asset      = "grype_{version}_{os}_{arch}.{ext}",
    executable = "grype",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]

# system_install fallback when GitHub download is unavailable
system_install = cross_platform_install(
    windows = "Grype",
    macos   = "grype",
    linux   = "grype",
)
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
    "grype",
    published = False,
)
rez_fetch_versions = _REZ_SDK["fetch_versions"]
rez_download_url = _REZ_SDK["download_url"]
rez_install_layout = _REZ_SDK["install_layout"]
rez_bundle = _REZ_SDK["rez_bundle"]