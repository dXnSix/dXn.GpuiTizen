# GpuiTizen Video

A template for Seller Office's feature description file; complete the bracketed parts before submitting.

## What the app does

Plays a bundled 4-second test clip (a generated test pattern with a tone) full screen, looping, with a row of
controls over it: rewind 10 seconds, play or pause, forward 10 seconds, mute. It shows video playback through
Tizen's media player in a GPUI app.

## Screens and controls

- One screen. Left and Right move between the controls, OK uses one, Back closes the app.
- Everything is reachable with the four directions, OK and Back.
- When the TV interrupts playback (another app takes the decoder), Play opens the clip again.

## Privileges

- `http://tizen.org/privilege/internet`: [declared by the shared host; the clip is played from the package].

## Content and data

No account, no purchases, no personal data; the clip was generated for the app. [Contact for review questions.]
