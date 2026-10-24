"""Behavior tests for Make wrappers and the real-Git planning-worktree state machine."""
from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[1]
MAKE = shutil.which("make") or "make"


def run(args, cwd, *, check=True, env=None):
    result = subprocess.run(args, cwd=cwd, text=True, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, env=env)
    if check and result.returncode:
        raise AssertionError(f"{args} failed ({result.returncode})\n{result.stdout}\n{result.stderr}")
    return result


def git(cwd, *args, check=True):
    return run(["git", *args], cwd, check=check)


class PlanningFixture:
    def __init__(self, base: Path, *, spaces=False):
        name = "source checkout with spaces" if spaces else "source"
        self.source = base / name
        self.origin = base / "origin.git"
        self.source.mkdir(parents=True)
        git(self.source, "init", "-b", "main")
        git(self.source, "config", "user.name", "Fixture User")
        git(self.source, "config", "user.email", "fixture@example.invalid")
        (self.source / "tracked.txt").write_text("source base\n")
        git(self.source, "add", "tracked.txt")
        git(self.source, "commit", "-m", "source base")
        run(["git", "init", "--bare", str(self.origin)], base)
        git(self.source, "remote", "add", "origin", str(self.origin))
        git(self.source, "push", "-u", "origin", "main")
        shutil.copy2(REPO / "Makefile", self.source / "Makefile")
        shutil.copy2(REPO / ".gitignore", self.source / ".gitignore")
        (self.source / "scripts").mkdir()
        shutil.copy2(REPO / "scripts/planning-worktree.py", self.source / "scripts/planning-worktree.py")
        git(self.source, "add", "Makefile", ".gitignore", "scripts/planning-worktree.py")
        git(self.source, "commit", "-m", "fixture build tooling")
        self.index_before = (self.source / ".git/index").read_bytes()
        self.head_before = git(self.source, "rev-parse", "HEAD").stdout.strip()

    def make(self, *args, env=None, check=True):
        return run([MAKE, "--no-print-directory", *args], self.source, check=check, env=env)

    def source_unchanged(self):
        self.assert_source_unchanged()

    def assert_source_unchanged(self):
        assert git(self.source, "rev-parse", "HEAD").stdout.strip() == self.head_before
        assert (self.source / ".git/index").read_bytes() == self.index_before
        assert not git(self.source, "status", "--porcelain").stdout

    def plan(self, path=".worktrees/planning", check=True):
        return self.make("planning", f"PYTHON={shutil.which('python3') or 'python3'}", check=check)

    def planning(self):
        return self.source / ".worktrees/planning"


