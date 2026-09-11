#!/usr/bin/env python3
"""Hostile structural coverage for the F02.5 source-role catalog."""

import copy
import unittest

from source_role_contract import MAX_LOCAL_BYTES, RoleError, validate_catalog

REVISION = "a" * 40
DIGEST = "b" * 64
BUILDER_DIGEST = "c" * 64


def claims(selector=False, **changes):
    value = {"khronos_selector": selector, "api_support": False, "conformance": False,
             "certification": False, "profile_support": False, "performance": False}
    value.update(changes)
    return value


def raw(repo, path):
    return f"https://raw.githubusercontent.com/KhronosGroup/{repo}/{REVISION}/{path}"


def builder():
    return {"id": "webboxvm-source-builder", "sha256": BUILDER_DIGEST, "artifact": "tools/source-builder"}


def ref(identifier, kind="upstream-source", **changes):
    value = {"id": identifier, "kind": kind, "sha256": DIGEST}
    if kind != "webboxvm-transform":
        value["revision"] = REVISION
    value.update(changes)
    return value


def command(identifier, scope, inputs):
    return ["webboxvm-source-builder", f"--mode={scope.removeprefix('webboxvm-')}",
            *(f"@input:{item['id']}" for item in inputs), f"@output:{identifier}"]


def upstream(identifier="vulkan-spec", **changes):
    value = {"kind": "upstream-source", "id": identifier, "sha256": DIGEST, "bytes": 42,
             "license": "CC-BY-4.0", "attribution": "Khronos", "scope": "normative-source",
             "authority": "Khronos", "producer": "Khronos", "claims": claims(),
             "immutable_url": raw("Vulkan-Docs", "vkspec.adoc"), "revision": REVISION,
             "artifact": f"objects/{identifier}.bin"}
    value.update(changes)
    return value


def suite(path="external/vulkancts/mustpass/main/vk-default.txt", **changes):
    value = {"kind": "full-suite-root", "id": "vulkan-cts-mustpass", "sha256": DIGEST, "bytes": 42,
             "license": "Apache-2.0", "attribution": "Khronos", "scope": "full-conformance-suite",
             "authority": "Khronos", "producer": "Khronos", "claims": claims(True),
             "immutable_url": raw("VK-GL-CTS", path), "revision": REVISION, "artifact": "objects/root.bin",
             "profile": "vulkan-1.4-core", "suite_id": "vulkan-cts-default", "selector_path": path,
             "unfiltered": True}
    value.update(changes)
    return value


def suite_member(**changes):
    path = "external/vulkancts/mustpass/main/member.txt"
    value = upstream("member", scope="suite-member", immutable_url=raw("VK-GL-CTS", path),
                     artifact="objects/member.bin", suite_root_id="vulkan-cts-mustpass", member_path=path)
    value.update(changes)
    return value


def transform(identifier="vk14-map", inputs=None, **changes):
    inputs = inputs if inputs is not None else [ref("vulkan-spec")]
    scope = changes.get("scope", "webboxvm-core-definition")
    value = {"kind": "webboxvm-transform", "id": identifier, "sha256": DIGEST, "bytes": 42,
             "license": "CC-BY-4.0", "attribution": "Khronos", "scope": scope,
             "authority": "WebBoxVM", "producer": "WebBoxVM", "claims": claims(), "inputs": inputs,
             "command": command(identifier, scope, inputs), "artifact": f"objects/{identifier}.bin",
             "builder": builder()}
    value.update(changes)
    return value


def catalog(*records):
    return {"schema": 1, "records": list(records)}


