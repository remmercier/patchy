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
From 9f13ffd1036c302e1bdaf31dd4c8fcd1202ba981 Mon Sep 17 00:00:00 2001
From: Nikita Revenco <154856872+NikitaRevenco@users.noreply.github.com>
Date: Wed, 18 Dec 2024 15:05:00 +0000
Subject: [PATCH] feat: remove tab keybindings

---
 helix-term/src/keymap/default.rs | 2 +-
 helix-term/src/ui/menu.rs        | 4 ++--
 2 files changed, 3 insertions(+), 3 deletions(-)

diff --git a/helix-term/src/keymap/default.rs b/helix-term/src/keymap/default.rs
index c6cefd92..105b8f99 100644
--- a/helix-term/src/keymap/default.rs
+++ b/helix-term/src/keymap/default.rs
@@ -215,7 +215,7 @@ pub fn default() -> HashMap<Mode, KeyTrie> {

         // z family for save/restore/combine from/to sels from register

-        "C-i" | "tab" => jump_forward, // tab == <C-i>
+        "C-i" => jump_forward, // tab == <C-i>
         "C-o" => jump_backward,
         "C-s" => save_selection,

diff --git a/helix-term/src/ui/menu.rs b/helix-term/src/ui/menu.rs
index 612832ce..aaba784a 100644
--- a/helix-term/src/ui/menu.rs
+++ b/helix-term/src/ui/menu.rs
@@ -274,12 +274,12 @@ fn handle_event(&mut self, event: &Event, cx: &mut Context) -> EventResult {
                 return EventResult::Consumed(close_fn);
             }
             // arrow up/ctrl-p/shift-tab prev completion choice (including updating the doc)
-            shift!(Tab) | key!(Up) | ctrl!('p') => {
+            key!(Up) | ctrl!('p') => {
                 self.move_up();
                 (self.callback_fn)(cx.editor, self.selection(), MenuEvent::Update);
                 return EventResult::Consumed(None);
             }
-            key!(Tab) | key!(Down) | ctrl!('n') => {
+            key!(Down) | ctrl!('n') => {
                 // arrow down/ctrl-n/tab advances completion choice (including updating the doc)
                 self.move_down();
                 (self.callback_fn)(cx.editor, self.selection(), MenuEvent::Update);
--
2.47.0""",
]
```
