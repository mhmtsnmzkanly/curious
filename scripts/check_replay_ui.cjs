#!/usr/bin/env node
// Isolated archive replay + browser checks. Never connects to a live world.
const {chromium}=require(process.env.CURIOUS_PLAYWRIGHT||'playwright');
const {spawn,spawnSync}=require('node:child_process');
const fs=require('node:fs/promises'),os=require('node:os'),path=require('node:path'),net=require('node:net');
const assert=require('node:assert/strict');
(async()=>{
 const dir=await fs.mkdtemp(path.join(os.tmpdir(),'curious-replay-ui-'));
 const root=path.resolve(__dirname,'..');
 const fixture=String.raw`
import sqlite3,json,gzip,subprocess,sys
from pathlib import Path
p=Path(sys.argv[1])
for shard in range(3):
 db=p/f'history-{shard:06}.sqlite';c=sqlite3.connect(db)
 c.executescript('CREATE TABLE records(id INTEGER PRIMARY KEY,tick INTEGER,at_ms TEXT,kind TEXT,state_json TEXT,metrics_json TEXT); CREATE TABLE decision_traces(record_id INTEGER,entity_id INTEGER,trace_json TEXT); CREATE TABLE events(record_id INTEGER,ordinal INTEGER,payload TEXT);')
 for i in range(3):
  tick=shard*3+i
  e={'id':1,'species':'Herbivore','pos':{'x':i,'y':1},'generation':tick,'parents':[], 'life':{'health':100,'max_health':120,'energy':60,'max_energy':80,'water':30,'max_water':60,'stamina':12,'max_stamina':16,'age':tick},'memory':{},'learner':{'enabled':True,'update_enabled':False,'values':[{'pos':{'x':4,'y':3},'value':80,'visits':7}],'episode':None,'last_episode':None},'genome':{}}
  state={'tick':tick,'bounds':[0,5,0,5],'entities':[e],'resources':[[{'x':4,'y':3},{'Water':{'amount':10}}]],'environment':{'sites':[]}}
  c.execute('INSERT INTO records VALUES(?,?,?,?,?,?)',(i+1,tick,str(1700000000000+tick*1000),'tick',json.dumps(state),json.dumps({'alive':1})))
  c.execute('INSERT INTO decision_traces VALUES(?,?,?)',(i+1,1,json.dumps({'id':1,'reason':'SeekWater','intent':'Wait','outcome':{}})))
 c.commit();c.close()
 if shard==0:
  db.with_suffix('.sqlite.gz').write_bytes(gzip.compress(db.read_bytes()));db.unlink()
 elif shard==1:
  subprocess.run(['zstd','-q',str(db),'-o',str(db)+'.zst'],check=True);db.unlink()
`;
 let child,browser;
 try{
  const p=spawnSync('python3',['-c',fixture,dir],{encoding:'utf8'});assert.equal(p.status,0,p.stderr);
  const before=new Map();for(const f of await fs.readdir(dir))before.set(f,await fs.readFile(path.join(dir,f)));
  const sock=net.createServer();await new Promise(r=>sock.listen(0,'127.0.0.1',r));const port=sock.address().port;await new Promise(r=>sock.close(r));
  const base=`http://127.0.0.1:${port}`;
  child=spawn('python3',[path.join(root,'scripts/archive_replay.py'),'--data-dir',dir,'--port',String(port)],{stdio:'ignore'});
  let ready=false;for(let i=0;i<100;i++){try{if((await fetch(base+'/api/shards')).ok){ready=true;break}}catch{}await new Promise(r=>setTimeout(r,50));}assert.ok(ready,'server did not start');
  for(const method of ['POST','PUT','PATCH','DELETE'])assert.equal((await fetch(base+'/api/record?shard=0&id=1',{method})).status,405,method);
  assert.equal((await fetch(base+'/control')).status,404);
  assert.equal((await fetch(base+'/api/query?sql=DELETE')).status,404);
  browser=await chromium.launch({headless:true,executablePath:process.env.CURIOUS_CHROMIUM,args:['--no-sandbox']});
  const page=await browser.newPage({viewport:{width:1440,height:1100}}),errors=[],methods=[];
  page.on('pageerror',e=>errors.push(e.message));page.on('request',r=>methods.push(r.method()));
  await page.goto(base+'/replay');await page.waitForFunction(()=>document.querySelector('#tick').textContent==='8');
  assert.equal(await page.locator('#map-empty').isVisible(),false,'empty-state overlay on a populated world');
  let budgetPage=true;
  await page.route('**/api/individual-history?*',async route=>{
   if(budgetPage){budgetPage=false;await route.fulfill({status:200,contentType:'application/json',body:JSON.stringify({items:[],has_more:true,next_before:4,budget_limited:true})});}
   else await route.continue();
  });
  await page.locator('#roster button').first().click();assert.match(await page.locator('#individual-title').textContent(),/#1/);
  await page.waitForFunction(()=>document.querySelector('#documentary').textContent.includes('okuma bütçesi nedeniyle kısmi'));
  assert.equal(await page.locator('#older-life').isVisible(),true,'empty budget-limited page was mistaken for shard end');
  await page.locator('#older-life').click();
  await page.waitForFunction(()=>document.querySelector('#documentary').textContent.includes('taranmış 3 kaydın kapsamındadır'));
  assert.match(await page.locator('#documentary').textContent(),/Benzer karşılaşma kuralı/);
  assert.equal(await page.locator('#older-life').isVisible(),true,'bounded older-life pagination was not offered');
  await page.locator('#older-life').click();await page.waitForFunction(()=>document.querySelector('#documentary').textContent.includes('taranmış 6 kaydın kapsamındadır'));
  const lifeText=await page.locator('#documentary').textContent();assert.match(lifeText,/shard 1 \/ kayıt 1/);assert.match(lifeText,/shard 2 \/ kayıt 1/,'record IDs repeated across shards were treated as one identity');
  assert.match(await page.locator('#experience').textContent(),/7 puanlanmış deneme/);
  assert.match(await page.locator('#experience').textContent(),/tablo sabit tutulur/);
  await page.locator('#timeline').fill('0');await page.locator('#timeline').dispatchEvent('input');await page.waitForFunction(()=>document.querySelector('#tick').textContent==='6');
  await page.locator('#beginning').click();await page.waitForFunction(()=>document.querySelector('#tick').textContent==='0');
  await page.locator('#roster button').first().click();
  await page.route('**/api/record?*',async route=>{if(new URL(route.request().url()).searchParams.get('id')==='2')await new Promise(r=>setTimeout(r,200));await route.continue();});
  await page.locator('#timeline').fill('1');await page.locator('#timeline').dispatchEvent('input');
  await page.locator('#timeline').fill('0');await page.locator('#timeline').dispatchEvent('input');
  await page.waitForTimeout(350);assert.equal(await page.locator('#tick').textContent(),'0','stale seek response replaced newer selection');
  await page.unroute('**/api/record?*');
  await page.locator('#rate').selectOption('12');await page.locator('#play').click();
  await page.waitForFunction(()=>Number(document.querySelector('#tick').textContent)>=4);
  if((await page.locator('#play').textContent()).includes('Duraklat'))await page.locator('#play').click();
  assert.match(await page.locator('#individual-title').textContent(),/#1/,'selected individual lost during replay');
  const tick=await page.locator('#tick').textContent();await page.waitForTimeout(400);assert.equal(await page.locator('#tick').textContent(),tick,'pause failed');
  const out=path.resolve(process.env.CURIOUS_REPLAY_OUTPUT||path.join(root,'data/replay-ui-check'));await fs.mkdir(out,{recursive:true});
  await page.screenshot({path:path.join(out,'desktop.png'),fullPage:true});await page.setViewportSize({width:390,height:844});await page.screenshot({path:path.join(out,'mobile.png'),fullPage:true});
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false,'mobile overflow');assert.deepEqual(errors,[]);assert.ok(methods.every(m=>m==='GET'),'UI used a non-GET request');
  assert.deepEqual((await fs.readdir(dir)).sort(),[...before.keys()].sort(),'archive directory changed');for(const [f,b] of before)assert.deepEqual(await fs.readFile(path.join(dir,f)),b,'archive changed: '+f);
  console.log('Replay browser: gzip/zstd/raw, selection, seek, cross-shard play/pause, mobile, GET-only and unchanged archives: OK');
 }finally{
  if(browser)await browser.close();if(child && child.exitCode===null){child.kill();await new Promise(r=>child.once('exit',r));}await fs.rm(dir,{recursive:true,force:true});
 }
})().catch(e=>{console.error(e);process.exitCode=1});
