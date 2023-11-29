# Release process

To release new versions of one or more packages in the workspace:

1. Create a local branch and make these changes:
   1. Update the version of each package you want to release in the
      corresponding `Cargo.toml`. Follow semver.
   2. If needed, update the dependency version in packages that depend
      on the updated package. For example, if you are releasing changes
      to `uguid` that require a major version bump, the dependency on
      `uguid` in `gpt_disk_types` should be updated. Ditto for a minor
      version bump if the dependent package requires any of the new
      functionality. Patch version bumps do not require a change in the
      dependent package. Note that in cargo's version of semver, `0.x.y`
      treats `x` as the major version number.
   3. Run `cargo build` to ensure `Cargo.lock` is updated.
   4. Update the changelogs.
2. Commit those changes. The commit message subject must start with
   `release:`. Without that prefix, the automatic part of the release
   process will not run.
3. Push the branch and create a PR. See for example [#158].
4. When the PR is reviewed and merged, the automatic release process
   will kick off.
   
[#158]: https://github.com/google/gpt-disk-rs/pull/158

## The automatic part

The rest of the release process is triggered by
`.github/workflows/release.yml`, which runs when commits are pushed to
the main branch. It runs `cargo xtask auto_publish`.

The `auto_publish` command gets the local version of each package and,
if necessary, creates a remote git tag and publishes to crates.io. If
the git tag already exists, that part is skipped. If the crates.io
release already exists, that part is skipped.

## Crates.io token

Releasing to crates.io requires an API token. That is passed to the job
via a repository secret called `CARGO_REGISTRY_TOKEN`. The secret is
configured in the repo settings. When creating an API token, restrict
the scope to `publish-update`.
