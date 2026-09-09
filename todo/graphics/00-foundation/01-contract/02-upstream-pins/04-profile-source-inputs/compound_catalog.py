"""Reviewed exact components and configurations for compound selector audits."""

REVISION = "067e8832315e79817ede1c4863804e440f5d1c80"
ROOT = "external/openglcts/data/gl_cts/data/mustpass/gles/khronos_mustpass/main"
RAW = f"https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/{REVISION}/{ROOT}/"
PROVENANCE = f"https://github.com/KhronosGroup/VK-GL-CTS/tree/{REVISION}"
LICENSE = "Apache-2.0 (VK-GL-CTS repository LICENSE)"
NORMAL = "--deqp-screen-rotation=unspecified --deqp-surface-width=64 --deqp-surface-height=64 --deqp-base-seed=1 --deqp-watchdog=disable"
WIDE = "--deqp-screen-rotation=unspecified --deqp-surface-width=113 --deqp-surface-height=47 --deqp-base-seed=2 --deqp-watchdog=disable"
FBO_WIDTH = "--deqp-screen-rotation=unspecified --deqp-surface-width=64 --deqp-surface-height=-1 --deqp-base-seed=3 --deqp-gl-config-name=rgba8888d24s8 --deqp-surface-type=fbo --deqp-watchdog=disable"
FBO_HEIGHT = "--deqp-screen-rotation=unspecified --deqp-surface-width=-1 --deqp-surface-height=64 --deqp-base-seed=3 --deqp-gl-config-name=rgba8888d24s8 --deqp-surface-type=fbo --deqp-watchdog=disable"


def item(name: str, digest: str, size: int, cases: int, reason: str) -> dict[str, object]:
    identifier = f"gles-cts-{name.removesuffix('.txt')}"
    role = f"GLES CTS {reason} selector component; generated status not asserted"
    return {
        "id": identifier, "source_family": "gles-cts-component", "immutable_url": f"{RAW}{name}",
        "revision": REVISION, "sha256": digest, "bytes": size, "license": LICENSE,
        "local_cache": f"webboxvm-graphics/f02/{identifier}/{digest}.source", "generated_code_role": role,
        "provenance": PROVENANCE, "selector": f"{ROOT}/{name}", "case_count": cases, "reason": reason,
    }


def config(name: str, command: str, first: bool, reason: str) -> dict[str, object]:
    return {"selector": f"{ROOT}/{name}", "command_line": command,
            "name": "khr-glesext" if reason == "optional-extension" else "khr-main", "os": "any",
            "use_for_first_egl_config": first, "reason": reason}


CORE = (
    item("gles2-khr-main.txt", "aed17d34d64047973c439835ee7cfa8a109c2159512fe1fe59fb07fc05d395f2", 38127, 473, "core"),
    item("gles3-khr-main.txt", "c79097f9f9c69a4556e235b3301c0dc9a5d73375c05f193a6f5e127d853ace72", 443126, 6498, "core"),
    item("gles31-khr-main.txt", "9fd8e4ff51616262c567f7a9eec83f69d2d311a4293a45d2d377306d426d520e", 288777, 4101, "core"),
    item("gles32-khr-main.txt", "426fa31d557e08e214b75fda5d9efcaec54a42cdc14d4915939a7d02ccb6f249", 105294, 1405, "core"),
)
OPTIONAL = (item("gles32-khr-glesext.txt", "789b0474c61693baaa7cbc941575f23b383e2ba3c4991b1a910e046a17a6e0e6", 81374, 1097, "optional-extension"),)
CONFIGURATIONS = (
    config("gles2-khr-main.txt", NORMAL, True, "core"), config("gles2-khr-main.txt", NORMAL, False, "core"),
    config("gles3-khr-main.txt", NORMAL, True, "core"), config("gles3-khr-main.txt", NORMAL, False, "core"),
    config("gles31-khr-main.txt", NORMAL, True, "core"), config("gles31-khr-main.txt", NORMAL, False, "core"),
    config("gles32-khr-main.txt", NORMAL, True, "core"), config("gles32-khr-main.txt", WIDE, True, "core"),
    config("gles32-khr-main.txt", FBO_WIDTH, True, "core"), config("gles32-khr-main.txt", FBO_HEIGHT, True, "core"),
    config("gles32-khr-main.txt", NORMAL, False, "core"), config("gles32-khr-main.txt", WIDE, False, "core"),
)
OPTIONAL_CONFIGURATIONS = (config("gles32-khr-glesext.txt", NORMAL, True, "optional-extension"),)

CATALOG = {"gles-3.2": {"candidate_id": "gles-cts-manifest",
           "candidate_sha256": "9f466f19a26120149bab06e70596c9fff57b61d767cfdbadc7ba13934ce2ea96",
           "included": CORE, "excluded": OPTIONAL, "configurations": CONFIGURATIONS,
           "excluded_configurations": OPTIONAL_CONFIGURATIONS}}
