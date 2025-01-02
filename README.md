# patchy

`patchy` makes it easy to maintain personal forks in which you merge some pull requests of your liking to have more features than other people.

- [Why should I use it?](#why-should-i-use-it)
- [Usage](#usage)
  - [Config](#config)
  - [Patches](#patches)
  - [Versioning](#versioning)
- [Installation](#installation)
  - [Binary](#binary)
  - [Homebrew](#homebrew)
  - [Cargo](#cargo)
  - [PowerShell](#powershell)
  - [Nix](#nix)

## Why should I use it?

- Merge multiple pull requests and commits into a single repo effortlessly
- Sync those pull requests and the main branch with a single command
- Add new pull requests and update existing ones easily

## Usage

Go to a git repo, and initialize the config file:

```sh
patchy init
```

Invoke `patchy` by running the following command:

```sh
patchy run
```

### Config

I'm using the [Helix Editor](https://github.com/helix-editor/helix) but there are some pull requests which add awesome features. 

I found myself very frequently doing the same tasks in order to sync the 4 pull requests I like to use and keep them up to date. With patchy, I just run one command and it handles the rest.

Here's my config:

```toml
# main repository to fetch from
repo = "helix-editor/helix"

# the repository's branch
remote-branch = "master"

# This is the branch where you will see all result from patchy's work. Set it to any branch you want.
# WARNING: Make sure you do not store any important work on this branch. It will be erased.
local-branch = "patchy"

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
  "11164",
]

# A list of patches to apply, more on them later
patches = ["remove-tab"]
```

Running `patchy run` outputs:

![patchy output](https://github.com/user-attachments/assets/c0076588-6e57-4a80-9d05-955a4dff2580)

With this, all I will need to do is run `patchy run` and it will automatically update all of the pull requests and sync the master branch to the latest changes.

### Patches

You might want to apply some changes to your repo, but it's not a pull request. No worries! `patchy` is built for this.

Create a patch from a commit:

```sh
# obtain commit hashes e.g. from `git log`
patchy gen-patch <hash-of-commit>
```

For example, I'm running:

```sh
patchy gen-patch 7bb8ec5a77769d88855d41dd5fecfaece54cf471
```

It generated a file, `.patchy/feat-swap-light-and-dark-colors.patch`:

```patch
diff --git a/README.md m/README.md
index 11a909b2..4eae6a8d 100644
--- a/README.md
+++ m/README.md
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

To use your new `.patch`, edit your `.patchy/config.toml` like so:

```diff
--- patches = []
+++ patcher = [ "feat-swap-light-and-dark-colors" ]
```


### Versioning

Each pull request's branch contains commits. By default, we will always use the latest commit. However you can pin a commit to a specific version with the following syntax:

```toml
remote-branch = "main @ cfd225baedbb5fb9cbc9742f91244fa50882b580"

pull-requests = [
   "145 @ fccc58957eece10d0818dfa000bf5123e26ee32f",
   "88 @ a556aeef3736a3b6b79bb9507d26224f5c0c3449"
]
```

Where the hashes represent each `sha1` hash of every commit.

This is handy if you don't want things to randomly break when some of the pull requests push a new change.

## Installation

Patchy can be installed on Linux, Windows and macOS.

### Binary

Install the binary directly into your system, available for macOS and Linux.

Recommended for Linux users.

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/NikitaRevenco/patchy/releases/latest/download/patchy-installer.sh | sh
```

### Homebrew

Recommended for macOS users.

```bash
brew install NikitaRevenco/tap/patchy
```

### Cargo

```bash
cargo install patchy-bin
```

### PowerShell

Recommended for Windows users.

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/NikitaRevenco/patchy/releases/latest/download/patchy-installer.ps1 | iex"
```

### Nix

```bash
nix profile install github:NikitaRevenco/patchy/main
```