class BuildToolTests(unittest.TestCase):
    def test_make_targets_forward_toolchain_and_failures(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            log = root / "cargo.log"
            fake = root / "fake-cargo"
            fake.write_text("#!/bin/sh\nset -eu\nprintf '%s\\n' \"$*\" >> \"$CARGO_LOG\"\n[ \"${FAKE_FAIL:-}\" != 1 ] || exit 23\ncase \" $* \" in *' build '*) mode=debug; case \" $* \" in *' --release '*) mode=release ;; esac; mkdir -p \"$CARGO_TARGET_DIR/$mode\"; printf '%s\\n' \"$mode\" > \"$CARGO_TARGET_DIR/$mode/vlajmaji\" ;; esac\n")
            fake.chmod(0o755)
            env = os.environ | {"CARGO_LOG": str(log), "PATH": str(root) + os.pathsep + os.environ["PATH"], "CARGO_TARGET_DIR": str(root / "cargo artifacts")}
            for target, fragment in (("build", "build --workspace --locked"),
                                     ("build-release", "build --workspace --release --locked"),
                                     ("test-default", "test --workspace --locked"),
                                     ("test-no-default", "test --locked -p vlajmaji-search --no-default-features"),
                                     ("test-all-features", "test --workspace --all-features --locked"),
                                     ("lint", "clippy --workspace --all-targets --locked -- -D warnings"),
                                     ("docs", "doc --workspace --locked --no-deps")):
                result = run([MAKE, "--no-print-directory", f"CARGO={fake}", "RUST_TOOLCHAIN=1.98.0", f"INSTALL_DIR={root / 'installed tools'}", target], REPO, env=env)
                self.assertEqual(result.returncode, 0)
                self.assertIn("+1.98.0 " + fragment, log.read_text())
                if target in ("build", "build-release"):
                    expected = "release\n" if target == "build-release" else "debug\n"
                    self.assertEqual((root / "installed tools" / "vlajmaji").read_text(), expected)
            before = len(log.read_text().splitlines())
            missing = run([MAKE, "--no-print-directory", f"CARGO={fake}", "publish-one"], REPO, env=env, check=False)
            self.assertNotEqual(missing.returncode, 0)
            self.assertEqual(len(log.read_text().splitlines()), before)
            for target, expected in (("publish", "publish --workspace --registry crates-io --locked"),
                                     ("publish-dry-run", "publish --workspace --registry crates-io --dry-run --locked"),
                                     ("publish-one", "publish -p vlajmaji-search --registry crates-io --locked")):
                args = [MAKE, "--no-print-directory", f"CARGO={fake}", target]
                if target == "publish-one":
                    args.append("CRATE=vlajmaji-search")
                run(args, REPO, env=env)
                self.assertIn("+1.98.0 " + expected, log.read_text())
            failed = run([MAKE, "--no-print-directory", f"CARGO={fake}", f"INSTALL_DIR={root / 'installed tools'}", "build"], REPO, env=env | {"FAKE_FAIL": "1"}, check=False)
            self.assertNotEqual(failed.returncode, 0)
            self.assertEqual((root / "installed tools" / "vlajmaji").read_text(), "release\n")
            self.assertNotIn("Build complete", failed.stdout)

    def test_install_copies_selected_mode_and_reports_missing_artifacts(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            shutil.copy2(REPO / "Makefile", root / "Makefile")
            target = root / "cargo artifacts"
            for mode in ["debug", "release"]:
                directory = target / mode
                directory.mkdir(parents=True)
                binary = directory / "vlajmaji"
                binary.write_text("#!/bin/sh\necho " + mode + "\n")
                binary.chmod(0o755)
            args = [MAKE, "--no-print-directory", f"CARGO_TARGET_DIR={target}"]
            run(args + ["install"], root)
            installed = root / "bin" / "vlajmaji"
            self.assertEqual(run([str(installed)], root).stdout, "debug\n")
            self.assertEqual(installed.read_bytes(), (target / "debug" / "vlajmaji").read_bytes())
            custom = root / "custom install"
            run(args + ["install", "MODE=release", f"INSTALL_DIR={custom}"], root)
            self.assertEqual(run([str(custom / "vlajmaji")], root).stdout, "release\n")
            (target / "debug" / "vlajmaji").unlink()
            failed = run(args + ["install"], root, check=False)
            self.assertNotEqual(failed.returncode, 0)
            self.assertNotIn("Binaries available", failed.stdout)
            invalid = run(args + ["build", "MODE=invalid"], root, check=False)
            self.assertNotEqual(invalid.returncode, 0)

    def test_help_does_not_require_build_tools(self):
        result = run([MAKE, "--no-print-directory", "help"], REPO)
        self.assertIn("planning", result.stdout)
        self.assertIn("publish-dry-run", result.stdout)

    def test_create_empty_root_push_idempotence_and_preserve_source(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp), spaces=True)
            first = f.make("planning")
            work = f.planning()
            commit = git(work, "rev-parse", "HEAD").stdout.strip()
            self.assertEqual(len(git(work, "rev-list", "--parents", "-n", "1", commit).stdout.split()), 1)
            self.assertEqual(git(work, "ls-tree", "-r", "--name-only", "HEAD").stdout, "")
            self.assertEqual(git(f.origin, "show-ref", "refs/heads/planning").stdout.split()[0], commit)
            self.assertIn("created empty planning root", first.stdout)
            self.assertEqual(f.make("planning").returncode, 0)
            self.assertEqual(git(work, "rev-parse", "HEAD").stdout.strip(), commit)
            f.assert_source_unchanged()

    def test_remote_branch_without_local_branch_attaches(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            clone = Path(tmp) / "seed"
            run(["git", "clone", str(f.origin), str(clone)], Path(tmp))
            git(clone, "config", "user.name", "Fixture User")
            git(clone, "config", "user.email", "fixture@example.invalid")
            git(clone, "checkout", "--orphan", "planning")
            git(clone, "rm", "-rf", ".", check=False)
            git(clone, "commit", "--allow-empty", "-m", "remote plan")
            git(clone, "push", "origin", "planning")
            f.make("planning")
            self.assertEqual(git(f.planning(), "branch", "--show-current").stdout.strip(), "planning")
            self.assertEqual(git(f.source, "show-ref", "--verify", "--quiet", "refs/heads/planning", check=False).returncode, 0)
            f.assert_source_unchanged()

    def test_remote_update_fast_forwards_clean_planning_worktree(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            f.make("planning")
            clone = Path(tmp) / "remote update"
            run(["git", "clone", str(f.origin), str(clone)], Path(tmp))
            git(clone, "config", "user.name", "Fixture User")
            git(clone, "config", "user.email", "fixture@example.invalid")
            git(clone, "checkout", "planning")
            (clone / "later.txt").write_text("later\n")
            git(clone, "add", "later.txt")
            git(clone, "commit", "-m", "remote update")
            git(clone, "push", "origin", "planning")
            f.make("planning")
            self.assertTrue((f.planning() / "later.txt").exists())
            f.assert_source_unchanged()

    def test_local_ahead_preserved_without_push(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            f.make("planning")
            work = f.planning()
            (work / "local.txt").write_text("local\n")
            git(work, "add", "local.txt")
            git(work, "commit", "-m", "local ahead")
            before = git(work, "rev-parse", "HEAD").stdout.strip()
            f.make("planning")
            self.assertEqual(git(work, "rev-parse", "HEAD").stdout.strip(), before)
            f.assert_source_unchanged()

    def test_existing_local_branch_without_remote_is_attached_and_pushed(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            git(f.source, "branch", "planning", "HEAD")
            expected = git(f.source, "rev-parse", "refs/heads/planning").stdout.strip()
            f.make("planning")
            self.assertEqual(git(f.planning(), "rev-parse", "HEAD").stdout.strip(), expected)
            self.assertEqual(git(f.origin, "show-ref", "refs/heads/planning").stdout.split()[0], expected)

    def test_dirty_planning_refused_without_data_loss(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            f.make("planning")
            dirty = f.planning() / "untracked"
            dirty.write_text("preserve me")
            result = f.make("planning", check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(dirty.read_text(), "preserve me")
            f.assert_source_unchanged()

    def test_true_divergence_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            f.make("planning")
            work = f.planning()
            clone = Path(tmp) / "divergent"
            run(["git", "clone", str(f.origin), str(clone)], Path(tmp))
            git(clone, "config", "user.name", "Fixture User")
            git(clone, "config", "user.email", "fixture@example.invalid")
            git(clone, "checkout", "planning")
            (clone / "remote.txt").write_text("remote\n")
            git(clone, "add", "remote.txt")
            git(clone, "commit", "-m", "remote side")
            git(clone, "push", "origin", "planning")
            (work / "local.txt").write_text("local\n")
            git(work, "add", "local.txt")
            git(work, "commit", "-m", "local side")
            before = git(work, "rev-parse", "HEAD").stdout.strip()
            result = f.make("planning", check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("diverged", result.stderr)
            self.assertEqual(git(work, "rev-parse", "HEAD").stdout.strip(), before)

    def test_path_collision_and_wrong_branch_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            occupied = f.source / ".worktrees" / "planning"
            occupied.mkdir(parents=True)
            (occupied / "keep").write_text("keep")
            result = f.make("planning", check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual((occupied / "keep").read_text(), "keep")
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            wrong = f.source / ".worktrees" / "planning"
            git(f.source, "worktree", "add", "-b", "other", str(wrong), "HEAD")
            result = f.make("planning", check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertTrue((wrong / "tracked.txt").exists())

    def test_unreachable_origin_is_not_treated_as_missing_branch(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            git(f.source, "remote", "set-url", "origin", str(Path(tmp) / "absent.git"))
            result = f.make("planning", check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("could not query remote branch", result.stderr)
            self.assertFalse((f.source / ".worktrees/planning").exists())
            self.assertEqual(git(f.source, "show-ref", "--verify", "--quiet", "refs/heads/planning", check=False).returncode, 1)
            f.assert_source_unchanged()

    def test_implementation_dirty_state_is_preserved(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            staged = f.source / "tracked.txt"
            staged.write_text("staged change\n")
            git(f.source, "add", "tracked.txt")
            staged_index = (f.source / ".git/index").read_bytes()
            staged.write_text("staged plus unstaged\n")
            untracked = f.source / "untracked.txt"
            untracked.write_text("preserve\n")
            head = git(f.source, "rev-parse", "HEAD").stdout.strip()
            f.make("planning")
            self.assertEqual(git(f.source, "rev-parse", "HEAD").stdout.strip(), head)
            self.assertEqual((f.source / ".git/index").read_bytes(), staged_index)
            self.assertEqual(staged.read_text(), "staged plus unstaged\n")
            self.assertEqual(untracked.read_text(), "preserve\n")

    def test_branch_checked_out_elsewhere_is_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            git(f.source, "branch", "planning", "HEAD")
            elsewhere = Path(tmp) / "already attached"
            git(f.source, "worktree", "add", str(elsewhere), "planning")
            result = f.make("planning", check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("already checked out elsewhere", result.stderr)
            self.assertTrue((elsewhere / "tracked.txt").exists())
            self.assertFalse(f.planning().exists())

    def test_push_rejection_preserves_empty_local_root_and_worktree(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            hook = f.origin / "hooks" / "pre-receive"
            hook.write_text("#!/bin/sh\necho fixture push rejection >&2\nexit 1\n")
            hook.chmod(0o755)
            result = f.make("planning", check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertTrue(f.planning().exists())
            commit = git(f.planning(), "rev-parse", "HEAD").stdout.strip()
            self.assertEqual(len(git(f.planning(), "rev-list", "--parents", "-n", "1", commit).stdout.split()), 1)
            self.assertEqual(git(f.planning(), "ls-tree", "-r", "--name-only", "HEAD").stdout, "")
            self.assertEqual(git(f.origin, "show-ref", "--verify", "--quiet", "refs/heads/planning", check=False).returncode, 1)
            f.assert_source_unchanged()

    def test_symlink_path_is_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            outside = Path(tmp) / "outside"
            outside.mkdir()
            link = f.source / ".worktrees" / "planning"
            link.parent.mkdir()
            link.symlink_to(outside, target_is_directory=True)
            result = f.make("planning", check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("symlink", result.stderr)
            self.assertFalse((outside / ".git").exists())

    def test_local_branch_behind_remote_without_worktree_fast_forwards_once(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            f.make("planning")
            work = f.planning()
            old = git(work, "rev-parse", "HEAD").stdout.strip()
            git(f.source, "worktree", "remove", str(work))
            clone = Path(tmp) / "remote fast forward"
            run(["git", "clone", str(f.origin), str(clone)], Path(tmp))
            git(clone, "config", "user.name", "Fixture User")
            git(clone, "config", "user.email", "fixture@example.invalid")
            git(clone, "checkout", "planning")
            (clone / "later.txt").write_text("remote advanced\n")
            git(clone, "add", "later.txt")
            git(clone, "commit", "-m", "remote advances")
            git(clone, "push", "origin", "planning")
            self.assertEqual(git(f.source, "rev-parse", "refs/heads/planning").stdout.strip(), old)
            f.make("planning")
            self.assertTrue((f.planning() / "later.txt").exists())
            self.assertEqual(git(f.planning(), "rev-parse", "HEAD").stdout.strip(),
                             git(f.origin, "rev-parse", "refs/heads/planning").stdout.strip())

    def test_helper_can_be_invoked_from_repository_subdirectory(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            subdir = f.source / "nested"
            subdir.mkdir()
            result = run(["python3", "../scripts/planning-worktree.py", "--path", ".worktrees/planning"], subdir)
            self.assertIn("created empty planning root", result.stdout)
            self.assertEqual(git(f.planning(), "branch", "--show-current").stdout.strip(), "planning")

    def test_git_administration_path_is_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = PlanningFixture(Path(tmp))
            result = run(["python3", "scripts/planning-worktree.py", "--path", ".git/planning"],
                         f.source, check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Git administration", result.stderr)
            self.assertFalse((f.source / ".git/planning").exists())


if __name__ == "__main__":
    unittest.main()
