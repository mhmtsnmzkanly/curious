#!/usr/bin/env python3
"""Out-of-process health alerts. Never changes simulation state or seeds new life."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import urllib.request


def evaluate(previous, health, now, label="çayır", key_prefix="curious-meadow"):
    state = dict(previous)
    state["label"]=label
    state["key_prefix"]=key_prefix
    state['pending'] = list(previous.get('pending', []))
    failures = previous.get('failures', 0) + 1 if not health.get('reachable') else 0
    state['failures'] = failures
    kind = previous.get('incident') if failures and failures < 3 else None
    if failures >= 3:
        kind = 'unreachable'
    elif health.get('storage_error'):
        kind = 'storage_error'
    elif health.get('reachable') and health.get('alive') == 0:
        kind = 'extinction'
    old = previous.get('incident')
    if kind != old:
        state['incident'] = kind
        if kind or old:
            message = {
                'unreachable': 'Kontrol servisine art arda 3 kontrolde ulaşılamadı.',
                'storage_error': 'Kalıcı kayıt hatası var; dünya ilerlemesi duraklatıldı.',
                'extinction': 'Yaşayan birey kalmadı. Otomatik yeniden canlı eklenmedi.',
                None: 'Önceki sorun artık gözlenmiyor; servis tekrar erişilebilir ve yaşam sürüyor.',
            }[kind]
            queue(state, kind or 'recovery', message, health, now)
    budget = health.get('history_budget_bytes', 0)
    ratio = (health.get('history_bytes') or 0) / budget if budget else 0
    if ratio >= .8 and not previous.get('budget_warned'):
        state['budget_warned'] = True
        queue(state, 'storage_budget', 'Kayıt bütçesinin %80’i aşıldı. Bütçede duraklama olmadan arşiv/alan planı gerekiyor.', health, now)
    elif ratio < .7:
        state['budget_warned'] = False
    state['last_check'] = now
    state['last_health'] = health
    return state


def queue(state, event, message, health, now):
    state['pending'].append({
        'title': 'Curious ('+state.get('label','çayır')+'): ' + {'recovery': 'durum düzeldi', 'extinction': 'çayırda yaşam tükendi',
                              'storage_error': 'kayıt hatası', 'unreachable': 'servise ulaşılamıyor',
                              'storage_budget': 'kayıt bütçesi azalıyor'}[event],
        'message': message + f" Tick: {health.get('tick', '?')}. Kontrol zamanı (UTC epoch): {now}.",
        'type': 'success' if event == 'recovery' else 'warning' if event in ('extinction', 'storage_budget') else 'error',
        'target': {'user': 'u_e049630155fc'},
        'idempotency_key': f"{state.get('key_prefix','curious-meadow')}-{event}-{now}",
    })


def save(path, state):
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile('w', dir=path.parent, delete=False) as file:
        json.dump(state, file)
        file.flush()
        os.fsync(file.fileno())
        temporary = Path(file.name)
    temporary.replace(path)
    fd=os.open(path.parent,os.O_RDONLY|os.O_DIRECTORY)
    try:os.fsync(fd)
    finally:os.close(fd)


def dispatch(state,path,notify_cli,token_file):
    for payload in list(state['pending']):
        with tempfile.TemporaryDirectory(prefix='curious-alert-') as temporary:
            message = Path(temporary)/'message.json'
            message.write_text(json.dumps(payload, ensure_ascii=False))
            command = ['node', notify_cli, '-t', token_file, 'send', str(message), '--json', '--timeout', '10']
            try:
                result = subprocess.run(command, capture_output=True, text=True, timeout=45)
                accepted = result.returncode == 0 and json.loads(result.stdout).get('success')
            except (OSError, ValueError, subprocess.TimeoutExpired):
                accepted = False
        if not accepted:
            print('Notify delivery not confirmed; durable pending payload retained for retry.')
            break
        state['pending'].remove(payload)
        save(path, state)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--url', default='http://127.0.0.1:7879')
    parser.add_argument('--state', default='/root/curious/data/meadow-monitor.json')
    parser.add_argument('--notify-cli', default='/root/notify/bin/notify.js')
    parser.add_argument('--token-file', default='/root/notify/token.json')
    parser.add_argument('--label',default='çayır')
    parser.add_argument('--key-prefix',default='curious-meadow')
    args = parser.parse_args()
    path = Path(args.state)
    previous = json.loads(path.read_text()) if path.exists() else {}
    now = int(time.time())
    try:
        with urllib.request.urlopen(args.url.rstrip('/') + '/status', timeout=5) as response:
            health = {'reachable': True, **json.load(response)}
    except (OSError, ValueError):
        health = {'reachable': False}
    state = evaluate(previous, health, now,args.label,args.key_prefix)
    # Persist exact payloads before transport; retries reuse their canonical body/key.
    save(path, state)
    dispatch(state,path,args.notify_cli,args.token_file)
    print(json.dumps({'reachable': health['reachable'], 'incident': state.get('incident'), 'pending': len(state['pending'])}))


if __name__ == '__main__':
    main()
