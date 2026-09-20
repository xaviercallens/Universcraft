"""
UniversCraft / HoloEngine — Real-Time Demonstration of Kerr Black Hole Relativistic Spacetime
Renders a 360-degree orbital sequence on the NVIDIA GeForce RTX GPU.
"""

import math
import os
import struct
import sys
import time
import numpy as np
from PIL import Image
import wgpu

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')

from test_rtx_gpu import extract_scenes

def main():
    print("=" * 75)
    print(" 🕳️ REAL-TIME TOPOLOGY DEMONSTRATION: KERR BLACK HOLE RELATIVISTIC SPACETIME")
    print("=" * 75)

    base_dir = os.path.dirname(os.path.abspath(__file__))
    source_path = os.path.join(base_dir, "src", "client", "advanced_scenes.rs")
    scenes = extract_scenes(source_path)
    shader_code = scenes["black_hole"]

    adapter = wgpu.gpu.request_adapter_sync(power_preference="high-performance")
    device = adapter.request_device_sync()
    print(f"[*] GPU Engine: {adapter.summary}")

    width, height = 1280, 720
    total_pixels = width * height
    output_size_bytes = total_pixels * 4

    shader_module = device.create_shader_module(code=shader_code)
    storage_buffer = device.create_buffer(size=output_size_bytes, usage=wgpu.BufferUsage.STORAGE | wgpu.BufferUsage.COPY_SRC)
    readback_buffer = device.create_buffer(size=output_size_bytes, usage=wgpu.BufferUsage.MAP_READ | wgpu.BufferUsage.COPY_DST)

    bind_group_layout = device.create_bind_group_layout(entries=[
        {"binding": 0, "visibility": wgpu.ShaderStage.COMPUTE, "buffer": {"type": wgpu.BufferBindingType.uniform}},
        {"binding": 1, "visibility": wgpu.ShaderStage.COMPUTE, "buffer": {"type": wgpu.BufferBindingType.storage}},
    ])

    pipeline_layout = device.create_pipeline_layout(bind_group_layouts=[bind_group_layout])
    compute_pipeline = device.create_compute_pipeline(
        layout=pipeline_layout,
        compute={"module": shader_module, "entry_point": "raymarch_main"},
    )

    wg_x = (width + 15) // 16
    wg_y = (height + 15) // 16

    num_frames = 36
    orbit_radius = 22.0
    orbit_height = 5.5
    frames = []

    print(f"[*] Dispatching {num_frames} Kerr spacetime frames at {width}x{height} resolution...")
    t_start_all = time.perf_counter()

    for f_idx in range(num_frames):
        angle = (2.0 * math.pi * f_idx) / num_frames
        cam_x = orbit_radius * math.sin(angle)
        cam_z = -orbit_radius * math.cos(angle)
        cam_y = orbit_height + 2.0 * math.sin(angle)

        target = [0.0, 0.0, 0.0]
        dir_vec = [target[0] - cam_x, target[1] - cam_y, target[2] - cam_z]
        norm = math.sqrt(dir_vec[0]**2 + dir_vec[1]**2 + dir_vec[2]**2)
        cam_dir = [dir_vec[0] / norm, dir_vec[1] / norm, dir_vec[2] / norm]

        buf_data = struct.pack(
            "<3ff3fI3fIIfII",
            cam_x, cam_y, cam_z, 65.0,
            cam_dir[0], cam_dir[1], cam_dir[2], width,
            0.0, 1.0, 0.0, height,
            48, 250.0, 0, 0
        )
        uniform_buf = device.create_buffer_with_data(data=buf_data, usage=wgpu.BufferUsage.UNIFORM)

        bind_group = device.create_bind_group(layout=bind_group_layout, entries=[
            {"binding": 0, "resource": {"buffer": uniform_buf, "offset": 0, "size": uniform_buf.size}},
            {"binding": 1, "resource": {"buffer": storage_buffer, "offset": 0, "size": storage_buffer.size}},
        ])

        t0 = time.perf_counter()
        encoder = device.create_command_encoder()
        cpass = encoder.begin_compute_pass()
        cpass.set_pipeline(compute_pipeline)
        cpass.set_bind_group(0, bind_group)
        cpass.dispatch_workgroups(wg_x, wg_y, 1)
        cpass.end()
        encoder.copy_buffer_to_buffer(storage_buffer, 0, readback_buffer, 0, output_size_bytes)
        device.queue.submit([encoder.finish()])

        readback_buffer.map_sync(wgpu.MapMode.READ)
        data = readback_buffer.read_mapped()
        readback_buffer.unmap()
        t1 = time.perf_counter()

        frame_ms = (t1 - t0) * 1000.0
        frame_fps = 1000.0 / frame_ms if frame_ms > 0 else 0.0

        raw_bytes = bytes(data)
        pix = np.frombuffer(raw_bytes, dtype=np.uint8).reshape((height, width, 4))
        img = Image.fromarray(pix, mode="RGBA")
        frames.append(img)

        if f_idx % 6 == 0 or f_idx == num_frames - 1:
            print(f"  Frame {f_idx+1:02d}/{num_frames:02d} | Angle: {math.degrees(angle):5.1f}° | Latency: {frame_ms:5.2f} ms ({frame_fps:5.1f} FPS)")

    t_total = time.perf_counter() - t_start_all
    avg_fps = num_frames / t_total
    print(f"\n[✓] Rendered {num_frames} frames in {t_total:.2f} s -> Average System Throughput: {avg_fps:.1f} FPS")

    out_gif = os.path.join(base_dir, "public", "output", "black_hole_realtime_demo.gif")
    print(f"[*] Compiling Kerr Black Hole demonstration GIF to: {out_gif}")
    small_frames = [f.resize((640, 360), Image.Resampling.BILINEAR) for f in frames]
    small_frames[0].save(
        out_gif,
        save_all=True,
        append_images=small_frames[1:],
        duration=50,
        loop=0
    )
    print(f"[✓] Animated demonstration saved ({os.path.getsize(out_gif):,} bytes)")

if __name__ == "__main__":
    main()
