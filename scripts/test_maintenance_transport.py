"""Real-Git checks for the disposable maintenance transport boundaries."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

import maintenance_transport as transport


class TransportTests(unittest.TestCase):
    def setUp(self):
        self.previous = Path.cwd()
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        os.chdir(self.temp.name)
        self.addCleanup(os.chdir, self.previous)
        self.git('init', '-q')
        self.git('config', 'user.email', 'fixture@example.invalid')
        self.git('config', 'user.name', 'fixture')
        self.write('tracked', 'baseline')
        self.git('add', 'tracked')
        self.git('commit', '-qm', 'baseline')
        self.inputs = ['cli_manifests/fixture/snapshots/1.2.3/union.json',
                       'cli_manifests/fixture/reports/1.2.3/coverage.any.json']
        self.outputs = self.inputs + ['cli_manifests/fixture/versions/1.2.3.json']
        self.plan = {'input_paths': self.inputs, 'output_paths': self.outputs}
        for path in self.outputs:
            self.write(path, 'acquired')
        self.state = Path(self.temp.name).parent / (Path(self.temp.name).name + '-inputs.json')
        self.delivery = self.state.with_suffix('.delivery.json')
        self.addCleanup(self.state.unlink, missing_ok=True)
        self.addCleanup(self.delivery.unlink, missing_ok=True)

    def git(self, *args):
        return subprocess.check_output(['git', *args], stderr=subprocess.STDOUT)

    def write(self, path, text):
        p = Path(path)
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)

    def prepare(self, publication=True):
        paths = ['docs/agents/lifecycle/fixture-maintenance/governance/maintenance-request.toml',
                 'cli_manifests/fixture/reports/1.2.3/depth-record.json']
        if publication:
            paths += list(transport.PUBLICATION_PATHS)
        for path in paths:
            self.write(path, 'prepared')
        return {'applied': True, 'files': [{'path': p, 'sha256': hashlib.sha256(Path(p).read_bytes()).hexdigest()} for p in paths]}

    def test_stage_before_freeze_and_exact_delivery_excludes_unowned_evidence(self):
        extra = 'cli_manifests/fixture/reports/1.2.3/extra-evidence.json'
        self.write(extra, 'private evidence')
        self.write('untracked', 'leave alone')
        transport.stage_inputs(self.plan, self.state)
        self.assertEqual(set(self.git('diff', '--cached', '--name-only').decode().splitlines()), set(self.inputs))
        report = self.prepare()
        transport.stage_delivery(self.plan, report, self.state, self.delivery, False)
        transport.verify(self.delivery)
        staged = set(self.git('diff', '--cached', '--name-only').decode().splitlines())
        self.assertEqual(staged, set(self.outputs) | {f['path'] for f in report['files']})
        self.assertEqual(Path(extra).read_text(), 'private evidence')
        self.assertNotIn(extra, staged)
        self.assertNotIn('untracked', staged)

    def test_unrelated_staged_changes_are_refused_without_repairing_the_index(self):
        self.write('tracked', 'other work')
        self.git('add', 'tracked')
        before = self.git('ls-files', '--stage', '-z')
        with self.assertRaisesRegex(ValueError, 'staged'):
            transport.stage_inputs(self.plan, self.state)
        self.assertEqual(self.git('ls-files', '--stage', '-z'), before)

    def test_partial_index_and_symlink_inputs_are_refused(self):
        path = self.inputs[0]
        self.git('add', '--', path)
        self.write(path, 'later bytes')
        with self.assertRaisesRegex(ValueError, 'staged'):
            transport.stage_inputs(self.plan, self.state)
        self.git('reset', '-q')
        Path(path).unlink()
        Path(path).symlink_to(Path('tracked').resolve())
        with self.assertRaisesRegex(ValueError, 'regular|symlink'):
            transport.stage_inputs(self.plan, self.state)

    def test_changed_acquisition_or_preparation_bytes_refuse_delivery(self):
        transport.stage_inputs(self.plan, self.state)
        report = self.prepare()
        self.write(self.inputs[0], 'changed after freeze')
        with self.assertRaisesRegex(ValueError, 'changed'):
            transport.stage_delivery(self.plan, report, self.state, self.delivery, False)
        self.write(self.inputs[0], 'acquired')
        self.write(report['files'][0]['path'], 'changed after preparation')
        with self.assertRaisesRegex(ValueError, 'changed'):
            transport.stage_delivery(self.plan, report, self.state, self.delivery, False)

    def test_missing_runtime_or_partial_publication_refuses_delivery(self):
        transport.stage_inputs(self.plan, self.state)
        report = self.prepare()
        Path(transport.PUBLICATION_PATHS[2]).unlink()
        with self.assertRaisesRegex(ValueError, 'changed|missing'):
            transport.stage_delivery(self.plan, report, self.state, self.delivery, False)
        report['files'] = report['files'][:-1]
        with self.assertRaisesRegex(ValueError, 'publication'):
            transport.stage_delivery(self.plan, report, self.state, self.delivery, False)

    def test_verify_refuses_later_index_or_worktree_changes(self):
        transport.stage_inputs(self.plan, self.state)
        report = self.prepare()
        transport.stage_delivery(self.plan, report, self.state, self.delivery, False)
        self.write(self.outputs[0], 'changed after audit')
        with self.assertRaisesRegex(ValueError, 'changed'):
            transport.verify(self.delivery)
        self.write(self.outputs[0], 'acquired')
        self.write('tracked', 'unexpected staged')
        self.git('add', 'tracked')
        with self.assertRaisesRegex(ValueError, 'staged'):
            transport.verify(self.delivery)

    def test_initial_packet_uses_only_prepared_paths_and_read_only_preserves_index(self):
        report = self.prepare(False)
        transport.stage_delivery(None, report, None, self.delivery, False)
        transport.verify(self.delivery)
        self.assertEqual(set(self.git('diff', '--cached', '--name-only').decode().splitlines()), {f['path'] for f in report['files']})
        self.git('reset', '-q')
        for path in transport.PUBLICATION_PATHS:
            self.write(path, 'publication')
        before = self.git('ls-files', '--stage', '-z')
        self.delivery.unlink()
        transport.stage_delivery(self.plan, None, None, self.delivery, True)
        self.assertEqual(self.git('ls-files', '--stage', '-z'), before)

    def test_initial_delivery_refuses_a_partial_owned_index(self):
        report = self.prepare(False)
        path = report['files'][0]['path']
        self.git('add', '--', path)
        self.write(path, 'later preparation bytes')
        report['files'][0]['sha256'] = hashlib.sha256(Path(path).read_bytes()).hexdigest()
        before = self.git('ls-files', '--stage', '-z')
        with self.assertRaisesRegex(ValueError, 'staged'):
            transport.stage_delivery(None, report, None, self.delivery, False)
        self.assertEqual(before, self.git('ls-files', '--stage', '-z'))

    def test_tracked_input_drift_after_staging_refuses_delivery_and_commit(self):
        transport.stage_inputs(self.plan, self.state)
        report = self.prepare()
        self.write('tracked', 'changed policy/source input')
        with self.assertRaisesRegex(ValueError, 'tracked inputs'):
            transport.stage_delivery(self.plan, report, self.state, self.delivery, False)
        self.write('tracked', 'baseline')
        transport.stage_delivery(self.plan, report, self.state, self.delivery, False)
        self.write('tracked', 'changed after audit')
        with self.assertRaisesRegex(ValueError, 'tracked inputs'):
            transport.verify(self.delivery)

    def test_existing_or_symlinked_scratch_state_cannot_overwrite_workspace_files(self):
        self.state.symlink_to(Path('tracked').resolve())
        with self.assertRaises(FileExistsError):
            transport.stage_inputs(self.plan, self.state)
        self.assertEqual(Path('tracked').read_text(), 'baseline')

    def test_failed_or_dry_run_preparation_cannot_be_delivered(self):
        with self.assertRaisesRegex(ValueError, 'applied'):
            transport.stage_delivery(None, {'applied': False, 'files': []}, None, self.delivery, False)
        self.assertFalse(self.delivery.exists())


if __name__ == '__main__':
    unittest.main()
