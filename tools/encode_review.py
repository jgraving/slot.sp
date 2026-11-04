"""Encode and verify native 60 Hz RGBA review captures without GIF quantization."""
import argparse
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess


def encode(with_audio=False):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("captures", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    ffmpeg = os.environ.get("FFMPEG") or shutil.which("ffmpeg")
    ffprobe = os.environ.get("FFPROBE") or shutil.which("ffprobe")
    if not ffmpeg or not ffprobe:
        parser.error("ffmpeg and ffprobe are required (or set FFMPEG/FFPROBE)")
    frames = sorted(args.captures.glob("motion-*.png"))
    if not frames:
        parser.error("no motion-*.png captures")
    size = None
    for index, frame in enumerate(frames):
        if frame.name != f"motion-{index:04}.png":
            parser.error(f"non-contiguous frame sequence at {frame.name}")
        with frame.open("rb") as source:
            header = source.read(24)
        if header[:8] != b"\x89PNG\r\n\x1a\n":
            parser.error(f"invalid PNG: {frame}")
        dimensions = struct.unpack(">II", header[16:24])
        size = size or dimensions
        if dimensions != size or size[0] % 2 or size[1] % 2:
            parser.error(f"inconsistent or odd frame dimensions: {frame}")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    command = [ffmpeg, "-hide_banner", "-loglevel", "error", "-y", "-framerate", "60",
               "-i", str(args.captures / "motion-%04d.png")]
    if with_audio:
        sound = args.captures / "audio.s16le"
        expected_bytes = len(frames) * 48000 // 60 * 2 * 2
        if not sound.exists() or sound.stat().st_size != expected_bytes:
            parser.error("48 kHz stereo PCM duration differs from captured video")
        command += ["-f", "s16le", "-ar", "48000", "-ac", "2", "-i", str(sound),
                    "-c:a", "aac", "-b:a", "192k"]
    else:
        command += ["-an"]
    command += ["-c:v", "libx264", "-preset", "slow", "-crf", "17", "-pix_fmt", "yuv420p",
                "-frames:v", str(len(frames)), "-movflags", "+faststart", str(args.output)]
    subprocess.run(command, check=True)
    metadata = json.loads(subprocess.check_output([
        ffprobe, "-v", "error", "-count_frames", "-show_streams", "-of", "json", str(args.output)
    ]))
    video = next(stream for stream in metadata["streams"] if stream["codec_type"] == "video")
    assert (video["width"], video["height"]) == size
    assert int(video["nb_read_frames"]) == len(frames)
    assert video["avg_frame_rate"] == "60/1"
    assert abs(float(video["duration"]) - len(frames) / 60) < 0.002
    assert any(stream["codec_type"] == "audio" for stream in metadata["streams"]) == with_audio
    subprocess.run([ffmpeg, "-v", "error", "-i", str(args.output), "-f", "null", "-"], check=True)
    print(f"Verified {args.output}: {size[0]}×{size[1]}, {len(frames)} frames, {len(frames)/60:g}s")
