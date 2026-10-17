import argparse
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from check_reserve_policy import run


class ReserveGateTests(unittest.TestCase):
    def trial(self, outcomes, focus=None, mate=False):
        self.calls = []
        case = self
        class Child:
            def __init__(self, command, **kwargs):
                stage = Path(command[command.index('--output') + 1])
                case.calls.append(stage.name)
                case.assertIn('--reserve-opportunities', command)
                case.assertEqual('--mate-rendezvous' in command, mate)
                case.assertEqual('--shock-at' in command, stage.name.endswith('shock'))
                stage.mkdir(parents=True, exist_ok=True)
                (stage / 'summary.json').write_text(json.dumps(outcomes[stage.name]))
            def wait(self, timeout): return 0
            def poll(self): return 0
        with tempfile.TemporaryDirectory() as directory:
            args = argparse.Namespace(probe='frozen-probe', scenario='frozen-scenario', output=directory, focus_seeds=focus, mate_rendezvous=mate)
            with patch('check_reserve_policy.subprocess.Popen', Child):
                try: run(args)
                finally:
                    p = Path(directory) / 'state.json'
                    self.state = json.loads(p.read_text()) if p.exists() else {}

    def test_failed_normal_never_starts_shock(self):
        self.trial({'normal': {'complete': True, 'accepted': False}})
        self.assertEqual(self.calls, ['normal'])
        self.assertEqual(self.state['status'], 'rejected')

    def test_training_pass_opens_only_the_shock_gate(self):
        self.trial({s: {'complete': True, 'accepted': True} for s in ['normal', 'shock']})
        self.assertEqual(self.calls, ['normal', 'shock'])
        self.assertEqual(self.state['status'], 'passed-training-comparison')

    def test_rendezvous_setting_reaches_both_gated_stages(self):
        self.trial({s: {'complete': True, 'accepted': True} for s in ['normal', 'shock']}, mate=True)
        self.assertEqual(self.calls, ['normal', 'shock'])

    def test_incomplete_results_never_open_next_gate(self):
        with self.assertRaisesRegex(RuntimeError, 'Incomplete'):
            self.trial({'normal': {'complete': False, 'accepted': True}})
        self.assertEqual(self.calls, ['normal'])

    def test_failed_focus_stops_before_the_full_comparison(self):
        self.trial({'focused-shock': {'complete': True, 'accepted': False}}, focus='2,3')
        self.assertEqual(self.calls, ['focused-shock'])
        self.assertEqual(self.state['status'], 'rejected')

    def test_focus_passes_through_the_same_normal_gate(self):
        self.trial({s: {'complete': True, 'accepted': True} for s in ['focused-shock', 'normal', 'shock']}, focus='2,3')
        self.assertEqual(self.calls, ['focused-shock', 'normal', 'shock'])
