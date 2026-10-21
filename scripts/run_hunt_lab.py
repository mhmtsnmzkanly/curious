#!/usr/bin/env python3
"""Run and audit the preregistered four-arm physical hunt laboratory once."""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

PAIRS=[(1,41001),(2,41002),(3,41003),(4,41004),(5,41005),(42,41006),(12345,41007),(2026,41008)]
TRACKS=['passive','defensive']
ARMS=['rule_based','naive_online','experienced_online','experienced_frozen']
WINDOWS=['normal_first_20','normal_last_20','reversed_first_20','reversed_last_20']
SUMS=['ticks','food','movement_energy','attack_energy','incoming_damage','rejected','failed_ticks','energy_spent','non_action_energy']

def sha(p):
    h=hashlib.sha256()
    with Path(p).open('rb') as f:
        for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
    return h.hexdigest()

def schedule(seed,n):
    out=[]
    for _ in range(n):out.append(seed);seed=(seed*6364136223846793005+1)&((1<<64)-1)
    return out

def visits(values):return sum(v['visits'] for v in values)

def check_trial(t,index,seed,reversed,previous):
    if (t['trial']!=index or t['world_seed']!=seed or t['reversed']!=reversed or t['visits_before']!=previous
            or not 1<=t['ticks']<=20 or t['ticks']!=len(t['timeline'])):
        raise ValueError('Trial identity, clock or visit continuity mismatch')
    for key in SUMS:
        if not isinstance(t[key],int) or t[key]<0:raise ValueError('Negative/invalid physical counter')
    for key in ['food','movement_energy','attack_energy','incoming_damage','rejected']:
        if sum(x[key] for x in t['timeline'])!=t[key]:raise ValueError('Physical timeline accounting mismatch')
    hunter=next(b for b in t['initial_bodies'] if b['id']==1)
    if t['energy_spent']!=hunter['life']['energy']+t['food']-t['timeline'][-1]['energy'] or t['non_action_energy']!=t['energy_spent']-t['movement_energy']-t['attack_energy']:
        raise ValueError('Body energy balance mismatch')
    if [x['tick'] for x in t['timeline']]!=list(range(1,t['ticks']+1)):raise ValueError('Tick order mismatch')
    first=next((x['tick'] for x in t['timeline'] if x['food']>0),None)
    if first!=t['first_food_tick'] or (first is not None and first!=t['ticks']):raise ValueError('First meal clock mismatch')
    if t['failed_ticks']!=(t['ticks'] if t['food']==0 else 0):raise ValueError('Failed pursuit time mismatch')
    if t['stop'] not in ['FirstFood','HunterDied','TrialLimit']:raise ValueError('Unknown trial stop')
    if (t['stop']=='FirstFood')!=(t['food']>0):raise ValueError('Meal/stop mismatch')
    if t['stop']=='TrialLimit' and t['ticks']!=20:raise ValueError('Premature lab limit')
    if t['stop']=='HunterDied' and not t['hunter_died']:raise ValueError('False death')
    updated=0
    for e in t['endings']:
        r=e['result']
        if e['origin'] not in ['engine','lab_boundary']:raise ValueError('Unknown ending origin')
        if r['end'] not in ['Fed','TargetLost','TimedOut','Interrupted','Died','ForeignKill']:raise ValueError('Unknown ending')
        if r['updated'] and r['reward'] is None:raise ValueError('Update without reward')
        if e['origin']=='lab_boundary' and (r['reward'] is not None or r['updated']):raise ValueError('Laboratory censorship was scored')
        if not 0<=r['attempt']['ticks']<=20:raise ValueError('Episode clock exceeds limit')
        updated+=int(r['updated'])
    if t['visits_after']-t['visits_before']!=updated:raise ValueError('Completed audit and table visits disagree')
    return t['visits_after']

def aggregate(trials):
    result={'trials':len(trials),**{k:sum(t[k] for t in trials) for k in SUMS},
            'fed_trials':sum(t['food']>0 for t in trials),'deaths':sum(t['hunter_died'] for t in trials)}
    engine=[e for t in trials for e in t['endings'] if e['origin']=='engine']
    boundary=[e for t in trials for e in t['endings'] if e['origin']=='lab_boundary']
    result.update({'engine_ends':dict(Counter(e['result']['end'] for e in engine)),
        'lab_boundary_ends':len(boundary),'interruption_triggers':dict(Counter(e['trigger'] for e in engine if e['result']['end']=='Interrupted')),
        'scored_ends':sum(e['result']['reward'] is not None for e in engine),
        'unscored_ends':sum(e['result']['reward'] is None for e in engine),
        'ambiguous_ends':sum(e['result']['attempt']['ambiguous'] for e in engine),
        'updates':sum(e['result']['updated'] for e in engine),
        'scored_fraction':sum(e['result']['reward'] is not None for e in engine)/len(engine) if engine else None})
    result['total_action_energy']=result['movement_energy']+result['attack_energy']
    result['food_per_action_energy']=result['food']/result['total_action_energy'] if result['total_action_energy'] else None
    result['food_per_spent_energy']=result['food']/result['energy_spent'] if result['energy_spent'] else None
    result['food_rate']=result['fed_trials']/len(trials) if trials else None
    return result

