#!/usr/bin/env python3
"""Read-only species/generation survival evidence from isolated run shards."""
import argparse, collections, json
from pathlib import Path
from measure_ecosystem import history_connection

def inspect(directory):
    individuals={}
    last_tick=0
    for path in sorted(Path(directory).glob('history-*')):
        if not path.name.endswith(('.sqlite','.sqlite.gz')):continue
        with history_connection(path) as db:
            for tick,kind,payload in db.execute('SELECT tick,kind,state_json FROM records ORDER BY id'):
                if kind not in ('initial','tick'):continue
                last_tick=max(last_tick,tick)
                for e in json.loads(payload)['entities']:
                    row=individuals.setdefault(e['id'],{'species':e['species'],'generation':e['generation'],'born':tick,'mature':False,'reproduced':False,'ate':0,'drank':0,'death':None})
                    row['mature'] |= e['life']['health']>0 and e['life']['age']>=e['life']['maturity_age']
            for tick,kind,payload in db.execute('SELECT r.tick,e.kind,e.payload FROM events e JOIN records r ON r.id=e.record_id ORDER BY r.id,e.ordinal'):
                e=json.loads(payload)[kind]
                if kind=='EntityMated':
                    for id in [e['parent1'],e['parent2']]:individuals[id]['reproduced']=True
                elif kind=='EntityDied':individuals[e['id']]['death']={'tick':tick,'cause':e.get('cause')}
                elif kind=='EntityAte':individuals[e['id']]['ate']+=e['amount']
                elif kind=='EntityDrank':individuals[e['id']]['drank']+=e['amount']
    species={}
    for name in ['Herbivore','Carnivore','Omnivore']:
        rows=[e for e in individuals.values() if e['species']==name]
        if not rows:continue
        young=[e for e in rows if e['generation']>0]
        species[name]={'births':len(young),'descendants_matured':sum(e['mature'] for e in young),'descendants_reproduced':sum(e['reproduced'] for e in young),'max_generation':max(e['generation'] for e in rows),'deaths':dict(collections.Counter(e['death']['cause'] for e in rows if e['death']))}
    return {'last_tick':last_tick,'species':species,'individuals':individuals}
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('directory');args=p.parse_args()
    print(json.dumps(inspect(args.directory),indent=2))
