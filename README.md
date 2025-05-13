# github-release-generator

Generate a simple GitHub changelog using the GitHub API. Linked issues are shown
using their issue title while unknown commits are shown in the "raw" changelog.

Check `--help` for possible and required command line arguments.

## Example Usage

```sh
cargo run -- -d $DIR --from $FROM --to $TO --github-repo $REPO  --github-token $GITHUB_TOKEN
```

## Docker container

A Docker container is provided. Example usage:

```sh
docker run --rm -v $(pwd):/data ghcr.io/stonemaster/github-release-generator:main \
  -d /data --from $FROM --to $TO --github-repo $REPO  --github-token $GITHUB_TOKEN
```
