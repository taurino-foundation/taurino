# Taurino GitHub Automation

## Required one-time configuration

1. Install the `.github/` directory **and** the two release-please JSON files at repository root. The manifest initially contains `0.1.0` — adapt if the currently published version differs. Do not reinitialize an existing release history blindly.
2. Configure `RELEASE_PLEASE_TOKEN` as an Actions repository secret. Use a narrowly scoped GitHub App token / fine-grained PAT with repository **Contents: read/write** and **Pull requests: read/write**. The secret lets bot-generated PRs run CI and allows the lockfile sync automation. Some orgs also need Actions > General > **Allow GitHub Actions to create and approve pull requests**. The fallback to `GITHUB_TOKEN` can create a release PR, but it will not reliably run other workflows on the resulting PR and cannot update `Cargo.lock` via the lockfile sync job.
3. Settings > Branches or Rulesets: protect `main`, require PRs and CI, and **squash merge** with Conventional Commit PR titles, e.g. `feat(runtime): add IPC reconnect`. GitHub may require configuring the default squash commit format to use the PR title; Release Please relies on the **actual merged commit messages**, not on PR titles alone.
4. Create Settings > Environments > `pypi`. Optionally add required reviewers (when supported for your repository). This adds a second approval on top of manually selecting **Actions > Publish Python > Run workflow**.
5. PyPI pending Trusted Publisher must be configured as: project **taurino**, owner **taurino-foundation**, repo **taurino**, workflow **publish-python.yaml**, environment **pypi** (matching the screenshot). A pending publisher does not reserve the name and activation occurs on first successful publish. See security notes below.
6. On the first PR, inspect CI results. Linux WebKit-dependent Rust crates may require system-library changes; do not disable failing tests just to release.

## Daily workflow

- Create branch: `git switch -c feat/my-feature`
- Make changes, `git commit -m "feat(ipc): add a new transport"`, push and open PR.
- After PR passes CI, squash merge into main.
- Release Please creates / updates the release PR and `CHANGELOG.md`. It synchronizes crate version in `crates/taurino/Cargo.toml` and the workspace's `taurino` dependency version. A separate safe PR-only workflow updates `Cargo.lock`.
- Review/merge release PR: Release Please creates the Git tag + GitHub Release **without publishing to PyPI**.
- When you decide to publish: Actions > **Publish Python** > **Run workflow** > enter existing release tag (`v0.1.1`) > Run. This builds wheels and an sdist, then publishes using OIDC. If environment approval is enabled, click **Review deployments**.

## Version policy

- Starting at `0.1.0`; only conventional commits with release-relevant types trigger a version bump. `feat` normally bumps MINOR; `fix` bumps PATCH, with pre-1.0 release-please knobs affecting this behavior.
- `pyproject.toml` uses `dynamic=["version"]` and Maturin reads `crates/taurino/Cargo.toml`; no independent Python version bump is required.
- `Cargo.lock` is refreshed in CI for pre-release builds; the lockfile-sync job commits the change back to release PR (requires `RELEASE_PLEASE_TOKEN`).
- Release Please is not AI: it writes structured release notes based on Conventional Commits. For high-quality release notes, make commit subjects meaningful.

## Important limits / cautions

- Current project declares `requires-python >=3.9,<3.15`, but several production dependencies (e.g. `black`, `maturin`, `isort`, `ormsgpack`) may not support the full range. CI will expose actual incompatibilities. **Remove development tools from production `project.dependencies`** and place in `dependency-groups` when ready; do not publish until the claimed Python compatibility is tested.
- The PyPI workflow publishes only the four listed build targets (Linux x86_64, Windows x86_64, macOS Intel and ARM). Linux's manylinux 2_28 target may need additional dependency handling for WebKit/Tao. Adding a target requires testing; no implicit claims about other platforms.
- Every release's source snapshot must be reproducible; this is a baseline automation, not a certificate that dependencies or outputs are safe. Consider SHA pinning actions, SLSA provenance and Sigstore attestations after functional validation.
- First PyPI publication is **not** guaranteed merely by having a pending publisher. If someone else creates the PyPI `taurino` project first, publishing can fail. Verify project ownership before publishing.
- Release Please PRs created using default `GITHUB_TOKEN` won't trigger separate CI; use a GitHub App or PAT. Protect these credentials, avoid running write-token jobs on untrusted forks, and rotate credentials as needed.
- Scheduled dependency/security jobs may identify existing vulnerabilities and fail intentionally.
