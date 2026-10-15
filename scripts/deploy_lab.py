#!/usr/bin/env python3
"""Back up the paused lab, promote a tested immutable binary, verify additive migration.
Rollback retains the failed run too. Does not deploy over the long-running meadow.
"""
import argparse,datetime,hashlib,json,os,shutil,sqlite3,subprocess,time,urllib.request
from pathlib import Path

def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def preserved(old,new):
 if isinstance(old,dict):return all(k in new and preserved(v,new[k]) for k,v in old.items())
 if isinstance(old,list):return len(old)==len(new) and all(preserved(a,b) for a,b in zip(old,new))
 return old==new

def main():
 p=argparse.ArgumentParser();p.add_argument('binary');p.add_argument('--revision',required=True);a=p.parse_args()
 root=Path(__file__).resolve().parents[1];source=Path(a.binary).resolve();data=root/'data/lab';link=root/'bin/curious-lab'
 revision=subprocess.check_output(['git','rev-parse','--short','HEAD'],cwd=root,text=True).strip()
 if a.revision!=revision or subprocess.check_output(['git','status','--porcelain'],cwd=root,text=True).strip():raise RuntimeError('deploy requires current committed revision and clean worktree')
 def request(route):
  headers={};token=os.environ.get('CURIOUS_DEPLOY_TOKEN')
  if token:headers['Authorization']='Bearer '+token
  with urllib.request.urlopen(urllib.request.Request('http://127.0.0.1:7880'+route,headers=headers),timeout=5) as f:return json.load(f)
 before=request('/state');assert request('/status')['paused'],'pause lab before deployment'
 previous=link.resolve();destination=root/'bin/releases'/('curious-'+revision)
 if destination.exists():assert digest(destination)==digest(source),'immutable release already differs'
 else:shutil.copy2(source,destination);destination.chmod(0o755)
 stamp=datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ');backup=root/'data/backups'/('lab-'+stamp);backup.mkdir(parents=True)
 subprocess.run(['systemctl','stop','curious-lab.service'],check=True);promoted=False
 def point(target):
  temporary=link.with_name(link.name+'.next');temporary.unlink(missing_ok=True);temporary.symlink_to(os.path.relpath(target,link.parent));temporary.replace(link)
 try:
  shutil.copytree(data,backup/'data')
  for db in (backup/'data').glob('history-*.sqlite'):
   with sqlite3.connect(f'file:{db}?mode=ro',uri=True) as c:assert c.execute('PRAGMA integrity_check').fetchone()[0]=='ok'
  (backup/'before.json').write_text(json.dumps(before)+'\n')
  point(destination);promoted=True
  subprocess.run(['systemctl','start','curious-lab.service'],check=True)
  for _ in range(50):
   try:after=request('/state');break
   except OSError:time.sleep(.1)
  else:raise RuntimeError('new lab did not become ready')
  legacy=dict(before);legacy.pop('version');assert preserved(legacy,after),'migration changed existing state'
  metadata={'revision':revision,'binary_sha256':digest(destination),'previous_binary':str(previous),'backup':str(backup),'tick':after['tick'],'before_version':before['version'],'after_version':after['version'],'paused':request('/status')['paused']}
  (backup/'deployment.json').write_text(json.dumps(metadata,indent=2)+'\n');(data/'deployment.json').write_text(json.dumps(metadata,indent=2)+'\n');print(json.dumps(metadata))
 except BaseException:
  if promoted:
   subprocess.run(['systemctl','stop','curious-lab.service'],check=True)
   data.rename(backup/'failed-data');shutil.copytree(backup/'data',data);point(previous)
  subprocess.run(['systemctl','start','curious-lab.service'],check=True)
  raise
if __name__=='__main__':main()
