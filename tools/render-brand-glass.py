"""Render actual, beveled variable-font glass for slot.sp.

blender -b --python tools/render-brand-glass.py -- GEOMETRY.json OUTDIR
    [--frames 6,24,40] [--samples 64] [--threads 8] [--width 480] [--height 128]

Input: {"frames":[{"t":0.0,"glyphs":[{"name":"s","contours":[[[x,y],...],...]}]}]}
Contours are already shaped/positioned in common word coordinates, Y-up. Preserve
font winding so counters remain holes. One font em should be approximately 1.0;
word width ~3.16 and visible height ~0.93. An optional repeated closing point is
removed. Output is straight RGBA numbered 0000.png, etc. Untouched frame numbers
make preview renders easy to inspect without confusing their position in time.

A dark studio environment, large teal/violet softboxes and one neutral travelling
strip give polished glass a reflected world. Its actual volume has absorption,
transmission, double reflections and real sidewalls. No alpha-outline trick.
"""
import argparse
import json
import math
import os
import sys
from pathlib import Path

import bpy
from mathutils import Vector


def args():
    p = argparse.ArgumentParser()
    p.add_argument("geometry")
    p.add_argument("output")
    p.add_argument("--frames", default="")
    p.add_argument("--samples", type=int, default=64)
    p.add_argument("--threads", type=int, default=min(12, os.cpu_count() or 4))
    p.add_argument("--width", type=int, default=480)
    p.add_argument("--height", type=int, default=128)
    return p.parse_args(sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else [])


def smooth(a, b, t):
    u = max(0.0, min(1.0, (t-a)/(b-a)))
    return u*u*(3.0-2.0*u)


def aim(obj, target=(0.0, 0.0, 0.0)):
    obj.rotation_euler = (Vector(target)-obj.location).to_track_quat('-Z', 'Y').to_euler()


def area(name, position, energy, color, size, size_y):
    data = bpy.data.lights.new(name, 'AREA')
    data.shape = 'RECTANGLE'
    data.energy = energy
    data.color = color
    data.size = size
    data.size_y = size_y
    obj = bpy.data.objects.new(name, data)
    bpy.context.collection.objects.link(obj)
    obj.location = position
    aim(obj)
    return obj


def material():
    m = bpy.data.materials.new('Optical glass — teal/violet absorption')
    m.use_nodes = True
    n = m.node_tree.nodes
    n.clear()
    links = m.node_tree.links
    out = n.new('ShaderNodeOutputMaterial')
    out.location = (780, 80)

    glass = n.new('ShaderNodeBsdfGlass')
    glass.name = 'Polished optical glass'
    glass.distribution = 'BECKMANN'
    glass.inputs['Color'].default_value = (0.96, 0.99, 1.0, 1.0)
    glass.inputs['Roughness'].default_value = 0.038
    glass.inputs['IOR'].default_value = 1.45
    glass.location = (250, 150)

    # The terminal surface becomes a softly lit manufactured white mark. Root's
    # atlas compositor may replace its very last frame with an exact flat mask.
    porcelain = n.new('ShaderNodeBsdfPrincipled')
    porcelain.name = 'Warm white terminal mark'
    porcelain.location = (230, -90)
    porcelain.inputs['Base Color'].default_value = (0.925, 0.94, 0.90, 1.0)
    porcelain.inputs['Roughness'].default_value = 0.25
    if 'Specular IOR Level' in porcelain.inputs:
        porcelain.inputs['Specular IOR Level'].default_value = 0.25
    elif 'Specular' in porcelain.inputs:
        porcelain.inputs['Specular'].default_value = 0.25
    mix = n.new('ShaderNodeMixShader')
    mix.name = 'Material lock-in'
    mix.location = (570, 120)
    links.new(glass.outputs[0], mix.inputs[1])
    links.new(porcelain.outputs[0], mix.inputs[2])
    links.new(mix.outputs[0], out.inputs['Surface'])

    # Position is shared by all letters; dye evolves across the word rather
    # than resetting at every character (which makes materials look synthetic).
    geo = n.new('ShaderNodeNewGeometry')
    geo.location = (-750, -240)
    sep = n.new('ShaderNodeSeparateXYZ')
    sep.location = (-560, -240)
    links.new(geo.outputs['Position'], sep.inputs[0])
    remap = n.new('ShaderNodeMapRange')
    remap.location = (-370, -240)
    remap.inputs['From Min'].default_value = -1.8
    remap.inputs['From Max'].default_value = 1.8
    remap.inputs['To Min'].default_value = 0.0
    remap.inputs['To Max'].default_value = 1.0
    if hasattr(remap, 'clamp'):
        remap.clamp = True
    links.new(sep.outputs['X'], remap.inputs['Value'])
    dye = n.new('ShaderNodeMixRGB')
    dye.blend_type = 'MIX'
    dye.location = (-120, -280)
    dye.inputs[1].default_value = (0.32, 0.88, 0.82, 1.0)
    dye.inputs[2].default_value = (0.66, 0.42, 0.94, 1.0)
    links.new(remap.outputs['Result'], dye.inputs[0])
    absorption = n.new('ShaderNodeVolumeAbsorption')
    absorption.name = 'Real glass thickness absorption'
    absorption.location = (350, -380)
    absorption.inputs['Density'].default_value = 0.9
    links.new(dye.outputs[0], absorption.inputs['Color'])
    links.new(absorption.outputs[0], out.inputs['Volume'])
    return m, mix, glass, absorption


