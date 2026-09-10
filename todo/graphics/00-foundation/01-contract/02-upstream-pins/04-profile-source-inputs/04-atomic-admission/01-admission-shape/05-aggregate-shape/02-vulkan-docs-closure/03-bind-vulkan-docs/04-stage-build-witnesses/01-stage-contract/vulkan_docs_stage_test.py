"""Live pinned-prerequisite and plan-integrity tests for actual Docs staging."""

from __future__ import annotations

import copy
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import vulkan_docs_stage_fs as FS
from vulkan_docs_stage_bind import PROJECT_ROOT, build_plan, plan_value, verify_value
from vulkan_docs_stage_contract import verify_plan
from vulkan_docs_stage_model import RUN_IDS, StageError
from vulkan_docs_stage_parse import canonical

HERE = Path(__file__).resolve().parent
BIND = HERE.parent.parent
OBSERVATION = BIND / "03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json"
ARTIFACTS = PROJECT_ROOT / ".artifacts/graphics/f02.4.4.1.5.2.3.3.1"


def reseal(value: dict[str, object]) -> None:
    payload = dict(value)
    payload.pop("plan_sha256", None)
    value["plan_sha256"] = canonical(payload, "webboxvm-graphics-vulkan-docs-build-staging-plan-v1")


class StagingPlanTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temporary = tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-stage-plan-")
        cls.root = Path(cls.temporary.name).resolve()
        cls.plan = build_plan(OBSERVATION, ARTIFACTS, cls.root)

    @classmethod
    def tearDownClass(cls) -> None:
        cls.temporary.cleanup()

    def test_live_plan_binds_both_recorded_captures_without_payload_staging(self) -> None:
        value = self.plan.value
        self.assertEqual((self.plan.state, self.plan.admitted, self.plan.cutover_ready), ("staging-only-unadmitted", False, False))
        self.assertEqual((len(value["inputs"]["raw_records"]), len(value["inputs"]["derived_records"])), (298, 1462))
        self.assertEqual(tuple(item["run_id"] for item in value["captures"]), RUN_IDS)
        self.assertEqual(value["witness"]["build"]["image"], "khronosgroup/docker-images@sha256:f1ca671f3bdb10ad49e238b9bf28853088a21af49504498fc9084c9b4fea4762")
        self.assertEqual(value["layout"]["outputs"], {run_id: f"outputs/{run_id}" for run_id in RUN_IDS})

    def test_self_sealed_active_or_reordered_plan_is_rejected(self) -> None:
        active = copy.deepcopy(self.plan.value)
        active["status"] = "admitted"
        reseal(active)
        with self.assertRaisesRegex(StageError, "admitted or active"):
            plan_value(active, self.root)
        reordered = copy.deepcopy(self.plan.value)
        reordered["captures"].reverse()
        reseal(reordered)
        with self.assertRaisesRegex(StageError, "reorders or replaces"):
            plan_value(reordered, self.root)

    def test_resealed_build_mutation_cannot_replace_live_pinned_context(self) -> None:
        changed = copy.deepcopy(self.plan.value)
        changed["witness"]["build"]["image"] = "example.invalid@sha256:" + "a" * 64
        reseal(changed)
        with self.assertRaisesRegex(StageError, "selected pinned prerequisites"):
            verify_value(changed, self.plan)

    def test_changed_raw_witness_is_validated_before_its_build_context_is_embedded(self) -> None:
        changed = copy.deepcopy(self.plan.value["witness"])
        changed["build"]["image"] = "example.invalid@sha256:" + "a" * 64
        with patch("vulkan_docs_stage_bind._read_witness", return_value=changed):
            with self.assertRaisesRegex(StageError, "prerequisite binding"):
                build_plan(OBSERVATION, ARTIFACTS, self.root)

    def test_public_plan_verifier_requires_exact_live_plan(self) -> None:
        self.assertEqual(verify_plan(self.plan.value, OBSERVATION, ARTIFACTS, self.root).value, self.plan.value)

    def test_public_writes_require_an_explicit_bounded_payload(self) -> None:
        for payload, maximum in ((b"x", None), (b"xx", 1), ("x", 1)):
            with self.subTest(payload=payload, maximum=maximum), self.assertRaisesRegex(StageError, "payload"):
                FS.atomic_file(self.plan, "limits/member", payload, "test member", maximum_bytes=maximum)
        self.assertEqual(FS.atomic_file(self.plan, "limits/member", b"x", "test member", maximum_bytes=1).read_bytes(), b"x")


if __name__ == "__main__":
    unittest.main()
