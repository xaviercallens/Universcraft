import os

source_path = r"D:\xdev\UniversCraft\holo_engine\src\client\advanced_scenes.rs"
assets_dir = r"D:\xdev\UniversCraft\holo_engine\assets\shaders"

os.makedirs(assets_dir, exist_ok=True)

with open(source_path, 'r', encoding='utf-8') as f:
    lines = f.readlines()

out_lines = []
shaders = [
    "ocean_sunset",
    "desert_dunes",
    "ice_glacier",
    "black_hole",
    "earth_orbit",
    "cloudscape",
]

in_shader = False
current_shader = None
shader_lines = []

i = 0
while i < len(lines):
    line = lines[i]
    if not in_shader:
        out_lines.append(line)
        for s in shaders:
            if f'if scene_id == "{s}" {{' in line:
                current_shader = s
                i += 1
                if 'return r#"' in lines[i]:
                    in_shader = True
                    shader_lines = []
                    out_lines.append(f'        return include_str!("../../assets/shaders/scene_{s}.wgsl").to_string();\n')
                else:
                    out_lines.append(lines[i])
                break
    else:
        if '"#.to_string();' in line:
            in_shader = False
            with open(os.path.join(assets_dir, f"scene_{current_shader}.wgsl"), 'w', encoding='utf-8', newline='\n') as sf:
                content = "".join(shader_lines)
                if content.startswith("\n"):
                    content = content[1:]
                sf.write(content)
        else:
            shader_lines.append(line)
    i += 1

with open(source_path, 'w', encoding='utf-8') as f:
    f.writelines(out_lines)

print("Extraction complete.")