class SourceRoleContractTests(unittest.TestCase):
    def test_roles_accept_pins_above_the_local_size_limit(self):
        self.assertEqual(validate_catalog(catalog(upstream(bytes=MAX_LOCAL_BYTES + 1), transform(),
                                                  suite(bytes=MAX_LOCAL_BYTES + 1))),
                         ("vulkan-spec", "vk14-map", "vulkan-cts-mustpass"))

    def test_catalog_requires_unique_records_and_exact_schema(self):
        for value in (catalog(upstream(), upstream()), {"schema": 1, "records": [], "extra": 1}):
            with self.subTest(value=value), self.assertRaises(RoleError):
                validate_catalog(value)

    def test_raw_identity_reuses_the_strict_fetch_url_rule(self):
        foreign = raw("Vulkan-Docs", "vkspec.adoc").replace("KhronosGroup", "Elsewhere")
        for value in (upstream(revision="main"), upstream(immutable_url=foreign),
                      upstream(immutable_url=raw("Vulkan-Docs", "vkspec.adoc") + "?mutable=1")):
            with self.subTest(value=value), self.assertRaises(RoleError):
                validate_catalog(catalog(value))

    def test_all_non_selector_claims_are_explicitly_false(self):
        cases = (upstream(claims=claims(api_support=True)), transform(claims=claims(performance=True)),
                 suite(claims=claims(True, conformance=True)), upstream(claims=claims(True)))
        for value in cases:
            records = (value, transform()) if value["kind"] == "upstream-source" else (upstream(), value)
            with self.subTest(value=value), self.assertRaises(RoleError):
                validate_catalog(catalog(*records))

    def test_registry_metadata_stays_an_upstream_nonselector(self):
        registry = upstream(scope="registry-metadata", immutable_url=raw("Vulkan-Docs", "xml/vk.xml"))
        self.assertEqual(validate_catalog(catalog(registry)),
                         ("vulkan-spec",))
        with self.assertRaises(RoleError):
            validate_catalog(catalog(upstream(scope="registry-metadata", claims=claims(True),
                                              immutable_url=raw("Vulkan-Docs", "xml/vk.xml"))))
        for value in (upstream(scope="registry-metadata"),
                      upstream(immutable_url=raw("Vulkan-Docs", "xml/vk.xml"))):
            with self.subTest(value=value), self.assertRaises(RoleError):
                validate_catalog(catalog(value))

    def test_transform_requires_local_producer_canonical_argv_inputs_and_size_cap(self):
        cases = (transform(authority="Khronos"), transform(command=["sh", "-c", "curl https://bad"]),
                 transform(command=["webboxvm-source-builder", "--mode=core-definition", "@output:vk14-map"]),
                 transform(inputs=[]), transform(bytes=MAX_LOCAL_BYTES + 1))
        for value in cases:
            with self.subTest(value=value), self.assertRaises(RoleError):
                validate_catalog(catalog(upstream(), value))

    def test_transforms_resolve_exact_inputs_and_reject_cycles(self):
        bad_digest, missing = transform(inputs=[ref("vulkan-spec", sha256="d" * 64)]), transform(inputs=[ref("absent")])
        first = transform("first", inputs=[ref("second", "webboxvm-transform")])
        second = transform("second", inputs=[ref("first", "webboxvm-transform")])
        for value in (catalog(upstream(), bad_digest), catalog(upstream(), missing), catalog(first, second)):
            with self.subTest(value=value), self.assertRaises(RoleError):
                validate_catalog(value)

    def test_full_suite_requires_its_canonical_unfiltered_selector(self):
        docs = suite(path="vkspec.adoc", immutable_url=raw("Vulkan-Docs", "vkspec.adoc"), selector_path="vkspec.adoc")
        fraction = suite(path="external/vulkancts/mustpass/main/vk-fraction-mandatory-tests.txt")
        for value in (docs, fraction, suite(unfiltered=False)):
            with self.subTest(value=value), self.assertRaises(RoleError):
                validate_catalog(catalog(value))

    def test_shards_need_a_pinned_suite_member_and_complete_metadata(self):
        root, member = suite(), suite_member(bytes=10)
        metadata = {"source_id": "member", "source_sha256": DIGEST, "source_bytes": 10,
                    "count": 2, "offset": 0, "reassembled_sha256": DIGEST}
        first = transform("part-0", scope="webboxvm-byte-preserving-shard", bytes=5, inputs=[ref("member")],
                          shard=metadata | {"index": 0})
        second = transform("part-1", scope="webboxvm-byte-preserving-shard", bytes=5, inputs=[ref("member")],
                           shard=metadata | {"index": 1, "offset": 5})
        self.assertEqual(validate_catalog(catalog(root, member, first, second)), ("vulkan-cts-mustpass", "member", "part-0", "part-1"))
        for value in (catalog(member, first), catalog(root, member, first, copy.deepcopy(second))):
            if len(value["records"]) == 4:
                value["records"][3]["shard"]["offset"] = 4
            with self.subTest(value=value), self.assertRaises(RoleError):
                validate_catalog(value)


if __name__ == "__main__":
    unittest.main()
