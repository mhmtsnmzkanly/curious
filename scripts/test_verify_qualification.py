import json
from pathlib import Path
import tempfile
import unittest

from research_protocol import digest, protocol, TRAINING_SEEDS
from validate_coexistence import summary
from verify_qualification import verify


class QualificationProofTests(unittest.TestCase):
    def campaign(self, root):
        write = lambda p, value: p.write_text(json.dumps(value))
        scenario = {'founders': [{'species':'Carnivore','life':{'max_energy':270,'energy':270,'reproduction_interval':350}}]}
        write(root/'scenario.json', scenario); (root/'probe').write_bytes(b'compiled probe fixture')
        c = {'revision':'runner','engine_revision':'engine','probe':str(root/'probe'),
             'probe_sha256':digest(root/'probe'),'scenario':str(root/'scenario.json'),
             'scenario_sha256':digest(root/'scenario.json'),'reserve_opportunities':True,
             'mate_rendezvous':True,'held_out_seeds':list(range(20001,20031)),
             'deploy_after_pass':False,'prevalidated':{}}
        stages = [('canary-fixed',root/'scenario.json',TRAINING_SEEDS,20000,'fixed',False)]
        for field in ['max_energy','reproduction_interval']:
            for percent in [90,110]:
                v=json.loads(json.dumps(scenario)); life=v['founders'][0]['life']; capacity=life['max_energy']
                life[field]=life[field]*percent//100
                if field=='max_energy':life['energy']=life['energy']*life['max_energy']//capacity
                p=root/f'fixed-{field}-{percent}.json';write(p,v)
                stages.append((f'sensitivity-fixed-{field}-{percent}',p,TRAINING_SEEDS[:4],20000,'fixed',False))
        stages += [('qualification-fixed',root/'scenario.json',c['held_out_seeds'],100000,'fixed',False),
                   ('resource-shock',root/'scenario.json',TRAINING_SEEDS,20000,'fixed',True)]
        stages += [('canary-'+mode,root/'scenario.json',TRAINING_SEEDS,20000,mode,False) for mode in ['learning','mutation','both']]
        stages += [('qualification-both',root/'scenario.json',c['held_out_seeds'],100000,'both',False)]
        for name,sc,seeds,ticks,mode,shock in stages:
            p=root/name;p.mkdir();(p/'probe').write_bytes((root/'probe').read_bytes());(p/'scenario.json').write_bytes(sc.read_bytes())
            cached=name in ['canary-fixed','resource-shock']
            m={'revision':'engine' if cached else 'runner','binary_sha256':c['probe_sha256'],
               'scenario_sha256':digest(sc),'dirty':False,'seeds':seeds,'ticks':ticks,'mode':mode,
               'companions':True,'reserve_opportunities':True,'mate_rendezvous':True,
               'shock':{'at':10000 if shock else 0,'duration':1000 if shock else 500,'percent':50}}
            rows=[]
            for seed in seeds:
                row={'seed':seed,'requested_ticks':ticks,'ticks':ticks,'mode':mode,'first_extinction':None,
                     'alive':{'Herbivore':20,'Carnivore':20},'late_births':{'Herbivore':2,'Carnivore':2},
                     'late_descendant_breeders':{'Herbivore':2,'Carnivore':2},
                     'companions':True,'reserve_opportunities':True,'mate_rendezvous':True,'shock':m['shock']}
                rows.append(row);write(p/f'seed-{seed}.json',row)
            write(p/'manifest.json',m);write(p/'summary.json',summary(rows,m))
            if cached:c['prevalidated'][name]={'path':str(p),**{f+'_sha256':digest(p/f) for f in ['manifest.json','summary.json']}}
        write(root/'config.json',c);write(root/'protocol.json',protocol(c))
        write(root/'state.json',{'status':'qualified','completed':[s[0] for s in stages]})

    def test_all_gates_recomputed_and_first_passing_seed_selected(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d);self.campaign(p);proof=verify(p)
            self.assertEqual(len(proof['phases']),11)
            self.assertEqual(proof['seed'],20001)
            self.assertTrue(proof['conditions']['policy']['mate_rendezvous'])

    def test_qualified_label_cannot_hide_missing_gate_or_bad_raw_lineage(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d);self.campaign(p)
            f=p/'qualification-both/seed-20001.json';r=json.loads(f.read_text())
            r['late_descendant_breeders']['Carnivore']=0;f.write_text(json.dumps(r))
            with self.assertRaisesRegex(ValueError,'summary'):verify(p)
            state=json.loads((p/'state.json').read_text());state['completed'].pop()
            (p/'state.json').write_text(json.dumps(state))
            with self.assertRaisesRegex(ValueError,'gates'):verify(p)

    def test_changed_policy_and_sensitivity_world_block_deployment(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d);self.campaign(p)
            f=p/'fixed-max_energy-90.json';v=json.loads(f.read_text());v['founders'][0]['life']['energy']+=1;f.write_text(json.dumps(v))
            with self.assertRaisesRegex(ValueError,'variant'):verify(p)
            c=json.loads((p/'config.json').read_text());c['mate_rendezvous']=False
            (p/'config.json').write_text(json.dumps(c))
            with self.assertRaisesRegex(ValueError,'protocol'):verify(p)
