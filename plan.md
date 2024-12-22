Data for a pull request can be accessed via the following link:

```
https://api.github.com/repos/helix-editor/helix/pulls/12309
```

Then we can access the fields:

- `.head.repo.clone_url` is a string like `https://github.com/NikitaRevenco/helix.git`.
- `.head.ref` is the branch name of the pull request.

Now we know which branch and which url we should clone from.

Here's what we do.

First, fetch all of the remotes for all of the pull requests, including the main repo with this command:

```
git remote add local-branch-name remote-name
git fetch remote-name branch-name:branch-name
```

Check out the main repository that we cloned. Not the pull request repositories, but the main one.

Merge all of the pull requests' remotes into our main repo

```
git merge remote-name/branch-name --message "Merge #remote-name#"
```

Once all have been merged, create a new branch which the user chose:

This will move our changes from the thing that we did.

```
git switch --create branch-name2
```

First, backup the `.gpatch.toml` file:

```
git switch --create gpatch-backup
git add .gpatch.toml
git commit --message "Store .gpatch.toml"
```

The following command force replaces the original branch name with our new branch.

```
git branch --move --force branch-name2 branch-name
```

At this point, we have merged all the pull requests and also stored it in a new branch.

We're going to restore our `.gpatch.toml` file now:

```
git cherry-pick --no-commit gpatch-backup
git commit --message "Restore .gpatch.toml"
```
