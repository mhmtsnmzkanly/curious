#!/usr/bin/env python3
"""Run and validate the bounded six-arm V3 hunt risk laboratory once."""
from collections import Counter
from datetime import datetime, timezone
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

import run_hunt_lab as common

PAIRS = [(training, 43001 + i) for i, (training, _) in enumerate(common.PAIRS)]
ARMS = ['rule_based', 'naive_online', 'experienced_online', 'experienced_risk_online',
        'experienced_risk_frozen', 'experienced_risk_ablation_frozen']
TRACKS = ['passive', 'defensive']
WINDOWS = ['normal_first_20', 'normal_last_20', 'reversed_first_20', 'reversed_last_20']

def risk_totals(values):
    if len(values) > 108: raise ValueError('Risk table exceeds context bound')
    seen=set(); exposures=injuries=damage=0
    for v in values:
        c=tuple(v['context'][k] for k in ('energy','stamina','distance','health'))
        if any(type(x) is not int for x in c) or c in seen or not (0<=c[0]<3 and 0<=c[1]<3 and 0<=c[2]<3 and 0<=c[3]<4): raise ValueError('Invalid/duplicate risk context')
        seen.add(c); e=v['exposures']; i=v['injuries']; d=v['damage']
        if any(type(x) is not int for x in (e,i,d)) or not 0<e<=1_000_000 or i<0 or i>e or d<0 or d<i or (i==0)!=(d==0) or d>e*1_000_000: raise ValueError('Risk evidence bounds/invariants violated')
        exposures+=e; injuries+=i; damage+=d
    return exposures,injuries,damage

def validate_v3_trial(t,index,seed,reversed,previous):
    current=common.check_trial(t,index,seed,reversed,previous,2)
    for key in ['final_health','prey_defense_damage','external_damage','unattributed_damage']:
        if not isinstance(t.get(key),int) or t[key]<0: raise ValueError(f'Invalid {key}')
    if t.get('risk_exposures_before') is not None:
        before=(t['risk_exposures_before'],t['risk_injuries_before'],t['risk_damage_before'])
        after=(t['risk_exposures_after'],t['risk_injuries_after'],t['risk_damage_after'])
        if any(type(x) is not int or x<0 for x in (*before,*after)): raise ValueError('Partial risk counter triplet')
        if any(b>a for b,a in zip(before,after)): raise ValueError('Risk counters regressed')
    for tick in t['timeline']:
        if 'risk_explored_unknown' in tick and not isinstance(tick['risk_explored_unknown'],bool):
            raise ValueError('Invalid unknown-context exploration audit')
    return current

def expected_risk_delta(t, risk_enabled, update_enabled):
    if not (risk_enabled and update_enabled): return (0,0,0)
    return eligible_observations(t)

def eligible_observations(t):
    exposures=injuries=damage=0
    for ending in t['endings']:
        if ending['origin']!='engine': continue
        a=ending['result']['attempt']
        if (a['attack_energy']>0 and not a['ambiguous']
            and a['incoming_damage']==a['prey_defense_damage']):
            exposures+=1; injuries+=int(a['prey_defense_damage']>0); damage+=a['prey_defense_damage']
    return exposures,injuries,damage

def aggregate(trials):
    result={'trials':len(trials)}
    for key in ['ticks','food','movement_energy','attack_energy','energy_spent','incoming_damage',
                'prey_defense_damage','external_damage','unattributed_damage','rejected','failed_ticks']:
        result[key]=sum(t[key] for t in trials)
    result.update(deaths=sum(bool(t['hunter_died']) for t in trials),
                  mean_final_health=sum(t['final_health'] for t in trials)/len(trials),
                  fed_trials=sum(t['food']>0 for t in trials),
                  censored=sum(e['origin']=='lab_boundary' for t in trials for e in t['endings']),
                  injured_trials=sum(t['prey_defense_damage']>0 for t in trials),
                  censored_eligible_windows=sum(e['origin']=='lab_boundary' and e['result']['attempt']['attack_energy']>0
                      and not e['result']['attempt']['ambiguous']
                      and e['result']['attempt']['incoming_damage']==e['result']['attempt']['prey_defense_damage']
                      for t in trials for e in t['endings']),
                  unknown_risk_explorations=sum(x.get('risk_explored_unknown') is True for t in trials for x in t['timeline']))
    observed=[eligible_observations(t) for t in trials]
    result['observed_exposures']=sum(x[0] for x in observed)
    result['observed_injuries']=sum(x[1] for x in observed)
    result['observed_damage']=sum(x[2] for x in observed)
    result['observed_injury_rate']=(result['observed_injuries']/result['observed_exposures']
                                    if result['observed_exposures'] else None)
    for key in ['risk_exposures','risk_injuries','risk_damage']:
        bk=key.replace('risk_','risk_')+'_before'; ak=key.replace('risk_','risk_')+'_after'
        if all(t.get(bk) is not None for t in trials):
            result['model_'+key+'_delta']=sum(t[ak]-t[bk] for t in trials)
    result['injury_rate_per_trial']=result['injured_trials']/len(trials) if trials else None
    return result

