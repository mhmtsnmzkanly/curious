import unittest
from report_soak import evaluate
class SoakTests(unittest.TestCase):
 def summary(self,span,count=100):return {'first':{'at':1},'last':{'at':span+1,'status':{'tick':1000,'alive':8,'history_bytes':10000}},'count':count,'rss_min':9000,'rss_max':10000,'fd_max':9,'pids':[1]}
 def test_accelerated_ticks_are_not_real_days(self):
  s=evaluate({},self.summary(100),101);self.assertEqual(s['pending'],[])
 def test_actual_day_is_reported_once(self):
  summary=self.summary(86400);s=evaluate({},summary,86401);self.assertEqual(s['reported_days'],[1]);s=evaluate(s,summary,86402);self.assertEqual(len(s['pending']),1)
 def test_missing_coverage_or_stale_sampler_cannot_claim_milestone(self):
  self.assertEqual(evaluate({},self.summary(86400,2),86401)['pending'],[])
  self.assertEqual(evaluate({},self.summary(86400),90000)['pending'],[])
if __name__=='__main__':unittest.main()
