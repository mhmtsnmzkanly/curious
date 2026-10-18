#!/usr/bin/env python3
"""Frozen, gated research workflow. Executes experiments, never invents fixes."""
import argparse
import fcntl
import hashlib
import json
from pathlib import Path
import subprocess
import socket
import time
import urllib.request

from monitor_runtime import dispatch, save
from research_protocol import TRAINING_SEEDS, policy_flags, protocol, freeze, reused_stage, progress_text, progress_block


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def notices(config, key=None, title=None, message=None, details=None):
    root = Path(config['output'])
    root.mkdir(parents=True, exist_ok=True)
    path = root / 'notifications.json'
    with (root / '.notifications.lock').open('a') as guard:
        fcntl.flock(guard, fcntl.LOCK_EX)
        state = json.loads(path.read_text()) if path.exists() else {'pending': [], 'enqueued': [], 'history': []}
        if key and key not in state['enqueued']:
            payload = {'title': title, 'message': message, 'type': 'error' if key.startswith(('rejected-', 'infrastructure-')) or key == 'calibration-exhausted' else 'info',
                       'target': {'user': 'u_e049630155fc'},
                       'idempotency_key': 'curious-research-' + hashlib.sha256((config['revision'] + config['scenario_sha256'] + config['output']).encode()).hexdigest()[:16] + '-' + key}
            if config.get('page_id'):
                payload['page_id'] = config['page_id']
            elif details:
                payload['panel'] = {'title': title, 'mode': 'static', 'manifest': {'version': 1, 'blocks': [
                    {'id': 'report', 'type': 'markdown', 'content': message + '\n\n```json\n' + json.dumps(details, indent=2, ensure_ascii=False) + '\n```'}]}}
            state['pending'].append(payload)
            state['enqueued'].append(key)
            state.setdefault('history', []).append({'key': key, 'title': title, 'message': message, 'details': details})
        state['last_check'] = int(time.time())
        save(path, state)
        if config.get('page_id'):
            manifest = json.loads(Path(config['page_manifest']).read_text())
            pipeline = root / 'state.json'
            if pipeline.exists():
                manifest['blocks'].append({'id': 'current-stage', 'type': 'markdown',
                    'content': progress_text(root, json.loads(pipeline.read_text()))})
                manifest['blocks'].append(progress_block(root, json.loads(pipeline.read_text())))
            for item in state.get('history', []):
                manifest['blocks'].append({'id': 'research-' + item['key'], 'type': 'markdown',
                    'content': '### ' + item['title'] + '\n\n' + item['message'] + ('\n\n```json\n' + json.dumps(item['details'], indent=2, ensure_ascii=False) + '\n```' if item['details'] else '')})
            update = {'title': 'Curious — av–avcı araştırması ve doğrulama', 'manifest': manifest}
            fingerprint = hashlib.sha256(json.dumps(update, sort_keys=True).encode()).hexdigest()
            if fingerprint != state.get('page_hash'):
                page_update = root / 'page-update.json'
                save(page_update, update)
                state['page_update_pending'] = True
                save(path, state)
                try:
                    call = subprocess.run(['node', '/root/notify/bin/notify.js', '-t', '/root/notify/token.json',
                                           'page', 'update', config['page_id'], str(page_update), '--json', '--timeout', '10'],
                                          capture_output=True, text=True, timeout=20)
                    if call.returncode == 0:
                        state['page_hash'] = fingerprint
                        state['page_update_pending'] = False
                except (OSError, subprocess.TimeoutExpired):
                    pass
                save(path, state)
            pipeline = root / 'state.json'
            if pipeline.exists():
                status = json.loads(pipeline.read_text()).get('status')
                wanted = 'failed' if status in ('rejected', 'error') else 'completed' if status == 'qualified' else 'running'
                if wanted != state.get('page_task_status'):
                    state['task_update_pending'] = True
                    save(path, state)
                    try:
                        call = subprocess.run(['node', '/root/notify/bin/notify.js', '-t', '/root/notify/token.json',
                                               'page', 'task-status', config['page_id'], wanted, '--json', '--timeout', '10'],
                                              capture_output=True, text=True, timeout=20)
                        if call.returncode == 0:
                            state['page_task_status'] = wanted
                            state['task_update_pending'] = False
                    except (OSError, subprocess.TimeoutExpired):
                        pass
                    save(path, state)
        dispatch(state, path, '/root/notify/bin/notify.js', '/root/notify/token.json')