def validate_pair(row,track,training_seed,evaluation_seed):
    if (row.get('schema')!='curious-hunt-lab-v3' or row['track']!=track or row['training_seed']!=training_seed
        or row['evaluation_seed']!=evaluation_seed or row['training_trials']!=80 or row['evaluation_trials']!=80
        or row['reversal_at']!=40 or row['max_trial_ticks']!=20 or row['controlled_body_reset'] is not True
        or len(row['training'])!=80 or len(row['risk_training'])!=80 or [a['arm'] for a in row['arms']]!=ARMS):
        raise ValueError('V3 protocol identity mismatch')
    seeds=common.schedule(training_seed,80)
    for i,s in enumerate(seeds):
        if row['training'][i]['world_seed']!=s or row['risk_training'][i]['world_seed']!=s: raise ValueError('Training schedule mismatch')
        for key in ('initial_bodies','initial_seen'):
            if row['training'][i][key]!=row['risk_training'][i][key]: raise ValueError('Plain/risk training initial state mismatch')
    for col,risk_enabled in (('training',False),('risk_training',True)):
        prev=0
        risk_before=(0,0,0)
        for i,(t,s) in enumerate(zip(row[col],seeds)):
            prev=validate_v3_trial(t,i,s,False,prev)
            if risk_enabled:
                if t.get('risk_exposures_before') is None: raise ValueError('Risk training counters missing')
                before=tuple(t[k+'_before'] for k in ('risk_exposures','risk_injuries','risk_damage'))
                after=tuple(t[k+'_after'] for k in ('risk_exposures','risk_injuries','risk_damage'))
                if before!=risk_before or tuple(a-b for a,b in zip(after,before))!=expected_risk_delta(t,True,True): raise ValueError('Risk training counter audit mismatch')
                risk_before=after
        vals=row['training_values'] if col=='training' else row['risk_training_values']
        if prev!=common.visits(vals): raise ValueError('Training visit total mismatch')
        if risk_enabled and risk_before!=risk_totals(row['risk_training_risk_values']): raise ValueError('Risk training table and counter ledger disagree')
    risk_totals(row['risk_training_risk_values'])
    if row['risk_training_risk_values'] != row['arms'][4]['initial_risk_values'] or row['risk_training_risk_values'] != row['arms'][5]['initial_risk_values']:
        raise ValueError('Frozen risk arms do not share trained evidence')
    if row['risk_training_values'] != row['arms'][4]['initial_values'] or row['risk_training_values'] != row['arms'][5]['initial_values']:
        raise ValueError('Frozen risk arms do not share trained Q table')
    seeds=common.schedule(evaluation_seed,80); reference=row['arms'][0]['trials']; summary={'track':track,'training_seed':training_seed,'evaluation_seed':evaluation_seed,'arms':{}}
    for arm in row['arms']:
        name=arm['arm']; prev=common.visits(arm['initial_values'])
        expected_flags={
            'rule_based':(False,False,True), 'naive_online':(True,False,True),
            'experienced_online':(True,False,True), 'experienced_risk_online':(True,True,True),
            'experienced_risk_frozen':(True,True,False), 'experienced_risk_ablation_frozen':(True,False,False)}[name]
        if tuple(arm.get(k) for k in ('enabled','risk_enabled','update_enabled'))!=expected_flags: raise ValueError('Arm feature flags mismatch')
        if any(type(arm.get(k)) is not bool for k in ('enabled','risk_enabled','update_enabled')): raise ValueError('Arm flags must be booleans')
        if len(arm['trials'])!=80: raise ValueError('Evaluation trial count mismatch')
        if name=='experienced_online' and arm['initial_values']!=row['training_values']: raise ValueError('Plain experienced arm has wrong training table')
        if name=='experienced_risk_online' and (arm['initial_values']!=row['risk_training_values'] or arm['initial_risk_values']!=row['risk_training_risk_values']): raise ValueError('Risk online arm has wrong training tables')
        if name in ('rule_based','naive_online') and (arm['initial_values'] or arm['initial_risk_values']): raise ValueError('Naive/control arm must start empty')
        if name=='experienced_online' and arm['initial_risk_values']: raise ValueError('Plain learner must not receive risk history')
        risk_before=risk_totals(arm['initial_risk_values'])
        for i,(t,s) in enumerate(zip(arm['trials'],seeds)):
            prev=validate_v3_trial(t,i,s,i>=40,prev)
            if expected_flags[1] or arm['initial_risk_values']:
                if t.get('risk_exposures_before') is None: raise ValueError('Risk arm counters missing')
                before=tuple(t[k+'_before'] for k in ('risk_exposures','risk_injuries','risk_damage'))
                after=tuple(t[k+'_after'] for k in ('risk_exposures','risk_injuries','risk_damage'))
                if before!=risk_before or tuple(a-b for a,b in zip(after,before))!=expected_risk_delta(t,expected_flags[1],expected_flags[2]):
                    raise ValueError('Risk evaluation counter continuity/eligible-closure mismatch')
                risk_before=after
            elif name in ('rule_based','naive_online','experienced_online') and any(k in t for k in ('risk_exposures_before','risk_injuries_before','risk_damage_before')):
                raise ValueError('Risk-off baseline emitted model counter snapshots')
            if t['initial_bodies']!=reference[i]['initial_bodies'] or t['initial_seen']!=reference[i]['initial_seen']:
                raise ValueError('Evaluation initial body/perception mismatch')
        if prev!=common.visits(arm['final_values']): raise ValueError('Final visit count mismatch')
        if name in ('experienced_risk_frozen','experienced_risk_ablation_frozen'):
            if arm['initial_values']!=arm['final_values'] or arm['initial_risk_values']!=arm['final_risk_values']:
                raise ValueError('Frozen food/risk table changed')
            for t in arm['trials']:
                if any(t.get(k+'_before')!=t.get(k+'_after') for k in ('risk_exposures','risk_injuries','risk_damage')):
                    raise ValueError('Frozen risk counters changed in trial')
        risk_totals(arm['initial_risk_values']); risk_totals(arm['final_risk_values'])
        if (expected_flags[1] or arm['initial_risk_values']) and risk_before!=risk_totals(arm['final_risk_values']):
            raise ValueError('Final risk table and counter ledger disagree')
        summary['arms'][name]={'windows':{label:aggregate(arm['trials'][start:start+20]) for label,start in zip(WINDOWS,(0,20,40,60))},'overall':aggregate(arm['trials'])}
    return summary

