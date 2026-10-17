#!/usr/bin/env node
// Exercise the browser's actual SQL against SQLite, including shard boundaries.
const {Reader, validId} = require('../web/archive.js');
const assert = require('node:assert/strict');
const {spawnSync} = require('node:child_process');
function query(sql, shard) {
  const python = `import sqlite3,json,sys
c=sqlite3.connect(':memory:')
c.executescript('CREATE TABLE records(id INTEGER PRIMARY KEY,tick INTEGER,kind TEXT,state_json TEXT); CREATE TABLE decision_traces(record_id INTEGER,entity_id INTEGER,trace_json TEXT); CREATE TABLE events(record_id INTEGER,ordinal INTEGER,kind TEXT,payload TEXT);')
shard=int(sys.argv[1])
if shard==0:
 for i in range(1,62):
  events=[{'EntityMated':{'parent1':1,'parent2':2,'child':3}}] if i==1 else []
  t={'tick':i,'id':1,'outcome':{'events':events}}
  body={'entities':[{'id':1,'parents':[],'generation':0},{'id':3,'parents':[1,2],'generation':1}]}
  c.execute('INSERT INTO records VALUES (?,?,?,?)',(i,i,'tick',json.dumps(body)))
  c.execute('INSERT INTO decision_traces VALUES (?,?,?)',(i,1,json.dumps(t)))
elif shard==1:
 c.execute('INSERT INTO records VALUES (1,62,?,?)',('tick','{}'))
 c.execute('INSERT INTO events VALUES (1,0,?,?)',('EntityDied',json.dumps({'EntityDied':{'id':1,'cause':'OldAge'}})))
print(json.dumps({'rows':c.execute(sys.stdin.read()).fetchall(),'truncated':False}))`;
  const p=spawnSync('python3',['-c',python,String(shard)],{input:sql,encoding:'utf8'});
  if(p.status) throw Error(p.stderr);
  return JSON.parse(p.stdout);
}
(async()=>{
  for(const id of [0,-1,1.2,'1;DROP TABLE records',Number.MAX_SAFE_INTEGER+1]) assert.throws(()=>validId(id));
  const api=async(route,body)=>route==='/databases'?[{shard:0},{shard:1}]:query(body.sql,body.shard);
  const r=new Reader(api,1);
  await r.more();
  assert.equal(r.page.length,50); assert.equal(r.lastTick,61); assert.equal(r.death.tick,62);
  assert.equal(r.individual.id,1); assert.equal(r.done,false);
  await r.more(); assert.equal(r.page.length,11); assert.equal(r.done,true);
  assert.equal(r.children.get(3).record,1); assert.equal(r.latestTrace.tick,61);
  const child=new Reader(api,3,r.children.get(3));
  await child.more(); assert.equal(child.individual.id,3); assert.deepEqual(child.parents,[1,2]);
  assert.equal(child.lastTick,1); assert.equal(child.page.length,0);
  const unknown=new Reader(api,999); await unknown.more(); assert.equal(unknown.individual,null); assert.equal(unknown.done,true);
  let calls=0; const bounded=new Reader(async(route)=>route==='/databases'?Array.from({length:12},(_,shard)=>({shard})): (++calls,{rows:[['[]',null,null]],truncated:false}),1);
  await bounded.more(); assert.equal(calls,4); assert.equal(bounded.done,false);
  const truncated=new Reader(async(route)=>route==='/databases'?[{shard:0}]:{rows:[],truncated:true},1);
  await assert.rejects(()=>truncated.more(),/eksik/); assert.equal(truncated.scanned,0);
  console.log('Archive SQL, pagination, boundary death, newborn, missing ID and bounded scans passed.');
})().catch(e=>{console.error(e);process.exitCode=1});