def live_deploy(config, source, qualification):
    """First deployment only after every gate; existing worlds are never upgraded."""
    if policy_flags(config):
        raise ValueError('Experimental policy deployment needs explicit runtime policy support')
    root = Path(config['project_root'])
    engine = Path(config['engine'])
    if digest(engine) != config['engine_sha256']:
        raise ValueError('Frozen engine changed')
    scenario = Path(config['scenario'])
    target = root / 'data/coexistence-scenario.json'
    if target.exists() and target.read_bytes() != scenario.read_bytes():
        raise ValueError('Refusing to replace another deployed scenario')
    if not target.exists():
        with target.open('xb') as file:
            file.write(scenario.read_bytes())
    link = root / 'bin/curious-coexistence'
    if link.exists() and link.resolve() != engine.resolve():
        raise ValueError('Another coexistence binary is already installed')
    if not link.exists():
        link.symlink_to(engine)
    marker = root / 'data/coexistence-deployment.json'
    if (root / 'data/coexistence').exists() and not marker.exists():
        raise ValueError('Existing world lacks deployment identity; refusing takeover')
    if marker.exists() and json.loads(marker.read_text())['engine_sha256'] != config['engine_sha256']:
        raise ValueError('Existing world belongs to another engine')
    if not marker.exists():
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 7882))
    seed = next(r['seed'] for r in qualification['seed_results'] if r['passed'])
    runtime = root / 'data/coexistence-runtime.env'
    encoded = f"CURIOUS_SEED={seed}\nCURIOUS_PAGE={config.get('page_id', 'p_6c6647a23f563ace')}\n"
    if runtime.exists() and runtime.read_text() != encoded:
        raise ValueError('Another runtime seed is already configured')
    runtime.write_text(encoded)
    runtime.chmod(0o600)
    for name in ['curious-coexistence.service', 'curious-coexistence-monitor.service',
                 'curious-coexistence-monitor.timer', 'curious-coexistence-sampler.service',
                 'curious-coexistence-sampler.timer', 'curious-coexistence-soak-report.service',
                 'curious-coexistence-soak-report.timer']:
        Path('/etc/systemd/system', name).write_bytes((source / 'deploy' / name).read_bytes())
    subprocess.run(['systemctl', 'daemon-reload'], check=True)
    if not marker.exists():
        save(marker, {'revision': config['revision'], 'engine_sha256': config['engine_sha256'], 'seed': seed,
                  'scenario_sha256': digest(scenario), 'qualified_at': int(time.time()),
                  'research': str(Path(config['output']).resolve())})
    subprocess.run(['systemctl', 'enable', '--now', 'curious-coexistence.service'], check=True)
    for _ in range(30):
        try:
            with urllib.request.urlopen('http://127.0.0.1:7882/status', timeout=3) as response:
                health = json.load(response)
            if health.get('storage_error'):
                raise RuntimeError('New world storage error')
            break
        except OSError:
            time.sleep(1)
    else:
        raise RuntimeError('New world did not become reachable')
    subprocess.run(['systemctl', 'start', 'curious-coexistence-monitor.service', 'curious-coexistence-sampler.service'], check=True)
    subprocess.run(['systemctl', 'enable', '--now', 'curious-coexistence-monitor.timer',
                    'curious-coexistence-sampler.timer', 'curious-coexistence-soak-report.timer'], check=True)
    return health


