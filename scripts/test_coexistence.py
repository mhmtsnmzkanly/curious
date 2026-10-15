import argparse
import contextlib
import io
import json
from pathlib import Path
import tempfile
import unittest

from validate_coexistence import passed, run, summary


def result():
    return {'seed': 1, 'ticks': 100000, 'requested_ticks': 100000, 'mode': 'fixed',
            'first_extinction': None, 'alive': {'Herbivore': 2, 'Carnivore': 2},
            'late_births': {'Herbivore': 3, 'Carnivore': 3},
            'late_descendant_breeders': {'Herbivore': 2, 'Carnivore': 2}}


class CoexistenceValidationTests(unittest.TestCase):
    def test_survivors_without_replacement_and_early_termination_do_not_pass(self):
        row = result()
        self.assertTrue(passed(row, 100000))
        row['late_descendant_breeders']['Carnivore'] = 0
        self.assertFalse(passed(row, 100000))
        row = result()
        row['ticks'] = 800
        self.assertFalse(passed(row, 100000))
        row = result()
        row['first_extinction'] = 500
        self.assertFalse(passed(row, 100000))

    def test_short_smoke_test_is_not_formal_qualification(self):
        manifest = {'seeds': [1], 'ticks': 100000, 'dirty': False}
        s = summary([result()], manifest)
        self.assertTrue(s['accepted'])
        self.assertFalse(s['qualified'])
        manifest['seeds'] = list(range(30))
        self.assertFalse(summary([result()], manifest)['complete'])
        self.assertFalse(summary([result()], manifest)['qualified'])
        rows = [{**result(), 'seed': s, 'first_extinction': None if s < 27 else 1} for s in range(30)]
        self.assertTrue(summary(rows, manifest)['qualified'])
        with self.assertRaises(ValueError):
            summary([result(), result()], manifest)

    def test_resume_keeps_frozen_artifacts_and_rejects_binary_drift(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scenario = root / 'scenario.json'
            scenario.write_text('{}')
            probe = root / 'fake-probe'
            probe.write_text('#!/usr/bin/env python3\nimport json,sys\nr=' + repr(result()) + '\nr["seed"]=int(sys.argv[3])\nprint(json.dumps(r))\n')
            probe.chmod(0o700)
            args = argparse.Namespace(probe=str(probe), scenario=str(scenario), output=str(root / 'run'),
                                      seeds='1', ticks=100000, mode='fixed', timeout=10,
                                      no_companions=False, allow_dirty=True, resume=False, notify=False)
            args.shock_at, args.shock_duration, args.shock_percent, args.jobs = 0, 500, 50, 2
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertTrue(run(args)['accepted'])
                saved = root / 'run' / 'seed-1.json'
                before = saved.stat().st_mtime_ns
                args.resume = True
                self.assertTrue(run(args)['accepted'])
            self.assertEqual(saved.stat().st_mtime_ns, before)
            probe.write_text(probe.read_text() + '# changed\n')
            with self.assertRaisesRegex(ValueError, 'Refusing resume'):
                run(args)


if __name__ == '__main__':
    unittest.main()
