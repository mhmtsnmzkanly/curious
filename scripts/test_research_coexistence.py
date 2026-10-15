import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import research_coexistence as research


class ResearchGateTests(unittest.TestCase):
    def campaign(self, root):
        scenario = root / 'scenario.json'
        scenario.write_text(json.dumps({'founders': [{'species': 'Carnivore', 'life': {'max_energy': 270, 'energy': 270, 'reproduction_interval': 350}}]}))
        probe = root / 'probe'
        probe.write_text('fixture, not an engine')
        return {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
                'output': str(root / 'run'), 'scenario': str(scenario), 'scenario_sha256': research.digest(scenario),
                'probe': str(probe), 'probe_sha256': research.digest(probe), 'deploy_after_pass': True}

    def fake_process(self, rejected, seen):
        class Process:
            def __init__(self, command, **kwargs):
                self.returncode = None
                output = Path(command[command.index('--output') + 1])
                output.mkdir(parents=True, exist_ok=True)
                name = output.name
                seen.append(name)
                ticks = int(command[command.index('--ticks') + 1])
                seeds = command[command.index('--seeds') + 1].split(',')
                (output / 'summary.json').write_text(json.dumps({'accepted': name != rejected,
                    'qualified': ticks >= 100000 and len(seeds) >= 30 and name != rejected,
                    'passed': len(seeds) if name != rejected else 0, 'planned': len(seeds), 'required': len(seeds)}))
            def wait(self, timeout=None):
                self.returncode = 0
                return 0
            def poll(self):
                return self.returncode
        return Process

    def test_biological_rejection_stops_before_holdouts_or_deployment(self):
        with tempfile.TemporaryDirectory() as temporary:
            config = self.campaign(Path(temporary))
            seen = []
            with patch.object(research.subprocess, 'check_output', return_value=config['revision']), patch.object(research.subprocess, 'Popen', self.fake_process('canary-fixed', seen)), patch.object(research, 'notices'), patch.object(research, 'live_deploy') as deploy:
                research.run(config)
            self.assertEqual(seen, ['canary-fixed'])
            deploy.assert_not_called()
            self.assertEqual(json.loads((Path(config['output']) / 'state.json').read_text())['status'], 'rejected')

    def test_live_world_requires_every_gate_including_long_learning_run(self):
        with tempfile.TemporaryDirectory() as temporary:
            config = self.campaign(Path(temporary))
            seen = []
            with patch.object(research.subprocess, 'check_output', return_value=config['revision']), patch.object(research.subprocess, 'Popen', self.fake_process(None, seen)), patch.object(research, 'notices'), patch.object(research, 'live_deploy', return_value={'tick': 1}) as deploy:
                research.run(config)
            self.assertEqual(seen[-1], 'qualification-both')
            self.assertEqual(len(seen), 11)
            deploy.assert_called_once()
            self.assertEqual(json.loads((Path(config['output']) / 'state.json').read_text())['status'], 'observing')

    def test_transport_failure_preserves_one_exact_notification_for_retry(self):
        with tempfile.TemporaryDirectory() as temporary:
            config = self.campaign(Path(temporary))
            with patch.object(research, 'dispatch'):
                research.notices(config, 'milestone', 'title', 'body')
                first = json.loads((Path(config['output']) / 'notifications.json').read_text())
                research.notices(config, 'milestone', 'title', 'body')
                again = json.loads((Path(config['output']) / 'notifications.json').read_text())
            self.assertEqual(len(again['pending']), 1)
            self.assertEqual(first['pending'], again['pending'])

    def test_predeclared_screening_failure_does_not_consume_holdouts(self):
        with tempfile.TemporaryDirectory() as temporary:
            config = self.campaign(Path(temporary))
            config['candidates'] = [{'name': 'a', 'scenario': config['scenario'], 'sha256': config['scenario_sha256']}]
            seen = []
            with patch.object(research.subprocess, 'check_output', return_value=config['revision']), patch.object(research.subprocess, 'Popen', self.fake_process('screen-a', seen)), patch.object(research, 'notices'), patch.object(research, 'live_deploy') as deploy:
                research.run(config)
            self.assertEqual(seen, ['screen-a'])
            deploy.assert_not_called()
            self.assertEqual(json.loads((Path(config['output']) / 'state.json').read_text())['stage'], 'calibration-exhausted')

    def test_next_candidate_can_be_screened_before_the_single_holdout_test(self):
        with tempfile.TemporaryDirectory() as temporary:
            config = self.campaign(Path(temporary))
            config['candidates'] = [{'name': n, 'scenario': config['scenario'], 'sha256': config['scenario_sha256']} for n in ['a', 'b']]
            seen = []
            with patch.object(research.subprocess, 'check_output', return_value=config['revision']), patch.object(research.subprocess, 'Popen', self.fake_process('screen-a', seen)), patch.object(research, 'notices'), patch.object(research, 'live_deploy', return_value={'tick': 1}):
                research.run(config)
            self.assertEqual(seen[:3], ['screen-a', 'screen-b', 'canary-b'])
            self.assertEqual(seen.count('qualification-fixed'), 1)

    def test_report_timer_does_not_rewrite_an_unchanged_page(self):
        with tempfile.TemporaryDirectory() as temporary:
            config = self.campaign(Path(temporary))
            root = Path(config['output'])
            root.mkdir()
            manifest = root / 'manifest.json'
            manifest.write_text(json.dumps({'version': 1, 'blocks': []}))
            (root / 'state.json').write_text(json.dumps({'status': 'running'}))
            config.update(page_id='synthetic-page', page_manifest=str(manifest))
            with patch.object(research.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0)), patch.object(research, 'dispatch'):
                research.notices(config, 'test', 'title', 'body')
            with patch.object(research.subprocess, 'run') as transport, patch.object(research, 'dispatch'):
                research.notices(config)
            transport.assert_not_called()


if __name__ == '__main__':
    unittest.main()