def validate_pair(row,track,training_seed,evaluation_seed):
    if (row['schema']!='curious-hunt-lab-v1' or row['track']!=track or row['training_seed']!=training_seed
        or row['evaluation_seed']!=evaluation_seed or row['training_trials']!=80 or row['evaluation_trials']!=80
        or row['reversal_at']!=40 or row['max_trial_ticks']!=20 or row['controlled_body_reset'] is not True
        or len(row['training'])!=80 or [a['arm'] for a in row['arms']]!=ARMS):raise ValueError('Protocol mismatch')
    previous=0
    for i,(t,seed) in enumerate(zip(row['training'],schedule(training_seed,80))):previous=check_trial(t,i,seed,False,previous)
    if previous!=visits(row['training_values']):raise ValueError('Training visits mismatch')
    reference=row['arms'][0]['trials'];summary={'track':track,'training_seed':training_seed,'evaluation_seed':evaluation_seed,
        'training_first_40':aggregate(row['training'][:40]),'training_last_40':aggregate(row['training'][40:]),
        'trained_contexts':len(row['training_values']),'trained_visits':previous,'arms':{}}
    for a in row['arms']:
        if len(a['trials'])!=80:raise ValueError('Evaluation trial count mismatch')
        name=a['arm'];previous=visits(a['initial_values'])
        if name.startswith('experienced') and a['initial_values']!=row['training_values']:raise ValueError('Unequal trained histories')
        if name in ['rule_based','naive_online'] and a['initial_values']:raise ValueError('Naive/control history is not empty')
        for i,(t,seed) in enumerate(zip(a['trials'],schedule(evaluation_seed,80))):
            previous=check_trial(t,i,seed,i>=40,previous)
            for key in ['initial_bodies','initial_seen']:
                if t[key]!=reference[i][key]:raise ValueError('Unequal initial bodies or perceptions')
            if name=='experienced_frozen' and (t['visits_after']!=visits(a['initial_values']) or any(e['result']['updated'] for e in t['endings'])):raise ValueError('Frozen table updated')
            if name=='rule_based' and (t['endings'] or t['visits_after']):raise ValueError('Disabled learner emitted results')
        if previous!=visits(a['final_values']):raise ValueError('Final visits mismatch')
        if name=='experienced_frozen' and a['initial_values']!=a['final_values']:raise ValueError('Frozen values changed')
        summary['arms'][name]={'total':aggregate(a['trials']),'final_contexts':len(a['final_values']),
            'windows':{label:aggregate(a['trials'][i*20:(i+1)*20]) for i,label in enumerate(WINDOWS)}}
    return summary

def summarize(path):
    expected=[(track,a,b) for track in TRACKS for a,b in PAIRS];results=[]
    with Path(path).open() as f:
        for track,a,b in expected:
            line=f.readline()
            if not line:raise ValueError('Missing laboratory pair')
            results.append(validate_pair(json.loads(line),track,a,b))
        if f.read().strip():raise ValueError('Unexpected extra results')
    means={}
    keys=SUMS+['fed_trials','deaths','total_action_energy','scored_ends','unscored_ends','updates','ambiguous_ends','lab_boundary_ends']
    for track in TRACKS:
        selected=[r for r in results if r['track']==track];means[track]={}
        for arm in ARMS:
            means[track][arm]={window:{k:sum(r['arms'][arm]['windows'][window][k] for r in selected)/len(selected) for k in keys} for window in WINDOWS}
    return {'schema':'curious-hunt-lab-summary-v1','pairs_per_track':8,'tracks':TRACKS,'controlled_body_reset':True,
        'superiority_claim':None,'mean_per_20_trials':means,'per_pair':results}

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',required=True,type=Path);p.add_argument('--output',required=True,type=Path);a=p.parse_args()
    repo=Path(__file__).resolve().parents[1];binary=a.binary.resolve(strict=True);output=a.output.resolve()
    if subprocess.check_output(['git','status','--porcelain'],cwd=repo,text=True).strip():raise ValueError('Commit clean source before the registered run')
    output.mkdir(parents=True,exist_ok=False);probe=output/'hunt-lab-probe';shutil.copy2(binary,probe)
    protocol=repo/'docs/HUNT_LAB_PROTOCOL.md';shutil.copy2(protocol,output/'protocol.md');work=output/'work';work.mkdir()
    paths=[*sorted((repo/'src').rglob('*.rs')),repo/'examples/hunt_lab.rs',repo/'Cargo.toml',repo/'Cargo.lock']
    manifest={'schema':'curious-hunt-lab-run-v1','status':'running','revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),
        'git_dirty':False,'binary_sha256':sha(probe),'protocol_sha256':sha(protocol),'sources':{str(x.relative_to(repo)):sha(x) for x in paths},
        'started_at':datetime.now(timezone.utc).isoformat(),'timeout_seconds':300}
    dest=output/'manifest.json';dest.write_text(json.dumps(manifest,indent=2)+'\n')
    try:
        with (output/'raw.jsonl').open('x') as raw,(output/'stderr.log').open('x') as err:
            subprocess.run([str(probe),'--samples',str(output/'samples')],cwd=work,stdout=raw,stderr=err,check=True,timeout=300)
        summary=summarize(output/'raw.jsonl');(output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
        manifest.update(status='completed',raw_sha256=sha(output/'raw.jsonl'),summary_sha256=sha(output/'summary.json'))
    except Exception as exc:
        manifest.update(status='failed',error=str(exc));dest.write_text(json.dumps(manifest,indent=2)+'\n');raise
    manifest['finished_at']=datetime.now(timezone.utc).isoformat();dest.write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({'status':manifest['status'],'output':str(output)}))
if __name__=='__main__':main()
