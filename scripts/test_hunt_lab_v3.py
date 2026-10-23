import copy
import unittest

from run_hunt_lab import schedule
from run_hunt_lab_v3 import aggregate, eligible_observations, risk_totals, validate_pair


class HuntLabV3AuditTests(unittest.TestCase):
    @staticmethod
    def valid_pair():
        train_seed, eval_seed = 1, 43001
        bodies = [{'id': 1, 'life': {'energy': 10}}]
        seen = [{'id': 2}]
        def trial(i, seed, reverse=False, risk=False):
            row = {'trial': i, 'world_seed': seed, 'reversed': reverse, 'visits_before': 0, 'visits_after': 0,
                   'ticks': 1, 'food': 1, 'movement_energy': 0, 'attack_energy': 1, 'incoming_damage': 0,
                   'prey_defense_damage': 0, 'external_damage': 0, 'unattributed_damage': 0, 'rejected': 0,
                   'failed_ticks': 0, 'energy_spent': 1, 'non_action_energy': 0, 'hunter_died': False,
                   'final_health': 10, 'initial_bodies': copy.deepcopy(bodies), 'initial_seen': copy.deepcopy(seen),
                   'first_food_tick': 1, 'stop': 'FirstFood', 'endings': [],
                   'timeline': [{'tick': 1, 'food': 1, 'movement_energy': 0, 'attack_energy': 1,
                                 'incoming_damage': 0, 'prey_defense_damage': 0, 'external_damage': 0,
                                 'unattributed_damage': 0, 'rejected': 0, 'energy': 10}]}
            if risk:
                row.update(risk_exposures_before=0, risk_exposures_after=0,
                           risk_injuries_before=0, risk_injuries_after=0,
                           risk_damage_before=0, risk_damage_after=0)
                row['timeline'][0]['risk_explored_unknown'] = False
            return row
        seeds=schedule(train_seed,80); evals=schedule(eval_seed,80)
        plain=[trial(i,s) for i,s in enumerate(seeds)]
        risk_train=[trial(i,s,risk=True) for i,s in enumerate(seeds)]
        arms=[]
        flags=[(False,False,True),(True,False,True),(True,False,True),
               (True,True,True),(True,True,False),(True,False,False)]
        names=['rule_based','naive_online','experienced_online','experienced_risk_online',
               'experienced_risk_frozen','experienced_risk_ablation_frozen']
        for name,(enabled,risk_enabled,update_enabled) in zip(names,flags):
            rows=[trial(i,s,i>=40,risk_enabled) for i,s in enumerate(evals)]
            arms.append({'arm':name,'enabled':enabled,'risk_enabled':risk_enabled,'update_enabled':update_enabled,
                         'initial_values':[],'final_values':[],'initial_risk_values':[],'final_risk_values':[],'trials':rows})
        return {'schema':'curious-hunt-lab-v3','track':'passive','training_seed':train_seed,'evaluation_seed':eval_seed,
                'training_trials':80,'evaluation_trials':80,'reversal_at':40,'max_trial_ticks':20,
                'controlled_body_reset':True,'training':plain,'risk_training':risk_train,
                'training_values':[],'risk_training_values':[],'risk_training_risk_values':[],'arms':arms}

    def test_risk_evidence_requires_valid_bounded_joint_counts(self):
        record = {'context': {'energy': 0, 'stamina': 1, 'distance': 2, 'health': 3},
                  'exposures': 4, 'injuries': 2, 'damage': 7}
        self.assertEqual(risk_totals([record]), (4, 2, 7))
        for bad in [dict(record, injuries=5), dict(record, damage=1), dict(record, damage=0),
                    dict(record, exposures=1_000_001)]:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                risk_totals([bad])

    def test_summary_keeps_injury_and_food_outcomes_separate(self):
        t = {'ticks': 2, 'food': 5, 'movement_energy': 1, 'attack_energy': 3,
             'energy_spent': 7, 'incoming_damage': 4, 'prey_defense_damage': 4,
             'external_damage': 0, 'unattributed_damage': 0, 'rejected': 0,
             'failed_ticks': 0, 'hunter_died': False, 'final_health': 8,
             'risk_exposures_before': 2, 'risk_exposures_after': 4,
             'risk_injuries_before': 1, 'risk_injuries_after': 2,
             'risk_damage_before': 3, 'risk_damage_after': 7,
             'endings': [{'origin': 'lab_boundary', 'result': {'attempt': {
                 'attack_energy': 3, 'ambiguous': False, 'incoming_damage': 2,
                 'prey_defense_damage': 2}}}, {'origin': 'engine', 'result': {'attempt': {
                 'attack_energy': 3, 'ambiguous': False, 'incoming_damage': 2,
                 'prey_defense_damage': 2}}}],
             'timeline': [{'risk_explored_unknown': True}]}
        result = aggregate([t])
        self.assertEqual(result['food'], 5)
        self.assertEqual(result['observed_injuries'], 1)
        self.assertEqual(result['observed_exposures'], 1)
        self.assertEqual(result['observed_injury_rate'], 1.0)
        self.assertEqual(result['model_risk_injuries_delta'], 1)
        self.assertEqual(result['model_risk_exposures_delta'], 2)
        self.assertEqual(result['mean_final_health'], 8)
        self.assertEqual(result['unknown_risk_explorations'], 1)

    def test_frozen_policy_observations_are_distinct_from_model_counter_updates(self):
        trial = {'endings': [
            {'origin': 'engine', 'result': {'attempt': {'attack_energy': 3, 'ambiguous': False,
             'incoming_damage': 4, 'prey_defense_damage': 4}}},
            {'origin': 'engine', 'result': {'attempt': {'attack_energy': 3, 'ambiguous': True,
             'incoming_damage': 8, 'prey_defense_damage': 4}}},
            {'origin': 'lab_boundary', 'result': {'attempt': {'attack_energy': 3, 'ambiguous': False,
             'incoming_damage': 2, 'prey_defense_damage': 2}}},
        ]}
        self.assertEqual(eligible_observations(trial), (1, 1, 4))

    def test_full_validator_rejects_arm_flags_truncation_and_counter_drift(self):
        row=self.valid_pair()
        validate_pair(row,'passive',1,43001)
        bad=copy.deepcopy(row); bad['arms'][4]['update_enabled']=True
        with self.assertRaises(ValueError): validate_pair(bad,'passive',1,43001)
        bad=copy.deepcopy(row); bad['arms'][2]['trials'].pop()
        with self.assertRaises(ValueError): validate_pair(bad,'passive',1,43001)
        bad=copy.deepcopy(row); bad['risk_training'][0]['risk_exposures_after']=1
        with self.assertRaises(ValueError): validate_pair(bad,'passive',1,43001)


if __name__ == '__main__':
    unittest.main()
