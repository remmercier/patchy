# gitpatcher

This CLI tool makes it easier to have personal forks of repositories where you simply merge a few pull requests of your choice.

For example, say you are using the Helix Editor and you're happy with the features but there's several pull requests which add features you would like to use. Manually rebasing those PRs can get old quick, especially if you want to often keep your branch up-to-date.

`gitpatcher` automates this task by allowing you to declaratively configure repositories. It uses a file `.gitpatcher.toml` which stores information such as:

- The repository you would like to clone
- Local branch `gitpatcher` should use where all the work happens
- Pull requests which should be automatically applied to the repository

## Configuration

Here's an example config

```toml
repo = "helix-editor/helix"
# WARNING: This branch will be hard-reset
local-branch = "@gitpatcher"
pull-requests = [
  # color swatches that appear next to colors
  "#12309",
  # file browser
  "#11285",
  # global statusline option
  "#8908",
  # command expansions
  "#11164",
]
patches = [
  # tab and s-tab conflict with eachother in insert mode
  """
]
```
