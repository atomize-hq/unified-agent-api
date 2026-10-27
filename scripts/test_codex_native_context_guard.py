import json
from pathlib import Path
import subprocess
import tempfile
import unittest

import codex_native_context_guard as guard


def quoted(value):
    return '"' + ''.join(f'\\x{byte:02x}' for byte in value.encode()) + '"'


def exec_line(path, args, environment, result="0"):
    array = lambda items: '[' + ', '.join(quoted(s) for s in items) + ']'
    return f'execve({quoted(path)}, {array(args)}, {array(environment)}) = {result}'


class NativeContextGuardTests(unittest.TestCase):
    def test_only_exact_approved_native_private_project_config_addition_passes(self):
        before = 'model = "gpt-5"\n'
        self.assertEqual(guard.config_refusals(before, before.encode()), [])
        approved = (before + guard.PRIVATE_PROJECT_TRUST).encode()
        self.assertEqual(guard.config_refusals(before, approved), [])
        for changed in [approved.replace(b'/pilot/work', b'/host/project'),
                        approved.replace(b'"trusted"', b'"untrusted"'),
                        approved + b'\nextra = true\n',
                        approved.replace(b'gpt-5', b'foreign-model'),
                        approved.replace(b'trust_level =', b'trust_level=')]:
            self.assertEqual(guard.config_refusals(before, changed), ['controlled_config_drift'])

    def test_recorded_native_audit_accounts_for_all_cases_and_grants_no_execution(self):
        from acquire_codex_pilot_protocol import digest
        root = Path(__file__).resolve().parents[1]
        report = json.loads((root/'cli_manifests/codex/reports/0.156.1/pilot-adapter-children.json').read_text())
        pairs = [(c['request']['selector'], c['request']['variant']) for c in report['cases']]
        self.assertEqual(len(pairs), 15)
        self.assertEqual(set(pairs), {(s, v) for s in guard.SELECTORS for v in guard.VARIANTS})
        self.assertIsNone(report['packet_binding'])
        self.assertFalse(report['execution_verified'])
        self.assertFalse(report['context_complete'])
        for name in ['codex_native_context_guard.py', 'observe_codex_adapter_children.py',
                     'observe_codex_pilot_context.py', 'acquire_codex_pilot_protocol.py']:
            self.assertEqual(report['input_sha256']['scripts/'+name], digest((root/'scripts'/name).read_bytes()))
        for case in report['cases']:
            self.assertFalse(case['execution_verified'])
            self.assertFalse(case['adapter']['execution_verified'])
            self.assertFalse(case['context_complete'])
            guard.validate_child(case['child'], case['request']['selector'],
                                 case['request']['variant'], case['child']['producer_environment'])
            expected_refusals = guard.config_refusals(report['expected_config_toml'], case['config_after_toml'].encode())
            trace_refusals = ['incomplete_trace'] if case['unresolved_trace_records'] else []
            self.assertEqual(case['trace_refusals'], trace_refusals)
            self.assertEqual(case['child_boundary_observed'], not trace_refusals)
            self.assertEqual(case['context_refusals'], expected_refusals + trace_refusals)

    def test_initial_context_refuses_foreign_loaders_config_and_symlinks(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            config = 'model = "gpt-5"\n'
            for name, content in guard.initial_files(config).items():
                (root/name).parent.mkdir(parents=True, exist_ok=True)
                (root/name).write_bytes(content)
            guard.validate_initial_root(root, config)
            (root/'work/.codex').mkdir()
            with self.assertRaises(ValueError):
                guard.validate_initial_root(root, config)
            (root/'work/.codex').rmdir()
            (root/'home/config.toml').write_text('model_provider = "foreign"\n')
            with self.assertRaises(ValueError):
                guard.validate_initial_root(root, config)
            (root/'home/config.toml').unlink()
            (root/'home/config.toml').symlink_to(root/'work/.git/config')
            with self.assertRaises(ValueError):
                guard.validate_initial_root(root, config)

    def test_source_binding_refuses_staged_unstaged_and_untracked_changes(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            def git(*args):
                subprocess.run(['git', '-C', temp, *args], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            git('init')
            (root/'source').write_text('original')
            git('add', '.')
            git('-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', 'commit', '-m', 'fixture')
            binding = guard.source_binding(root)
            self.assertEqual(len(binding['commit']), 40)
            for staged in [False, True]:
                (root/'source').write_text('changed')
                if staged:
                    git('add', 'source')
                with self.assertRaises(ValueError):
                    guard.source_binding(root)
                git('restore', '--staged', 'source')
                git('restore', 'source')
            (root/'untracked').write_text('foreign')
            with self.assertRaises(ValueError):
                guard.source_binding(root)
    def test_trace_preserves_exact_quotes_values_and_failed_exec_attempts(self):
        line = exec_line('/bin/codex', ['/bin/codex', '--config', 'approval_policy="never"'], ['PATH=/bin'])
        event = guard.parse_exec(line)
        self.assertEqual(event['argv'][-1], 'approval_policy="never"')
        self.assertEqual(event['environment'], {'PATH': '/bin'})
        self.assertTrue(event['success'])
        self.assertFalse(guard.parse_exec(exec_line('/bin/getconf', ['getconf', 'LONG_BIT'], [], '-1 ENOENT'))['success'])
        for bad in [line.replace('"\\x50\\x41\\x54\\x48\\x3d\\x2f\\x62\\x69\\x6e"', '...'),
                    exec_line('/bin/codex', [], ['A=1', 'A=2']), line[:-3]]:
            with self.assertRaises(ValueError):
                guard.parse_exec(bad)

    def test_actual_child_requires_exact_environment_cwd_and_invocation(self):
        environment = dict(PATH='/bin', HOME='/pilot/home', CODEX_HOME='/pilot/home',
                           TMPDIR='/pilot/tmp', XDG_CONFIG_HOME='/pilot/xdg/config',
                           XDG_DATA_HOME='/pilot/xdg/data', XDG_CACHE_HOME='/pilot/xdg/cache',
                           LANG='C.UTF-8', TZ='UTC', PWD='/pilot/work')
        child = {'executable': '/bin/codex', 'argv': ['/bin/codex', '--config',
                 'sandbox_mode="workspace-write"', 'app-server'], 'cwd': '/pilot/work',
                 'success': True, 'environment': dict(environment, CODEX_BINARY='/bin/codex', RUST_LOG='error')}
        guard.validate_child(child, 'fork-id-stdio', 'interactive', environment)
        for changes in [{'cwd': '/outside'}, {'success': False}, {'executable': '/other'},
                        {'argv': child['argv'] + ['--listen', '0.0.0.0']},
                        {'environment': dict(child['environment'], OPENAI_API_KEY='sentinel')},
                        {'environment': dict(child['environment'], CODEX_HOME='/other')},
                        {'environment': dict(child['environment'], RUST_LOG='')}]:
            with self.assertRaises(ValueError):
                guard.validate_child(dict(child, **changes), 'fork-id-stdio', 'interactive', environment)

        # Exec output must remain under the exact private path grammar; safety intent
        # cannot be widened by an extra flag or a selector from another operation.
        child['argv'] = ['/bin/codex', '--dangerously-bypass-approvals-and-sandbox',
                         'exec', '--color', 'never', '--skip-git-repo-check', '--json',
                         '--output-last-message', '/pilot/tmp/codex_last_message_1_4.txt']
        guard.validate_child(child, 'initial-json', 'external', environment)
        for path in ['/outside/output', '/pilot/tmp/../outside.txt', '/pilot/tmp/arbitrary.txt']:
            changed = dict(child, argv=child['argv'][:-1]+[path])
            with self.assertRaises(ValueError):
                guard.validate_child(changed, 'initial-json', 'external', environment)
        with self.assertRaises(ValueError):
            guard.validate_child(child, 'resume-last-json', 'external', environment)

    def test_only_direct_owned_child_can_establish_spawn_observation(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root/'trace.4').write_text(exec_line('/bin/adapter-observe', ['/bin/adapter-observe'], [])+'\nclone3({flags=CLONE_VM|CLONE_VFORK}, 88) = 5\n')
            (root/'trace.5').write_text('chdir('+quoted('/pilot/work')+') = 0\n'+exec_line('/bin/codex', ['/bin/codex'], [])+'\n')
            child, descendants, unresolved = guard.read_trace(root)
            self.assertEqual(child['pid'], 5)
            self.assertEqual(child['parent_pid'], 4)
            self.assertEqual(descendants, [])
            self.assertEqual(unresolved, [])
            (root/'trace.4').write_text(exec_line('/bin/adapter-observe', ['/bin/adapter-observe'], [])+'\n')
            with self.assertRaises(ValueError):
                guard.read_trace(root)

    def test_incomplete_ownership_or_cwd_records_cannot_establish_a_boundary(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            producer = exec_line('/bin/adapter-observe', ['/bin/adapter-observe'], [])+'\n'
            clone = 'clone3({flags=CLONE_VM|CLONE_VFORK}, 88) = 5\n'
            cwd = 'chdir('+quoted('/pilot/work')+') = 0\n'
            child = exec_line('/bin/codex', ['/bin/codex'], [])+'\n'
            (root/'trace.5').write_text(cwd+child)
            for extra in ['clone3({flags=CLONE_VM',
                          'clone3({flags=CLONE_VM...}, 88) = 6',
                          'clone3({flags=CLONE_VM}, 88) = 6']:
                (root/'trace.4').write_text(producer+clone+extra+'\n')
                with self.subTest(extra=extra), self.assertRaises(ValueError):
                    guard.read_trace(root)
            (root/'trace.4').write_text(producer+clone)
            for transition in ['chdir('+quoted('/outside')+') = ?',
                               'chdir('+quoted('/outside')+' <unfinished ...>',
                               'fchdir(9) = 0']:
                (root/'trace.5').write_text(cwd+transition+'\n'+child)
                with self.subTest(transition=transition), self.assertRaises(ValueError):
                    guard.read_trace(root)
            (root/'trace.5').write_text(cwd+'chdir('+quoted('/absent')+') = -1 ENOENT\n'+child)
            self.assertEqual(guard.read_trace(root)[0]['cwd'], '/pilot/work')

    def test_unknown_detached_syscall_is_retained_as_incomplete_input(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root/'trace.4').write_text(exec_line('/bin/adapter-observe', ['/bin/adapter-observe'], [])+'\nclone3({flags=CLONE_VM|CLONE_VFORK}, 88) = 5\n')
            (root/'trace.5').write_text('chdir('+quoted('/pilot/work')+') = 0\n'+exec_line('/bin/codex', ['/bin/codex'], [])+'\nclone(child_stack=NULL, flags=CLONE_THREAD|CLONE_FS) = 6\n')
            (root/'trace.6').write_text('???( <detached ...>\n')
            child, descendants, unresolved = guard.read_trace(root)
            self.assertEqual(child['pid'], 5)
            self.assertEqual(descendants, [])
            self.assertEqual(unresolved, [{'file': 'trace.6', 'pid': 6, 'line': 1,
                                          'text': '???( <detached ...>'}])


if __name__ == '__main__':
    unittest.main()