def run(config):
    source = Path(__file__).resolve().parents[1]
    actual = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=source, text=True).strip()
    if actual != config['revision'] or digest(config['probe']) != config['probe_sha256']:
        raise ValueError('Research source or probe identity changed')
    if digest(config['scenario']) != config['scenario_sha256']:
        raise ValueError('Research initial condition changed')
    root = Path(config['output'])
    root.mkdir(parents=True, exist_ok=True)
    declaration = protocol(config)
    if config.get('engine_source'):
        actual_source = subprocess.check_output(['git', 'rev-parse', 'HEAD:src', 'HEAD:Cargo.toml', 'HEAD:Cargo.lock'], cwd=source, text=True).splitlines()
        if actual_source != config['engine_source']:
            raise ValueError('Research engine source differs from the compiled baseline')
    freeze(root / 'protocol.json', declaration)
    path = root / 'state.json'
    state = json.loads(path.read_text()) if path.exists() else {'completed': []}
    if state.get('status') in ('rejected', 'qualified', 'observing'):
        notices(config)
        return

    def heartbeat(**fields):
        state.update(fields)
        state['last_check'] = int(time.time())
        save(path, state)

    def stage(name, scenario, seeds, ticks, mode='fixed', shock=False, gate=True):
        if digest(config['probe']) != config['probe_sha256'] or digest(config['scenario']) != config['scenario_sha256']:
            raise ValueError('Frozen campaign artifacts changed between stages')
        output = root / name
        heartbeat(status='running', stage=name)
        reused = reused_stage(config, name, scenario, seeds, ticks, mode, shock)
        if reused:
            result, evidence = reused
            save(root / (name + '-reused.json'), evidence)
        else:
            command = ['/usr/bin/python3', '-u', str(source / 'scripts/validate_coexistence.py'),
                       '--probe', config['probe'], '--scenario', str(scenario), '--output', str(output),
                       '--seeds', ','.join(map(str, seeds)), '--ticks', str(ticks), '--mode', mode,
                       '--jobs', '2', '--timeout', '14400']
            command += policy_flags(config)
            if output.exists():
                command.append('--resume')
            if shock:
                command += ['--shock-at', '10000', '--shock-duration', '1000', '--shock-percent', '50']
            with (root / (name + '.log')).open('a') as log:
                child = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
                try:
                    while True:
                        heartbeat()
                        try:
                            child.wait(timeout=20)
                            break
                        except subprocess.TimeoutExpired:
                            pass
                    if child.returncode:
                        raise RuntimeError(f'{name}: validation process failed ({child.returncode})')
                finally:
                    if child.poll() is None:
                        child.terminate()
                        child.wait(timeout=30)
            result = json.loads((output / 'summary.json').read_text())
        if result['accepted'] and ticks >= 100000 and len(seeds) >= 30 and not result.get('qualified'):
            raise ValueError('Long-run result lacks formal qualification')
        if name not in state['completed']:
            state['completed'].append(name)
        heartbeat(last_result=result)
        if not result['accepted']:
            if gate:
                heartbeat(status='rejected')
                notices(config, 'rejected-' + name, 'Curious: doğrulama eşiği geçilmedi',
                    f"{name}: {result['passed']}/{result['planned']} geçti; gereken {result['required']}. "
                    'Sonraki aşama ve canlı dağıtım başlatılmadı. Veriler korundu; motorun yeni kod üretmesi beklenmez.', result)
            return None
        return result

    try:
        research_seeds = TRAINING_SEEDS
        held_out = declaration['held_out_seeds']
        selection = config.get('candidates')
        candidates = selection or [{'name': 'fixed', 'scenario': config['scenario'], 'sha256': config['scenario_sha256']}]
        selected = None
        for candidate in candidates:
            scenario = Path(candidate['scenario'])
            if digest(scenario) != candidate['sha256']:
                raise ValueError('Predeclared calibration candidate changed')
            tag = candidate['name']
            heartbeat(candidate=tag)
            if selection:
                result = stage('screen-' + tag, scenario, [2, 5], 5000, gate=False)
                if result is None:
                    continue
            result = stage('canary-' + tag, scenario, research_seeds, 20000, gate=not selection)
            if result is None:
                if selection:
                    continue
                return
            sensitivity = {}
            robust = True
            for field in ['max_energy', 'reproduction_interval']:
                for percent in [90, 110]:
                    variant = json.loads(scenario.read_text())
                    for founder in variant['founders']:
                        if founder['species'] != 'Carnivore':
                            continue
                        life = founder['life']
                        before_capacity = life['max_energy']
                        life[field] = life[field] * percent // 100
                        if field == 'max_energy':
                            life['energy'] = life['energy'] * life['max_energy'] // before_capacity
                    encoded = json.dumps(variant, indent=2) + '\n'
                    variant_path = root / f'{tag}-{field}-{percent}.json'
                    if variant_path.exists() and variant_path.read_text() != encoded:
                        raise ValueError('Sensitivity initial condition changed')
                    variant_path.write_text(encoded)
                    name = f'sensitivity-{tag}-{field}-{percent}'
                    result = stage(name, variant_path, research_seeds[:4], 20000, gate=not selection)
                    if result is None:
                        robust = False
                        break
                    sensitivity[name] = result
                if not robust:
                    break
            if robust:
                selected = candidate
                break
            if not selection:
                return
        if selected is None:
            heartbeat(status='rejected', stage='calibration-exhausted')
            notices(config, 'calibration-exhausted', 'Curious: taramada dayanıklı aday çıkmadı',
                    'Önceden tanımlı adayların hiçbiri kısa koşu, sekiz seedlik süreklilik ve yakın parametre kapılarını birlikte geçemedi. '
                    'Görülmemiş 30 seed kullanılmadı; yeni dünya dağıtılmadı. Kayıtlar sonraki teşhis için saklandı.', {'completed': state['completed']})
            return
        config = {**config, 'scenario': selected['scenario'], 'scenario_sha256': selected['sha256']}
        scenario = Path(config['scenario'])
        heartbeat(selected=selected)
        notices(config, 'sensitivity', 'Curious: yakın koşullar da sürdü',
                'Enerji kapasitesi ve üreme aralığında ayrı ayrı ±%10 değişiklikler geçti. Şimdi görülmemiş 30 seed × 100.000 tick sınanıyor.', sensitivity)
        result = stage('qualification-fixed', scenario, held_out, 100000)
        if result is None:
            return
        notices(config, 'qualification-fixed', 'Curious: uzun kurallı denge testi geçti',
                '30 seed × 100.000 tick için 27/30 eşiği geçti. Şimdi kaynak şoku, öğrenme ve mutasyon kontrolleri çalışıyor. Gerçek gün/hafta ölçümü ayrı.', result)
        result = stage('resource-shock', scenario, research_seeds, 20000, shock=True)
        if result is None:
            return
        for mode in ['learning', 'mutation', 'both']:
            result = stage('canary-' + mode, scenario, research_seeds, 20000, mode)
            if result is None:
                return
        notices(config, 'learning-mutation', 'Curious: kaynak şoku ve öğrenme kontrolleri geçti',
                'Yenilenmenin geçici %50 azalması; öğrenme, mutasyon ve ikisinin birlikte olduğu kısa karşılaştırmalar geçti. Son olarak ikisi birlikte 30 seed × 100.000 tick sınanıyor.', result)
        result = stage('qualification-both', scenario, held_out, 100000, 'both')
        if result is None:
            return
        if config.get('deploy_after_pass'):
            health = live_deploy(config, source, result)
            heartbeat(status='observing', stage='real-time', health=health)
            notices(config, 'observing', 'Curious: doğrulanan karma dünya gözleme alındı',
                    'Bütün tanımlı kapılar geçti. Ayrı 7882 dünyası 0,1 tick/s çalışıyor; tür kaybı ve kayıt hatası izleniyor. Gerçek 1/7/30 gün gözlemi başladı; henüz bu süreler geçti denmiyor.', {'selected': state.get('selected'), 'qualification': result})
        else:
            heartbeat(status='qualified', stage='ready-to-observe')
    except Exception as error:
        heartbeat(status='error', error=f'{type(error).__name__}: {error}')
        notices(config, 'infrastructure-' + hashlib.sha256(str(error).encode()).hexdigest()[:12], 'Curious: deney çalıştırıcısında hata',
                str(error)[:1000] + '. Sonuçlar korunuyor; başarısız teknik işlem biyolojik başarı sayılmadı.')
        raise


def run_guarded(config):
    """Record identity/preflight errors too, so the independent watchdog sees them."""
    try:
        return run(config)
    except Exception as error:
        root = Path(config['output'])
        root.mkdir(parents=True, exist_ok=True)
        path = root / 'state.json'
        try:
            state = json.loads(path.read_text())
        except (OSError, ValueError):
            state = {'completed': []}
        state.update(status='error', error=f'{type(error).__name__}: {error}', last_check=int(time.time()))
        save(path, state)
        notices(config, 'infrastructure-' + hashlib.sha256(str(error).encode()).hexdigest()[:12],
                'Curious: deney çalıştırıcısında hata',
                str(error)[:1000] + '. Sonuçlar korunuyor; teknik hata biyolojik kabul sayılmadı.')
        raise


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', required=True)
    parser.add_argument('--notifications-only', action='store_true')
    args = parser.parse_args()
    config = json.loads(Path(args.config).read_text())
    if args.notifications_only:
        notices(config)
    else:
        with (Path(config['output']) / '.pipeline.lock').open('a') as guard:
            fcntl.flock(guard, fcntl.LOCK_EX | fcntl.LOCK_NB)
            run_guarded(config)
