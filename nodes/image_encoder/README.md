# robot-bus-image-encoder

Tool node: `sensor_msgs/Image` → `foxglove_msgs/CompressedVideo` (H.264 / H.265) using **system FFmpeg**.

## Requirements

- Running `robot_bus_broker`
- FFmpeg development libraries on the machine (not bundled)

```bash
brew install ffmpeg
# or: sudo apt install ffmpeg libavcodec-dev libavutil-dev libswscale-dev
```

## Run

```bash
cargo run -p robot-bus-image-encoder -- --params nodes/image_encoder/config/example.yaml
# just node-image-encoder
```

See [`config/example.yaml`](config/example.yaml) for parameters. How to add more tool nodes: [`../README.md`](../README.md).
