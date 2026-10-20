#!/usr/bin/env python3
"""Inspect recorded decisions read-only, including compressed historical shards."""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
from measure_ecosystem import history_connection


def diagnose(directory):
    totals=Counter(); individuals=defaultdict(lambda:{'reasons':Counter(),'first_tick':None,'last_tick':None})
    examples=[]
    for path in sorted(directory.glob('history-*')):
        if not path.name.endswith(('.sqlite','.sqlite.gz', '.sqlite.zst')):continue
        with history_connection(path) as db:
            if not db.execute("SELECT 1 FROM sqlite_master WHERE name='decision_traces'").fetchone():continue
            for (encoded,) in db.execute('SELECT trace_json FROM decision_traces ORDER BY record_id,entity_id'):
                trace=json.loads(encoded);body=trace['before'];reason=trace['decision']['reason']
                row=individuals[trace['id']];row['first_tick']=row['first_tick'] or trace['tick'];row['last_tick']=trace['tick'];row['reasons'][reason]+=1
                ready=(body['age']>=body['maturity_age'] and body['reproduction_cooldown']==0
                       and body['energy']>=body['max_energy']//2 and body['water']>=body['max_water']//2 and body['health']>=body['max_health']//2)
                if not ready:continue
                mates=[e for e in trace['seen']['entities'] if e['species']==trace['species'] and e['reproductive']]
                totals['ready']+=1
                totals['ready_with_visible_mate' if mates else 'ready_without_visible_mate']+=1
                if any(e['distance']<=1 for e in mates):
                    totals['ready_adjacent_mate']+=1;totals['adjacent_reason_'+reason]+=1
                    if reason in ('SeekWater','SeekFood') and len(examples)<8:examples.append(trace)
    return {'directory':str(directory),'totals':dict(totals),'individuals':dict(individuals),'examples':examples}

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('directory',type=Path);parser.add_argument('--output',type=Path)
    args=parser.parse_args();report=diagnose(args.directory)
    if args.output:args.output.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k!='examples'},indent=2))