def build_glyph(glyph, index, parent, mat, t):
    curve = bpy.data.curves.new('Glyph %02d %s' % (index, glyph.get('name', '')), 'CURVE')
    curve.dimensions = '2D'
    curve.fill_mode = 'BOTH'
    curve.resolution_u = 1
    curve.render_resolution_u = 1
    flatten = 1.0 - smooth(0.82, 0.96, t)
    curve.extrude = 0.070 * flatten
    curve.bevel_depth = (0.012 + 0.008 * smooth(0.0, 0.30, t)) * flatten
    curve.bevel_resolution = 4
    curve.use_fill_caps = True
    valid = False
    for contour in glyph.get('contours', []):
        points = [[float(p[0]), float(p[1])] for p in contour]
        if len(points) > 1 and abs(points[0][0]-points[-1][0]) + abs(points[0][1]-points[-1][1]) < 1e-8:
            points.pop()
        if len(points) < 3:
            continue
        spline = curve.splines.new('POLY')
        spline.points.add(len(points)-1)
        coordinates = []
        for x, y in points:
            coordinates.extend((x, y, 0.0, 1.0))
        spline.points.foreach_set('co', coordinates)
        spline.use_cyclic_u = True
        valid = True
    if not valid:
        bpy.data.curves.remove(curve)
        return None
    obj = bpy.data.objects.new(curve.name, curve)
    bpy.context.collection.objects.link(obj)
    obj.parent = parent
    curve.materials.append(mat)
    return obj


def clear_glyphs(objects):
    for obj in objects:
        curve = obj.data
        bpy.data.objects.remove(obj, do_unlink=True)
        if curve.users == 0:
            bpy.data.curves.remove(curve)


