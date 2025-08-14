# github-release-generator

Generate a simple GitHub changelog using the GitHub API. Linked issues are shown
using their issue title while unknown commits are shown in the "raw" changelog.

Using the great GitHub market place action [release-changelog-builder-action](https://github.com/mikepenz/release-changelog-builder-action)
it is easy to generate automatic changelogs for new releases. See the example below.

Check `--help` for possible and required command line arguments.

## Example Changelog

Below is an example of generated release notes:

```output
# v1.2.0 (2023-06-15)

## 🐛 Fixed Bugs

- Fix navigation bug in dashboard - #42
- Correct timestamp formatting in logs - #47

## ✅ Closed Issues

- Add dark mode support - #38
- Improve API response handling - #45

## ⚠️ Issues that are mentioned but not closed

- Documentation updates needed - #51

## Full changelog of commits

- a7b3c9d - Bump version to 1.2.0 by Alex Smith (2023-06-15 10:30:22 UTC)
- e4f5g6h - feat: implement dark mode across all views. Close #38. by Taylor Johnson (2023-06-14 16:45:19 UTC)
- i7j8k9l - fix: resolve dashboard navigation issues. Fix #42. by Sam Wilson (2023-06-13 09:20:11 UTC)
- m1n2o3p - feat: enhance error handling for API responses. Close #45. by Jamie Lee (2023-06-12 14:37:42 UTC)
- q4r5s6t - fix: correct timestamp format in log entries. Fix #47. by Robin Garcia (2023-06-11 11:52:33 UTC)
- u7v8w9x - docs: update API documentation sections. #51 by Morgan Chen (2023-06-10 08:15:47 UTC)
```

## Command line arguments

```sh
Usage: github-release-generator [OPTIONS] --from <FROM> --github-repo <GITHUB_REPO> --github-token <GITHUB_TOKEN>

Options:
  -f, --from <FROM>                  Start of range (commit hash, tag, branch, or refspec)
  -t, --to <TO>                      End of range (commit hash, tag, branch, or refspec) [default: HEAD]
      --github-repo <GITHUB_REPO>    GitHub repository in the form owner/repo
      --github-token <GITHUB_TOKEN>  GitHub API token
      --filter-label <FILTER_LABEL>  Only include issues/PRs with this label (optional)
  -d, --directory <DIRECTORY>        [default: .]
  -h, --help                         Print help
  -V, --version                      Print version
```

## Example Usage

```sh
cargo run -- -d $DIR --from $FROM --to $TO --github-repo $REPO  --github-token $GITHUB_TOKEN
```

## Example with Github CI

```yml
# Permissions are needed
permissions:
  contents: write
  issues: read
  pull-requests: read

       [...]

      - name: Run github release generator
        run: |
          docker run --rm -v $(pwd):/data ghcr.io/stonemaster/github-release-generator:main \
            -d /data \
            --github-repo ${{ github.repository }} \
            --github-token ${{ secrets.GITHUB_TOKEN }} \
            --from ${{ steps.last_tag.outputs.tag }} \
            --to ${{ github.ref }} \
            | tee log.txt
        shell: bash

      - name: Create Release
        uses: mikepenz/action-gh-release@v1
        with:
          body_path: log.txt
```

## Docker container

A Docker container is provided. Example usage:

```sh
docker run --rm -v $(pwd):/data ghcr.io/stonemaster/github-release-generator:main \
  -d /data --from $FROM --to $TO --github-repo $REPO  --github-token $GITHUB_TOKEN
```
