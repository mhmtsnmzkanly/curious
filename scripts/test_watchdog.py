import unittest
from monitor_watchdog import evaluate,research_checks
import json,tempfile
from pathlib import Path
from monitor_runtime import evaluate as monitor
class WatchdogTests(unittest.TestCase):
 def test_stale_monitor_alerts_once_and_recovers(self):
  observations={'meadow':{'last_check':1,'limit':180}}
  state=evaluate({},observations,200);self.assertEqual(len(state['pending']),1)
  state=evaluate(state,observations,250);self.assertEqual(len(state['pending']),1)
  observations['meadow']['last_check']=250;state=evaluate(state,observations,251);self.assertEqual(len(state['pending']),2)
  self.assertIsNone(state['incidents']['meadow'])
 def test_stuck_delivery_is_separate_from_monitor_liveness(self):
  state={}
  for now in [10,500,920]:state=evaluate(state,{'x':{'last_check':now,'limit':180,'pending':True}},now)
  self.assertEqual(state['incidents']['x'],'outbox')
 def test_world_alerts_do_not_share_idempotency_keys(self):
  a=monitor({}, {'reachable':True,'alive':0},1,'old','old')
  b=monitor({}, {'reachable':True,'alive':0},1,'new','new')
  self.assertNotEqual(a['pending'][0]['idempotency_key'],b['pending'][0]['idempotency_key'])
 def test_new_campaign_heartbeat_and_notify_outbox_are_independently_watched(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d);campaign=root/'experiments'/'reserve-qualification';campaign.mkdir(parents=True)
   (campaign/'protocol.json').write_text('{}')
   (campaign/'state.json').write_text(json.dumps({'status':'running','last_check':1}))
   (campaign/'notifications.json').write_text('{}')
   checks=research_checks(root)
   self.assertEqual({x[0] for x in checks},{'reserve-qualification','reserve-qualification-notify'})
   (campaign/'state.json').write_text(json.dumps({'status':'qualified','last_check':1}))
   self.assertEqual({x[0] for x in research_checks(root)},{'reserve-qualification-notify'})
if __name__=='__main__':unittest.main()
