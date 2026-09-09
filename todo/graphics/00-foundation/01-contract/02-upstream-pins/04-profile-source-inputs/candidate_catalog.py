"""Reviewed exact candidate identities for completed F02.4 audits."""

CATALOG = {
    "opengl-4.6-core": {
        "opengl-46-core-spec": {
            "source_family": "opengl-46-spec",
            "immutable_url": "https://raw.githubusercontent.com/KhronosGroup/OpenGL-Registry/1cdd228e34966dd6b95bd203e9f84faba0f371a1/specs/gl/glspec46.core.pdf",
            "revision": "1cdd228e34966dd6b95bd203e9f84faba0f371a1",
            "sha256": "a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee",
            "bytes": 3003752,
            "selector": "specs/gl/glspec46.core.pdf",
            "selector_case_count": 0,
            "decision": "accepted",
            "coverage": "complete-single-file",
            "admission_blocker": "",
        },
        "opengl-cts-manifest": {
            "source_family": "opengl-cts",
            "immutable_url": "https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/067e8832315e79817ede1c4863804e440f5d1c80/external/openglcts/data/gl_cts/data/mustpass/gl/khronos_mustpass/4.6.1.x/gl46-main.txt",
            "revision": "067e8832315e79817ede1c4863804e440f5d1c80",
            "sha256": "e28bbbbfd0f6c8d711554a01aa45819bdc7ca963c9426e997dfd1daaeb1d7b17",
            "bytes": 1353085,
            "selector": "external/openglcts/data/gl_cts/data/mustpass/gl/khronos_mustpass/4.6.1.x/gl46-main.txt",
            "selector_case_count": 19714,
            "decision": "accepted",
            "coverage": "complete-single-file",
            "admission_blocker": "",
        },
    },
}
