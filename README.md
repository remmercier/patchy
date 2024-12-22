# gpatch

`gpatch` makes life simple when you just want to use a repository with some of the pull requests from that repository merged into your personal fork.

Let's go on a pull request shopping spree together!

## Why should I use it?

- Merge multiple pull requests and commits into a single repo effortlessly
- Sync those pull requests and the main branch with a single command
- Add new pull requests and update existing ones easily

## Usage

Go to a git repo, and initialize the config file:

```sh
gpatch init
```

Invoke `gpatch` by running the following command:

```sh
gpatch run
```

### Patches

You might want to apply some changes to your repo, but it's not a pull request. No worries! `gpatch` is built for this.

Create a patch from a commit:

```sh
# obtain commit hashes e.g. from `git log`
gpatch gen <hash-of-commit>
```

For example, I'm running:

```sh
gpatch gen 7bb8ec5a77769d88855d41dd5fecfaece54cf471
```

It generated a file, `.gpatch/feat-swap-light-and-dark-colors.patch`:

```patch
diff --git a/README.md b/README.md
index 11a909b2..4eae6a8d 100644
--- a/README.md
+++ b/README.md
@@ -2,8 +2,8 @@

 <h1>
 <picture>
-  <source media="(prefers-color-scheme: dark)" srcset="logo_dark.svg">
   <source media="(prefers-color-scheme: light)" srcset="logo_light.svg">
+  <source media="(prefers-color-scheme: dark)" srcset="logo_dark.svg">
   <img alt="Helix" height="128" src="logo_light.svg">
 </picture>
 </h1>
```

To use your new `.patch`, edit your `.gpatch/config.toml` like so:

```diff
--- patches = []
+++ patcher = [ "feat-swap-light-and-dark-colors" ]
```

### Config

Generate the sample config:

```sh
gpatch init
```

This is a real-world example, specifically I myself used it at some point. I'm using the [Helix Editor](https://github.com/helix-editor/helix) but there are some pull requests which add awesome features. I found myself very frequently doing the same tasks in order to sync the 4 pull requests I like to use.

Here's my config:

```toml
# main repository to fetch from
repo = "helix-editor/helix"

# the repository's branch
remote-branch = "master"

# This is the branch where you will see all result from gitpatchers' work. Set it to any branch you want.
# WARNING: Make sure you do not store any important work on this branch. It will be erased.
local-branch = "gpatch"

# List of pull requests which you would like to merge
# TIP: Add comments above pull requests to help yourself understand which PRs do what
pull-requests = [
   # syntax highlighting for nginx files
   "12309",
   # adds file explorer
   "11285",
   # global status line
   "8908",
   # command expansions
   "11164"
]

# A list of patches to apply
patches = []
```

With this, all I will need to do is run `gpatch` **from the root of the repository** and it will automatically update all of the pull requests and sync the master branch to the latest changes.

## Installation

TODO: add section
