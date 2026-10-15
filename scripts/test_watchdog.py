import unittest
from monitor_watchdog import evaluate
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
if __name__=='__main__':unittest.main()