def summarize(path):
    expected=[(track,a,b) for track in TRACKS for a,b in PAIRS]; out=[]
    with Path(path).open() as f:
        for track,a,b in expected:
            line=f.readline()
            if not line: raise ValueError('Missing V3 result row')
            out.append(validate_pair(json.loads(line),track,a,b))
        if f.read().strip(): raise ValueError('Unexpected extra V3 output')
    return {'schema':'curious-hunt-lab-summary-v3','pairs_per_track':8,'tracks':TRACKS,'per_pair':out,
            'interpretation':'Injury outcomes must be read jointly with food, energy, deaths and final health; no weighted reward or deployment claim.'}

def render_report(summary):
    lines=['# Hunt laboratory V3 report','',
           'Validated output from the preregistered six-arm physical study. Risk changes are reported with food, energy, deaths, and final health; the study does not imply ecological acceptance or deployment.','']
    for track in TRACKS:
        lines += [f'## {track.title()} track','']
        for window in WINDOWS:
            lines += [f'### {window.replace("_", " ").title()}','',
                      '| Arm | Injured trials / 20 | Injury / trial | Observed injuries / exposures | Model exposure Δ | Food / trial | Energy spent / trial | Deaths / 20 | Mean final health | Censored eligible windows | Unknown-risk explorations |',
                      '|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|']
            for arm in ARMS:
                rows=[p['arms'][arm]['windows'][window] for p in summary['per_pair'] if p['track']==track]
                mean=lambda key: sum(r[key] for r in rows if r.get(key) is not None)/sum(r.get(key) is not None for r in rows) if any(r.get(key) is not None for r in rows) else None
                n=mean('observed_injuries'); d=mean('observed_exposures')
                rate=(n/d if n is not None and d else None)
                vals=[mean('injured_trials'),mean('injury_rate_per_trial'),rate,mean('model_risk_exposures_delta'),mean('food')/20,mean('energy_spent')/20,
                      mean('deaths'),mean('mean_final_health'),mean('censored_eligible_windows'),mean('unknown_risk_explorations')]
                fmt=lambda v:'n/a' if v is None else f'{v:.3f}'
                lines.append(f"| {arm} | {fmt(vals[0])} | {fmt(vals[1])} | {fmt(n)} / {fmt(d)} | {fmt(vals[3])} | {fmt(vals[4])} | {fmt(vals[5])} | {fmt(vals[6])} | {fmt(vals[7])} | {fmt(vals[8])} | {fmt(vals[9])} |")
            lines.append('')
    return '\n'.join(lines)

