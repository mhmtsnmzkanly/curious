import hashlib
import json
import subprocess
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path

import hunt_lab_completion_dispatch_v2 as dispatch


class HuntLabCompletionDispatchTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.experiments = self.root / "experiments"
        self.experiments.mkdir()
        self.outbox = self.root / "outbox"
        self.original_experiments = dispatch.EXPERIMENTS
        dispatch.EXPERIMENTS = self.experiments
        self.run_name = "hunt-lab-v3-test-run"
        self.output = self.experiments / self.run_name
        self.output.mkdir()
        self.config_path = self.root / "config.json"
        self.config = {
            "schema": dispatch.CONFIG_SCHEMA,
            "helper_sha256": dispatch.helper_hash(),
            "allowed_schema": "curious-hunt-lab-run-v3",
            "protocol_version": 3,
            "artifact_name": "hunt-lab-probe",
            "thread_id": "01a13cca-3f39-7ad1-907f-398e8926cd36",
            "expires_at": (datetime.now(timezone.utc) + timedelta(days=1)).isoformat(),
            "outbox_dir": str(self.outbox),
            "watched_runs": [{"name": self.run_name, "output_dir": str(self.output)}],
        }
        self.save_config()

    def tearDown(self):
        dispatch.EXPERIMENTS = self.original_experiments
        self.temp.cleanup()

    def save_config(self):
        self.config_path.write_text(json.dumps(self.config))

    def make_terminal_manifest(self, status="completed"):
        probe = self.output / "hunt-lab-probe"
        protocol = self.output / "protocol.md"
        probe.write_bytes(b"immutable probe")
        protocol.write_text("registered protocol\n")
        raw = self.output / "raw.jsonl"
        summary = self.output / "summary.json"
        raw.write_text('{"trial":1}\n')
        summary.write_text('{"schema":"summary"}\n')
        manifest = {
            "schema": "curious-hunt-lab-run-v3",
            "protocol_version": 3,
            "revision": "a" * 40,
            "binary_sha256": hashlib.sha256(probe.read_bytes()).hexdigest(),
            "protocol_sha256": hashlib.sha256(protocol.read_bytes()).hexdigest(),
            "status": status,
            "finished_at": datetime.now(timezone.utc).isoformat(),
        }
        if status == "completed":
            manifest["raw_sha256"] = hashlib.sha256(raw.read_bytes()).hexdigest()
            manifest["summary_sha256"] = hashlib.sha256(summary.read_bytes()).hexdigest()
        else:
            raw.unlink()
            summary.unlink()
        (self.output / "manifest.json").write_text(json.dumps(manifest))

    @staticmethod
    def queue_result(queue_id="01a16682-393e-7c42-9a7d-441adcad1f70"):
        return subprocess.CompletedProcess(
            [], 0,
            f"Queued message {queue_id} for thread 01a13cca-3f39-7ad1-907f-398e8926cd36.",
            "",
        )

    def test_nonterminal_or_missing_manifest_does_not_queue(self):
        calls = []
        result = dispatch.run(self.config_path, call=lambda argv, timeout: calls.append(argv))
        self.assertEqual(result["runs"][self.run_name], "waiting")
        self.assertEqual(calls, [])
        self.assertFalse((self.outbox / f"{self.run_name}.json").exists())

        self.make_terminal_manifest()
        manifest = json.loads((self.output / "manifest.json").read_text())
        manifest["status"] = "running"
        (self.output / "manifest.json").write_text(json.dumps(manifest))
        result = dispatch.run(self.config_path, call=lambda argv, timeout: calls.append(argv))
        self.assertEqual(result["runs"][self.run_name], "waiting")
        self.assertEqual(calls, [])

    def test_terminal_manifest_queues_once_and_records_id(self):
        self.make_terminal_manifest()
        calls = []

        def runner(argv, timeout):
            calls.append(argv)
            return self.queue_result()

        first = dispatch.run(self.config_path, call=runner)
        second = dispatch.run(self.config_path, call=runner)
        self.assertEqual(first["runs"][self.run_name], "queued")
        self.assertEqual(second["runs"][self.run_name], "already_queued")
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][:4], [dispatch.CODEX, "queue", "--thread", self.config["thread_id"]])
        state = json.loads((self.outbox / f"{self.run_name}.json").read_text())
        self.assertEqual(state["queue_id"], "01a16682-393e-7c42-9a7d-441adcad1f70")
        self.assertIn(f"completion_token=hunt-lab-v3:{self.run_name}:", calls[0][-1])

    def test_failed_terminal_manifest_requires_finished_at_and_immutable_hashes(self):
        self.make_terminal_manifest("failed")
        calls = []
        result = dispatch.run(self.config_path, call=lambda argv, timeout: (calls.append(argv), self.queue_result())[1])
        self.assertEqual(result["runs"][self.run_name], "queued")
        self.assertEqual(len(calls), 1)

        (self.outbox / f"{self.run_name}.json").unlink()
        manifest_path = self.output / "manifest.json"
        manifest = json.loads(manifest_path.read_text())
        manifest.pop("finished_at")
        manifest_path.write_text(json.dumps(manifest))
        result = dispatch.run(self.config_path, call=lambda argv, timeout: calls.append(argv))
        self.assertEqual(result["runs"][self.run_name], "invalid_terminal_manifest")
        self.assertEqual(len(calls), 1)
        marker = json.loads((self.outbox / f"{self.run_name}.invalid.json").read_text())
        self.assertEqual(marker["state"], "invalid_terminal_manifest")

    def test_completed_manifest_rejects_mismatched_probe_and_protocol_hashes(self):
        self.make_terminal_manifest()
        probe = self.output / "hunt-lab-probe"
        probe.write_bytes(b"changed probe")
        calls = []
        result = dispatch.run(self.config_path, call=lambda argv, timeout: calls.append(argv))
        self.assertEqual(result["runs"][self.run_name], "invalid_terminal_manifest")
        self.assertEqual(calls, [])

    def test_queue_failure_is_durable_notified_and_retryable(self):
        self.make_terminal_manifest()
        calls = []
        notifications = []

        def failing(argv, timeout):
            calls.append(argv)
            if argv[0] == dispatch.CODEX:
                return subprocess.CompletedProcess(argv, 1, "", "queue offline")
            notifications.append(json.loads(Path(argv[5]).read_text()))
            return subprocess.CompletedProcess(argv, 0, '{"success":true}', "")

        first = dispatch.run(self.config_path, call=failing)
        self.assertEqual(first["runs"][self.run_name], "pending")
        state_path = self.outbox / f"{self.run_name}.json"
        state = json.loads(state_path.read_text())
        self.assertEqual(state["state"], "pending")
        self.assertTrue(state["notify_sent"])
        self.assertEqual(len(calls), 2)
        self.assertEqual(calls[1][1], dispatch.NOTIFY)
        payload = notifications[0]
        self.assertEqual(payload["target"], {"user": "u_e049630155fc"})
        self.assertTrue(any(arg.startswith("bounded-lab-queue-error:") for arg in calls[1]))

        def succeeding(argv, timeout):
            calls.append(argv)
            return self.queue_result()

        second = dispatch.run(self.config_path, call=succeeding)
        self.assertEqual(second["runs"][self.run_name], "queued")
        self.assertEqual(len(calls), 3)

    def test_timeout_is_marked_at_least_once_and_notified(self):
        self.make_terminal_manifest()
        calls = []

        def timeout_then_notify(argv, timeout):
            calls.append(argv)
            if argv[0] == dispatch.CODEX:
                raise subprocess.TimeoutExpired(argv, timeout)
            return subprocess.CompletedProcess(argv, 0, '{"success":true}', "")

        dispatch.run(self.config_path, call=timeout_then_notify)
        state = json.loads((self.outbox / f"{self.run_name}.json").read_text())
        self.assertTrue(state["delivery_ambiguous"])
        self.assertTrue(state["at_least_once"])

    def test_expired_config_and_unlisted_paths_do_not_queue(self):
        self.make_terminal_manifest()
        self.config["expires_at"] = (datetime.now(timezone.utc) - timedelta(seconds=1)).isoformat()
        self.save_config()
        calls = []
        result = dispatch.run(self.config_path, call=lambda argv, timeout: calls.append(argv))
        self.assertEqual(result["status"], "expired")
        self.assertEqual(calls, [])

        self.config["expires_at"] = (datetime.now(timezone.utc) + timedelta(days=1)).isoformat()
        self.config["watched_runs"][0]["output_dir"] = str(self.root / "other" / self.run_name)
        self.save_config()
        with self.assertRaises(ValueError):
            dispatch.run(self.config_path, call=lambda argv, timeout: calls.append(argv))
        self.assertEqual(calls, [])

    def test_allowlisted_protocol_names_and_explicit_artifacts(self):
        cases = [
            ("curious-lifetime-lab-run-v1", "lifetime-lab-v1-test", "lab-probe"),
            ("curious-herbivore-lab-run-v2", "herbivore-lab-v2-test", "lab-probe"),
            ("curious-ecology-traits-run-v1", "ecology-traits-v1-test", "lab-probe"),
        ]
        for schema, name, artifact in cases:
            with self.subTest(schema=schema):
                self.config["allowed_schema"] = schema
                self.config["protocol_version"] = int(schema.rsplit("v", 1)[1])
                self.config["artifact_name"] = artifact
                self.config["watched_runs"] = [{"name": name, "output_dir": str(self.experiments / name)}]
                self.save_config()
                watched, _ = dispatch.validate_config(self.config, dispatch.helper_hash())
                self.assertEqual(watched[0]["name"], name)

    def test_schema_run_version_and_artifact_filename_are_strict(self):
        calls = []
        for config_changes in (
            {"allowed_schema": "curious-lifetime-lab-run-v1", "protocol_version": 2},
            {"allowed_schema": "curious-unknown-lab-run-v1", "protocol_version": 1},
            {"artifact_name": "../raw.jsonl"},
            {"artifact_name": None},
        ):
            with self.subTest(config_changes=config_changes):
                original = dict(self.config)
                self.config.update(config_changes)
                self.save_config()
                with self.assertRaises(ValueError):
                    dispatch.validate_config(self.config, dispatch.helper_hash())
                self.config = original
        self.assertEqual(calls, [])

    def test_selected_artifact_file_must_match_manifest_hash(self):
        self.make_terminal_manifest()
        (self.output / "lab-probe").write_bytes(b"alternate artifact")
        self.config["artifact_name"] = "lab-probe"
        self.save_config()
        calls = []
        result = dispatch.run(self.config_path, call=lambda argv, timeout: calls.append(argv))
        self.assertEqual(result["runs"][self.run_name], "invalid_terminal_manifest")
        self.assertEqual(calls, [])

    def test_changed_terminal_identity_is_not_queued_twice(self):
        self.make_terminal_manifest()
        calls = []
        dispatch.run(self.config_path, call=lambda argv, timeout: (calls.append(argv), self.queue_result())[1])
        manifest = json.loads((self.output / "manifest.json").read_text())
        manifest["finished_at"] = (datetime.now(timezone.utc) + timedelta(seconds=1)).isoformat()
        (self.output / "manifest.json").write_text(json.dumps(manifest))
        result = dispatch.run(self.config_path, call=lambda argv, timeout: (calls.append(argv), self.queue_result())[1])
        self.assertEqual(result["runs"][self.run_name], "identity_changed")
        self.assertEqual(len(calls), 1)

    def test_queue_response_must_name_requested_thread(self):
        expected = "01a13cca-3f39-7ad1-907f-398e8926cd36"
        other = "01a13cca-3f39-7ad1-907f-398e8926cd37"
        self.assertEqual(
            dispatch.queue_id_from(
                f"Queued message 01a16682-393e-7c42-9a7d-441adcad1f70 for thread {expected}.", expected
            ),
            "01a16682-393e-7c42-9a7d-441adcad1f70",
        )
        self.assertIsNone(
            dispatch.queue_id_from(
                f"Queued message 01a16682-393e-7c42-9a7d-441adcad1f70 for thread {other}.", expected
            )
        )


if __name__ == "__main__":
    unittest.main()
