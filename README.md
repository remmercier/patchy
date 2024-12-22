# gpatch

`gpatch` makes life simple when you just want to use a repository with some of the pull requests from that repository merged into your personal fork. A "Pull-Request Shopping", if you will.

## Why should I use it?

- Merge multiple pull requests and commits into a single repository effortlessly
- Sync those pull requests and the main remote easily
- Edit a simple toml config file to add new pull requests or remove existing ones, update with a single command

## Usage

Go to any git repository, and initialize the config file:

```sh
gpatch init
```

Invoke the `gpatch` by running the following command:

<!-- TODO: make it run from anywhere within repository -->

```sh
gpatch run
```

### Patches

Create a patch from a commit:

```sh
gpatch gen <hash-of-commit>
```

For example, I ran:

```sh
gpatch gen 7bb8ec5a77769d88855d41dd5fecfaece54cf471
```

It generated the following file, `.gpatch/feat-swap-light-and-dark-colors.patch`:

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

You can then use the `.patch` editing your TOML file like this:

```diff
--- patches = []
+++ patcher = [ "feat-swap-light-and-dark-colors" ]
```

This feature is handy when you want to have some special commits in your repository that you made yourself for example, but don't want to make a pull request for them.

### Configuration

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
pull-requests = [ "12309", "11285", "8908", "11164" ]

# An list of patches to apply (see below)
patches = []
```

With this, all I will need to do is run `gpatch` **from the root of the repository** and it will automatically update all of the pull requests and sync the master branch to the latest changes.

## Installation

TODO: add section
