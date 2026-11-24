load("@vx//stdlib:rez.star", "rez_bundle_source")

load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_go_provider")
load("@vx//stdlib:system_install.star", "cross_platform_install")

# Metadata
name = "syft"
description = "Syft - CLI tool and library for generating a Software Bill of Materials (SBOM)"
homepage = "https://syft.cli.anchore.io/"
repository = "https://github.com/anchore/syft"
license = "Apache-2.0"
ecosystem = "security"

# Runtimes
runtimes = [
    runtime_def("syft"),
]

# Permissions
permissions = github_permissions()

# Use github_go_provider template with custom asset naming
# syft uses underscores: syft_1.43.0_windows_amd64.zip
_p = github_go_provider("anchore", "syft",
    asset = "syft_{version}_{os}_{arch}.{ext}",
    executable = "syft",
)

fetch_versions = _p["fetch_versions"]
download_url = _p["download_url"]
install_layout = _p["install_layout"]
store_root = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment = _p["environment"]

system_install = cross_platform_install(
    windows = "syft",
    macos   = "syft",
    linux   = "syft",
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
    "syft",
    published = False,
)
rez_fetch_versions = _REZ_SDK["fetch_versions"]
rez_download_url = _REZ_SDK["download_url"]
rez_install_layout = _REZ_SDK["install_layout"]
rez_bundle = _REZ_SDK["rez_bundle"]