import unittest
from run_hunt_lab import ARMS,aggregate,schedule,summarize,validate_pair


def trial(index,seed,reversed=False):
    return {'trial':index,'world_seed':seed,'reversed':reversed,'initial_bodies':[{'id':1,'life':{'energy':40},'health':140,'age':0}],
        'initial_seen':[],'ticks':1,'food':10,'movement_energy':1,'attack_energy':3,
        'energy_spent':5,'non_action_energy':1,'incoming_damage':0,'rejected':0,'hunter_died':False,'first_food_tick':1,
        'failed_ticks':0,'stop':'FirstFood','visits_before':0,'visits_after':0,'endings':[],
        'prey_defense_damage':0,'external_damage':0,'unattributed_damage':0,
        'timeline':[{'tick':1,'energy':45,'food':10,'movement_energy':1,'attack_energy':3,'incoming_damage':0,'rejected':0,
                     'prey_defense_damage':0,'external_damage':0,'unattributed_damage':0}]}


def pair(track='passive',training_seed=1,evaluation_seed=41001,version=1):
    return {'schema':f'curious-hunt-lab-v{version}','track':track,'training_seed':training_seed,'evaluation_seed':evaluation_seed,
        'training_trials':80,'evaluation_trials':80,'reversal_at':40,'max_trial_ticks':20,
        'controlled_body_reset':True,'training_values':[],
        'training':[trial(i,s) for i,s in enumerate(schedule(training_seed,80))],
        'arms':[{'arm':name,'initial_values':[],'final_values':[],
                 'trials':[trial(i,s,i>=40) for i,s in enumerate(schedule(evaluation_seed,80))]} for name in ARMS]}


class HuntLabAuditTests(unittest.TestCase):
    def test_cost_and_meal_metrics_and_fixed_windows(self):
        r=validate_pair(pair(),'passive',1,41001)
        a=r['arms']['rule_based']['windows']['normal_first_20']
        self.assertEqual(a['food'],200);self.assertEqual(a['total_action_energy'],80)
        self.assertEqual(a['fed_trials'],20);self.assertEqual(a['food_per_action_energy'],2.5)
        self.assertIsNone(a['scored_fraction'])

    def test_tampered_cost_body_or_frozen_history_is_rejected(self):
        for key in ['cost','body','frozen']:
            r=pair()
            if key=='cost':r['arms'][1]['trials'][0]['movement_energy']=99
            elif key=='body':r['arms'][2]['trials'][0]['initial_bodies'][0]['health']=141
            else:r['arms'][3]['final_values']=[{'visits':0,'value':99}]
            with self.subTest(key=key),self.assertRaises(ValueError):validate_pair(r,'passive',1,41001)

    def test_lab_censorship_cannot_be_counted_as_learning(self):
        r=pair();r['arms'][1]['trials'][0]['endings']=[{'origin':'lab_boundary','trigger':'TrialLimit',
            'result':{'end':'Interrupted','reward':0,'updated':False,'attempt':{'ticks':1}}}]
        with self.assertRaisesRegex(ValueError,'censorship'):validate_pair(r,'passive',1,41001)

    def test_rewardless_engine_result_is_separate_from_boundary(self):
        t=trial(0,1);t['endings']=[{'origin':'engine','trigger':'SeekWater','result':{
            'end':'Interrupted','reward':None,'updated':False,'attempt':{'ticks':1,'ambiguous':False}}},
            {'origin':'lab_boundary','trigger':'TrialLimit','result':{'end':'Interrupted','reward':None,
                'updated':False,'attempt':{'ticks':1,'ambiguous':False}}}]
        a=aggregate([t]);self.assertEqual(a['unscored_ends'],1);self.assertEqual(a['lab_boundary_ends'],1)
        self.assertEqual(a['interruption_triggers'],{'SeekWater':1});self.assertEqual(a['updates'],0)

    def test_v2_damage_aggregation_and_eight_pair_means(self):
        rows=[]
        for track in ['passive','defensive']:
            for i,(training,_) in enumerate([(1,0),(2,0),(3,0),(4,0),(5,0),(42,0),(12345,0),(2026,0)]):
                p=pair(track,training,42001+i,2)
                for bucket in [p['training'],*[a['trials'] for a in p['arms']]]:
                    for t in bucket:
                        t['incoming_damage']=3;t['prey_defense_damage']=2;t['external_damage']=1
                        t['timeline'][0].update(incoming_damage=3,prey_defense_damage=2,external_damage=1)
                rows.append(p)
        import json,tempfile
        from pathlib import Path
        with tempfile.TemporaryDirectory() as temp:
            raw=Path(temp)/'raw.jsonl';raw.write_text(''.join(json.dumps(row)+'\n' for row in rows))
            result=summarize(raw,[(1,42001),(2,42002),(3,42003),(4,42004),(5,42005),(42,42006),(12345,42007),(2026,42008)],2)
        mean=result['mean_per_20_trials']['passive']['rule_based']['normal_first_20']
        self.assertEqual(mean['prey_defense_damage'],40)
        self.assertEqual(mean['external_damage'],20)

    def test_v2_rejects_malformed_source_sum(self):
        row=pair(evaluation_seed=42001,version=2)
        t=row['training'][0]
        t['incoming_damage']=2;t['prey_defense_damage']=1
        t['timeline'][0]['incoming_damage']=2;t['timeline'][0]['prey_defense_damage']=1
        with self.assertRaisesRegex(ValueError,'Damage source accounting'):
            validate_pair(row,'passive',1,42001,2)

if __name__=='__main__':unittest.main()
