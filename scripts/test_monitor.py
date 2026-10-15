import unittest
from monitor_runtime import evaluate

class MonitorTests(unittest.TestCase):
    def test_unreachable_requires_three_checks_and_deduplicates(self):
        state={}
        for n in range(3):state=evaluate(state,{'reachable':False},n)
        self.assertEqual(len(state['pending']),1)
        state=evaluate(state,{'reachable':False},4)
        self.assertEqual(len(state['pending']),1)
        state=evaluate(state,{'reachable':True,'alive':8,'tick':50},5)
        self.assertEqual(len(state['pending']),2)
        self.assertIsNone(state['incident'])
    def test_extinction_and_recovery_are_explicit(self):
        state=evaluate({}, {'reachable':True,'alive':0,'tick':100},1)
        self.assertEqual(state['incident'],'extinction')
        state=evaluate(state,{'reachable':True,'alive':0,'tick':101},2)
        self.assertEqual(len(state['pending']),1)
    def test_storage_error_takes_priority_over_extinction(self):
        state=evaluate({}, {'reachable':True,'alive':0,'storage_error':'full'},1)
        self.assertEqual(state['incident'],'storage_error')
    def test_budget_alert_is_not_repeated_each_minute(self):
        health={'reachable':True,'alive':8,'history_bytes':85,'history_budget_bytes':100}
        state=evaluate({},health,1);state=evaluate(state,health,2)
        self.assertEqual(len(state['pending']),1)

    def test_missing_disk_estimate_does_not_hide_a_storage_error(self):
        state=evaluate({}, {'reachable':True,'alive':8,'storage_error':'read error','history_bytes':None,'history_budget_bytes':100},1)
        self.assertEqual(state['incident'],'storage_error')

if __name__=='__main__':unittest.main()
