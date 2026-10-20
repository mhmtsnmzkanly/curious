#!/usr/bin/env python3
"""Prepare a fresh, explicitly configured observation run after verifying all gates."""
import argparse
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess

from research_protocol import digest
from validate_coexistence import write_json
from verify_qualification import verify


def prepare(args):
    root=Path(args.root).resolve();source=Path(__file__).resolve().parents[1]
    revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=source,text=True).strip()
    if subprocess.check_output(['git','status','--porcelain'],cwd=source,text=True).strip():
        raise ValueError('Prepare from committed, clean source')
    proof=verify(args.campaign)
    # Storage/runtime changes are allowed; the biological engine must match its accepted source.
    changed=subprocess.check_output(['git','diff','--name-only',proof['engine_revision'],'HEAD','--','src'],cwd=source,text=True).splitlines()
    if set(changed)-{'src/archive.rs','src/persistence.rs'}:
        raise ValueError('Biological engine or control source changed since qualification')
    data=root/'data/coexistence';bootstrap=root/'data/coexistence-bootstrap'
    scenario=root/'data/coexistence-scenario.json';runtime=root/'data/coexistence-runtime.env'
    link=root/'bin/curious-qualified'
    if any(p.exists() or p.is_symlink() for p in [data,bootstrap,scenario,runtime,link,root/'data/coexistence-deployment.json']):
        raise ValueError('Observation paths already exist; never replace or reinitialize a world')
    with socket.socket() as sock:sock.bind(('127.0.0.1',7882))
    binary=Path(args.binary).resolve();initializer=Path(args.initializer).resolve()
    for path in [binary,initializer]:
        if not os.access(path,os.X_OK):raise ValueError('Missing executable: '+str(path))
    releases=root/'bin/releases';releases.mkdir(parents=True,exist_ok=True)
    engine=releases/('curious-observation-'+revision[:7]);initial=releases/('prepare-observation-'+revision[:7])
    for src,dst in [(binary,engine),(initializer,initial)]:
        if dst.exists():
            if digest(src)!=digest(dst):raise ValueError('Immutable release already differs')
        else:shutil.copy2(src,dst)
    bootstrap.mkdir()
    config=json.loads((Path(args.campaign)/'config.json').read_text())
    with scenario.open('xb') as stream:stream.write(Path(config['scenario']).read_bytes())
    write_json(bootstrap/'qualification.json',proof);write_json(bootstrap/'conditions.json',proof['conditions'])
    subprocess.run([str(initial),str(scenario),str(bootstrap/'conditions.json'),str(data)],check=True)
    state=json.loads((data/'initial-state.json').read_text())
    if state['tick']!=0 or state['policy']!=proof['conditions']['policy'] or state['genetics']!=proof['conditions']['genetics']:
        raise ValueError('Prepared observation settings differ from qualification')
    link.symlink_to(os.path.relpath(engine,link.parent))
    with runtime.open('x') as stream:
        stream.write(f"CURIOUS_SEED={proof['seed']}\nCURIOUS_PAGE={args.page_id}\nCURIOUS_OBSERVATION_SCRIPTS={source}/scripts\n")
    runtime.chmod(0o600)
    metadata={'revision':revision,'scripts_source':str(source),'accepted_engine_revision':proof['engine_revision'],
              'engine_sha256':digest(engine),'initializer_sha256':digest(initial),'seed':proof['seed'],
              'scenario_sha256':digest(scenario),'initial_state_sha256':digest(data/'initial-state.json'),
              'initial_tick':0,'policy':state['policy'],'genetics':state['genetics'],
              'learning_enabled':True,'companions_enabled':True,'tps':0.03,'history_budget_bytes':32768*1024**2,
              'research':str(Path(args.campaign).resolve()),'data':str(data),'engine':str(engine)}
    write_json(bootstrap/'deployment.json',metadata)
    return metadata


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--root',default='/root/curious');p.add_argument('--campaign',required=True)
    p.add_argument('--binary',required=True);p.add_argument('--initializer',required=True)
    p.add_argument('--page-id',default='p_6c6647a23f563ace')
    print(json.dumps(prepare(p.parse_args()),indent=2))
