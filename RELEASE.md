# Release

To release a new compositor version:

- Go to `Actions` -> [`package for release`](https://github.com/membraneframework/video_compositor/actions/workflows/package_for_release.yml) -> Trigger build on master using "Run workflow" drop-down menu.
- Wait for a job to finish.
- Run `gh run list --workflow "package for release"` and find an ID of the workflow run that packaged release binaries. Running `./scripts/release.sh` without necessary environment variables will also display that list.
- Run 
  ```bash
  WORKFLOW_RUN_ID={WORKFLOW_RUN_ID} RELEASE_TAG={VERSION} ./scripts/release.sh
  ```
  e.g.
  ```bash
  WORKFLOW_RUN_ID=6302155380 RELEASE_TAG=v1.2.3 ./scripts/release.sh `
  ```


### Temporary workaround for Apple silicon devices

Currently we do not have a CI to build for Apple silicon, so for now compositor releases will be published from developer devices.

- Follow instructions above for other platforms to create new GitHub release. 
- To build and upload binaries for existing release run bellow command on an Apple silicon device.
  ```bash
  RELEASE_TAG={VERSION} cargo run --bin package_for_release
  ```
  e.g.
  ```bash
  RELEASE_TAG=v1.2.3 cargo run --bin package_for_release
  ```