def main():
    a = args()
    frames = json.loads(Path(a.geometry).read_text())['frames']
    if not frames:
        raise ValueError('No geometry frames supplied')
    selected = [int(x) for x in a.frames.split(',') if x.strip()] if a.frames else list(range(len(frames)))
    if not selected or any(i < 0 or i >= len(frames) for i in selected):
        raise ValueError('Requested frame is outside supplied geometry')
    output = Path(a.output)
    output.mkdir(parents=True, exist_ok=True)

    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete(use_global=False)
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    scene.cycles.device = 'CPU'
    scene.cycles.samples = a.samples
    scene.cycles.use_denoising = False
    if hasattr(scene.cycles, 'denoiser'):
        scene.cycles.denoiser = 'OPENIMAGEDENOISE'
    scene.cycles.max_bounces = 12
    scene.cycles.diffuse_bounces = 2
    scene.cycles.glossy_bounces = 5
    scene.cycles.transmission_bounces = 8
    scene.cycles.transparent_max_bounces = 8
    scene.cycles.volume_bounces = 0
    scene.cycles.sample_clamp_indirect = 3.0
    scene.cycles.seed = 17
    if hasattr(scene.cycles, 'use_animated_seed'):
        scene.cycles.use_animated_seed = False
    scene.render.threads_mode = 'FIXED'
    scene.render.threads = a.threads
    scene.render.resolution_x = a.width
    scene.render.resolution_y = a.height
    scene.render.resolution_percentage = 100
    scene.render.film_transparent = True
    # Keep refracted/reflected studio illumination in glass pixels. Transparent
    # glass film would strip the interior down to an outline, the old failure.
    if hasattr(scene.cycles, 'film_transparent_glass'):
        scene.cycles.film_transparent_glass = False
    scene.render.image_settings.file_format = 'PNG'
    scene.render.image_settings.color_mode = 'RGBA'
    scene.render.image_settings.color_depth = '8'
    scene.render.image_settings.compression = 15
    scene.render.filter_size = 1.1
    scene.view_settings.view_transform = 'AgX'
    try:
        scene.view_settings.look = 'AgX - Medium High Contrast'
    except (TypeError, ValueError):
        pass
    scene.view_settings.exposure = 0.0
    scene.view_settings.gamma = 1.0

    world = bpy.data.worlds.new('Dark optical studio')
    world.use_nodes = True
    world.node_tree.nodes['Background'].inputs['Color'].default_value = (0.09, 0.13, 0.20, 1.0)
    world.node_tree.nodes['Background'].inputs['Strength'].default_value = 0.30
    scene.world = world

    camera_data = bpy.data.cameras.new('Orthographic product camera')
    camera_data.type = 'ORTHO'
    camera = bpy.data.objects.new('Orthographic product camera', camera_data)
    bpy.context.collection.objects.link(camera)
    camera.location = (0.0, 0.0, 7.0)
    aim(camera)
    scene.camera = camera

    word = bpy.data.objects.new('Single coherent optical object', None)
    bpy.context.collection.objects.link(word)
    mat, material_lock, glass, absorption = material()

    # Area emitters are visible in reflection but do not paint luminous shapes
    # over the transparent background. A neutral key preserves believable glass.
    area('Large cool neutral key', (-1.7, 2.5, 3.0), 210.0, (0.83, 0.94, 1.0), 3.4, 0.75)
    area('Teal transmitted edge', (-3.4, -0.65, 1.0), 170.0, (0.08, 0.72, 0.72), 2.1, 3.3)
    area('Violet opposing edge', (3.2, 0.8, 1.6), 210.0, (0.48, 0.20, 1.0), 2.2, 3.7)
    area('Lower neutral card', (0.2, -2.5, 2.2), 75.0, (0.72, 0.85, 0.95), 3.8, 0.25)
    strip = area('Travelling white softbox', (-2.0, 0.5, 3.7), 165.0, (1.0, 0.98, 0.94), 0.24, 3.4)

    # Guard extreme supplied poses. Orthographic scale describes camera width
    # in the landscape render; 4.6 yields a ~330px final word as requested.
    xs, ys = [], []
    for f in frames:
        for g in f['glyphs']:
            for contour in g.get('contours', []):
                xs.extend(float(p[0]) for p in contour)
                ys.extend(float(p[1]) for p in contour)
    if not xs:
        raise ValueError('No contour points supplied')
    aspect = a.width / a.height
    safe_scale = max(4.6, (max(xs)-min(xs))*1.18, ((max(ys)-min(ys))*1.16 + 0.15)*aspect)
    current = []
    for index in selected:
        f = frames[index]
        t = float(f.get('t', index/max(1,len(frames)-1)))
        t = max(0.0, min(1.0, t))
        clear_glyphs(current)
        current = []
        for j, glyph in enumerate(f['glyphs']):
            obj = build_glyph(glyph, j, word, mat, t)
            if obj:
                current.append(obj)
        # One slow, directed optical turn; no independent character bobbing.
        approach = 1.0-smooth(0.04, 0.88, t)
        word.rotation_euler = (math.radians(7.0)*approach,
                               math.radians(-18.0)*approach,
                               math.radians(-1.5)*approach)
        word.location = (0.0, -0.015*approach, 0.0)
        camera_data.ortho_scale = safe_scale*(1.0+0.035*approach)
        sweep = smooth(0.08, 0.83, t)
        strip.location = (-2.3+4.6*sweep, 0.5-0.25*sweep, 3.7)
        aim(strip)
        # Leave glass on screen long enough to read as a real object. The
        # terminal material change is brief and coincides with geometric rest.
        lock = smooth(0.84, 0.96, t)
        material_lock.inputs[0].default_value = lock
        glass.inputs['Roughness'].default_value = 0.038 + 0.025*lock
        absorption.inputs['Density'].default_value = 0.9*(1-lock)
        scene.render.filepath = str(output / ('%04d.png' % index))
        print('BRAND_FRAME %d/%d t=%.4f -> %s' % (index,len(frames),t,scene.render.filepath), flush=True)
        bpy.ops.render.render(write_still=True)
    print('BRAND_RENDER_COMPLETE', flush=True)


if __name__ == '__main__':
    main()
