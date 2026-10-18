import argparse
import json
from pathlib import Path
import tempfile
import unittest

from research_protocol import policy_flags, protocol, freeze, reused_stage, digest, TRAINING_SEEDS
from validate_coexistence import summary, validate_treatment


class ProtocolTests(unittest.TestCase):
    def fixture(self, root):
        root.mkdir()
        (root/'probe').write_text('immutable executable fixture')
        (root/'scenario.json').write_text('{}')
        config = {'revision': 'runner', 'engine_revision': 'engine',
                  'probe_sha256': digest(root/'probe'), 'scenario_sha256': digest(root/'scenario.json'),
                  'reserve_opportunities': True, 'mate_rendezvous': True}
        manifest = {'revision': 'engine', 'binary_sha256': config['probe_sha256'],
                    'scenario_sha256': config['scenario_sha256'], 'dirty': False,
                    'seeds': TRAINING_SEEDS, 'ticks': 20000, 'mode': 'fixed', 'companions': True,
                    'reserve_opportunities': True, 'mate_rendezvous': True,
                    'shock': {'at': 0, 'duration': 500, 'percent': 50}}
        rows = []
        for seed in TRAINING_SEEDS:
            row = {'seed': seed, 'requested_ticks': 20000, 'ticks': 20000, 'mode': 'fixed',
                   'first_extinction': None, 'alive': {'Herbivore': 20, 'Carnivore': 20},
                   'late_births': {'Herbivore': 4, 'Carnivore': 4},
                   'late_descendant_breeders': {'Herbivore': 3, 'Carnivore': 3},
                   'reserve_opportunities': True, 'mate_rendezvous': True,
                   'companions': True, 'shock': manifest['shock']}
            (root/f'seed-{seed}.json').write_text(json.dumps(row)); rows.append(row)
        (root/'manifest.json').write_text(json.dumps(manifest))
        (root/'summary.json').write_text(json.dumps(summary(rows, manifest)))
        config['prevalidated'] = {'canary-fixed': {'path': str(root),
            **{f+'_sha256': digest(root/f) for f in ['manifest.json', 'summary.json']}}}
        return config

    def reuse(self, config, root):
        return reused_stage(config, 'canary-fixed', root/'scenario.json', TRAINING_SEEDS, 20000, 'fixed', False)

    def test_training_reuse_is_read_only_and_requires_exact_treatment(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)/'evidence'; config = self.fixture(root)
            before = {p.name: (p.read_bytes(), p.stat().st_mtime_ns) for p in root.iterdir()}
            result, evidence = self.reuse(config, root)
            self.assertTrue(result['accepted']); self.assertEqual(len(evidence['seed_sha256']), 8)
            self.assertEqual(before, {p.name: (p.read_bytes(), p.stat().st_mtime_ns) for p in root.iterdir()})
            config['mate_rendezvous'] = False
            with self.assertRaisesRegex(ValueError, 'treatment'): self.reuse(config, root)

    def test_raw_failures_cannot_hide_behind_an_accepted_summary(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)/'evidence'; config = self.fixture(root)
            p = root/'seed-1.json'; row = json.loads(p.read_text())
            row['late_descendant_breeders']['Carnivore'] = 0; p.write_text(json.dumps(row))
            with self.assertRaisesRegex(ValueError, 'summary'): self.reuse(config, root)

    def test_changed_artifacts_or_summary_are_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)/'evidence'; config = self.fixture(root)
            (root/'probe').write_text('changed executable')
            with self.assertRaisesRegex(ValueError, 'artifacts'): self.reuse(config, root)
            (root/'summary.json').write_text('{}')
            with self.assertRaisesRegex(ValueError, 'evidence changed'): self.reuse(config, root)

    def test_declared_holdouts_and_policies_cannot_change_on_resume(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)/'protocol.json'; config = {'held_out_seeds': list(range(20001, 20031))}
            freeze(p, protocol(config)); freeze(p, protocol(config))
            config['reserve_opportunities'] = True
            with self.assertRaisesRegex(ValueError, 'protocol changed'): freeze(p, protocol(config))
        for seeds in [[1]+list(range(20002,20031)), [20001]*30, list(range(20))]:
            with self.assertRaises(ValueError): protocol({'held_out_seeds': seeds})
        with self.assertRaises(ValueError): policy_flags({'mate_rendezvous': 'false'})

    def test_probe_cannot_ignore_requested_treatment_in_new_or_cached_results(self):
        args = argparse.Namespace(ticks=20000, mode='fixed', reserve_opportunities=True,
                                  mate_rendezvous=True, no_companions=False,
                                  shock_at=10000, shock_duration=1000, shock_percent=50)
        row = {'seed': 3, 'requested_ticks': 20000, 'mode': 'fixed', 'reserve_opportunities': True,
               'mate_rendezvous': True, 'companions': True,
               'shock': {'at': 10000, 'duration': 1000, 'percent': 50}}
        validate_treatment(row,args,3)
        for key, value in [('reserve_opportunities',False),('mate_rendezvous',False),('companions',False),('shock',{})]:
            with self.assertRaises(ValueError): validate_treatment({**row,key:value},args,3)