def sha(path): return common.sha(path)

def main():
    p=argparse.ArgumentParser(description=__doc__); p.add_argument('--binary',required=True,type=Path); p.add_argument('--output',required=True,type=Path); a=p.parse_args()
    repo=Path(__file__).resolve().parents[1]; binary=a.binary.resolve(strict=True); output=a.output.resolve()
    if subprocess.check_output(['git','status','--porcelain'],cwd=repo,text=True).strip(): raise ValueError('Commit clean source before registered run')
    output.mkdir(parents=True,exist_ok=False); probe=output/'hunt-lab-probe'; shutil.copy2(binary,probe)
    protocol=repo/'docs/HUNT_LAB_PROTOCOL_V3.md'; shutil.copy2(protocol,output/'protocol.md'); work=output/'work'; work.mkdir()
    command=[str(probe),'--protocol-version','3','--evaluation-seed-start','43001']
    sources=[*sorted((repo/'src').rglob('*.rs')),repo/'examples/hunt_lab.rs',repo/'scripts/run_hunt_lab_v3.py',repo/'scripts/run_hunt_lab.py',repo/'Cargo.toml',repo/'Cargo.lock']
    manifest={'schema':'curious-hunt-lab-run-v3','protocol_version':3,'evaluation_pairs':[{'training_seed':x,'evaluation_seed':y} for x,y in PAIRS],
        'command':command,'status':'running','revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),
        'git_dirty':False,'binary_sha256':sha(probe),'protocol_sha256':sha(protocol),'sources':{str(x.relative_to(repo)):sha(x) for x in sources},
        'started_at':datetime.now(timezone.utc).isoformat(),'timeout_seconds':300,'tick_ceiling':204800}
    dest=output/'manifest.json'; dest.write_text(json.dumps(manifest,indent=2)+'\n')
    try:
        with (output/'raw.jsonl').open('x') as raw,(output/'stderr.log').open('x') as err:
            subprocess.run(command,cwd=work,stdout=raw,stderr=err,check=True,timeout=300)
        summary=summarize(output/'raw.jsonl'); (output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
        (output/'report.md').write_text(render_report(summary))
        manifest.update(status='completed',raw_sha256=sha(output/'raw.jsonl'),summary_sha256=sha(output/'summary.json'),report_sha256=sha(output/'report.md'))
    except Exception as exc:
        manifest.update(status='failed',error=str(exc),finished_at=datetime.now(timezone.utc).isoformat())
        if (output/'raw.jsonl').exists(): manifest['raw_sha256']=sha(output/'raw.jsonl')
        if (output/'stderr.log').exists(): manifest['stderr_sha256']=sha(output/'stderr.log')
        dest.write_text(json.dumps(manifest,indent=2)+'\n'); raise
    manifest['finished_at']=datetime.now(timezone.utc).isoformat(); dest.write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({'status':manifest['status'],'output':str(output)}))

if __name__=='__main__': main()
