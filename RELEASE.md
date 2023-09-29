# Release

To release a new compositor version:

- Go to `Actions` -> [`package for release`](https://github.com/membraneframework/video_compositor/actions/workflows/release.yml) -> Dispatch new workflow on a master branch.
- Wait for a build to finish.
- Run `gh run list --workflow "package for release"` and find ID of workflow run you just run.
- Run `WORKFLOW_RUN_ID={WORKFLOW_RUN_ID} RELEASE_TAG={VERSION} ./scripts/release.sh` where `WORKFLOW_RUN_ID` is an ID from the previous step, and `VERSION` has a format `v{major}.{minor}.{patch}`. e.g. `WORKFLOW_RUN_ID=6302155380 RELEASE_TAG=v1.2.3 ./scripts/release.sh `


### Temporary workaround for macOS M1 binaries

Currently we do not have a CI to build for macOS M1, so for now compositor releases are run from a developer device.

To publish binaries for M1 devices, first follow instructions above for other platform. After GitHub release is created you can add binaries for M1 by running bellow command on M1 mac.

```bash
RELEASE_TAG={VERSION} cargo run --bin package_for_release --features standalone
```

e.g.
```bash
RELEASE_TAG=v1.2.3 cargo run --bin package_for_release --features standalone
```
