# clip.mp4

A synthetic test clip made for this example: ffmpeg's `testsrc2` pattern with a 440 Hz tone,
4 seconds, 640×360 at 30 fps, H.264 Main and AAC in MP4, generated in the `gpuitizen` Lima VM:

```bash
ffmpeg -f lavfi -i testsrc2=size=640x360:rate=30 -f lavfi -i sine=frequency=440:sample_rate=48000 \
  -t 4 -c:v libx264 -profile:v main -pix_fmt yuv420p -preset veryslow -crf 32 \
  -c:a aac -b:a 64k -movflags +faststart clip.mp4
```

`cargo xtask package --app video` copies it to the TPK's `res/`.
