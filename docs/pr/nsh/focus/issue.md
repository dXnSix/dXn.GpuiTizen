# Directional focus: move the keyboard focus with the arrows on every backend, and what a TV remote needs

> Draft for longbridge/nsh, not posted. Post only with the owner's go. The branches are built ahead of the answer
> (`pr/focus/01-core` … `07-web-tv`) and change in place if it differs. The desktop defaults are asked separately in
> [issue-desktop-tab.md](issue-desktop-tab.md), since they change what existing apps do.

We are bringing NSH to Samsung TVs (Tizen), where an app is driven by a remote: four arrows, OK and Back. Every view
the user reaches must be reachable with the arrows. Today only the terminal backend moves the keyboard focus with
them; GPUI, the desktops and the web scroll. We would like one set of rules for all of them, and a few props a TV app
needs on top.

## Today

| Backend | Tab | Arrows |
|---|---|---|
| Terminal | the stops, in tree order | move the focus (`keyboard.rs`, over Cells) |
| GPUI | fields and native components | scroll |
| macOS | fields and pressables, AppKit's key view loop (screen order) | scroll |
| Linux | fields and pressables, GTK's focus chain (screen order) | scroll |
| Windows | text fields only | scroll |
| Web | the browser's (DOM order) | scroll |
| Headless (`nsh test`) | `Screen::tab_stops`, tree order | – |

## 1. The rules in the core, over plain data

`nsh_core::focus`, beside `nsh_core::input`: the backend describes its stops (id, frame, focus stop, input scope,
layer, popup, tree order), the root view's layers and the modals that show; the core answers the scope (the newest
modal), the stops less what a backdrop covers, where Tab, Left / Right and Up / Down go, the reveal delta of each
scroll view, and where the focus goes back when a modal or a layer closes. These are the terminal's rules, moved:
Left / Right to the nearest on an overlapping row, Up / Down to the nearest below or above with an overlapping column
first.

Frames are in the backend's own unit, scaled per axis (the terminal gives each Cell as one point): every rule compares
one axis at a time and takes no length of its own. The terminal becomes the first user with no change in behaviour.

## 2. Each backend supplies its stops and applies the answer

- **GPUI**: views that are neither fields nor components get a focus of their own (with #1545's `key_focus`); the
  arrows come after the app's bindings, the menus, the focused field and selectable text.
- **macOS, Linux, Windows**: the same stops Tab goes through, with their frames as they show (scroll offsets and
  transforms applied); on Windows views get a focus of their own, as GPUI's.
- **Web**: `nsh.js` describes the page (`getBoundingClientRect`, scopes, layers, what is in sight) and the core's rules,
  compiled into the module, decide; the page focuses without the browser's scrolling and glides the scroll views.

Everywhere: an arrow that moves no focus scrolls as before; fields keep the arrows they use; Enter and Space press the
focused stop; `focus:` styles, `onFocus` / `onBlur` and the accessibility focus follow.

## 3. What a TV adds

```tsx
<View autoFocus nextFocusDown={play()} onPress={resume}><Text>Resume</Text></View>
<View ref={setPlay} preferredFocus onPress={start}><Text>Play</Text></View>
<FocusGroup destinations={[current()]} trap={menuOpen()}>{tiles}</FocusGroup>
```

- `autoFocus` on every element (today `TextInput`'s): a view takes the focus when it appears.
- `preferredFocus`: the stop that wins where the focus enters (a scope with nothing focused, a group).
- `nextFocusUp` / `Down` / `Left` / `Right`: a ref the arrow goes to past geometry; it may name a view rendered later.
- `FocusGroup` (`nsh/base`): entered at its `destinations`, else its `preferredFocus` stop, else the nearest; with
  `trap` the arrows stay inside. As Android TV's focus groups and the web's spatial navigation containers.
- Lists: an arrow past the last mounted row scrolls the list to the next row by index and focuses its first stop once
  mounted (`EngineControl::focus_past`, `Command::ScrollToIndex`, then `Command::Focus`).

They apply where the arrows move the focus between views: GPUI and the web, TVs included. On the native desktops they
are accepted and ignored for now.

## Questions

1. **The core module.** Is `nsh_core::focus` (decisions over plain data, the backends applying them) the shape you
   want, as `nsh_core::input` is for presses?
2. **The arrows by default**: see [issue-desktop-tab.md](issue-desktop-tab.md). GPUI and the web need them for TVs;
   on the desktops the answer may differ.
3. **The props' names.** `nextFocusUp` … `nextFocusRight` are React Native's (and Android's) names;
   `preferredFocus` is close to React Native's `hasTVPreferredFocus`; `autoFocus` keeps the input's meaning and
   widens to every element. OK, or other names?
4. **`FocusGroup` in `nsh/base`** as a component over `Interactive::focus_group`, or a prop on `View` only?
5. **Lists**: the platform asks the engine (`focus_past`) and the core scrolls by index. OK to add to `EngineControl`?
6. **`focus-visible:`** (#1519): a focus the arrows move shows it, as a key's. Agreed?
7. **iOS and Android TV**: native tvOS / Android TV focus engines would read the same props later; out of scope here.

## The PRs

| # | Branch | What | Needs |
|---|---|---|---|
| 1 | `pr/focus/01-core` | the core module; the terminal on it, unchanged | – |
| 2 | `pr/focus/02-gpui` | GPUI's arrows | 1 |
| 3 | `pr/focus/03-desktop-stops` | macOS, Linux, Windows: Tab in tree order, the arrows | 1, the desktop issue |
| 4 | `pr/focus/04-tv` | the TV props, `FocusGroup`, lists; GPUI honours them; compat cases | 2, an nsh-compat PR |
| 5 | `pr/focus/05-web` | the web's arrows | 1 |
| 6 | `pr/focus/06-gpui-tab-stops` | GPUI's Tab at the arrows' stops | 4, the desktop issue |
| 7 | `pr/focus/07-web-tv` | the web honours the TV props | 4, 5 |

Opened one at a time along each line, each holding only its own commits.
