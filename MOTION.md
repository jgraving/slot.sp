# Physical UI motion

Menus are persistent surfaces. Opening brings a surface up from below the display; closing returns it along that path. The surface, labels, focus plate, controls and legends move together. They remain opaque and are retired only after they have left the screen. Device chrome stays anchored.

All menu positions use the analytic damped spring in `slot-ui/src/expressive.rs`:

- Panel entry and exit: stiffness 800, critical damping (1.0). Boundaries settle without crossing the status bar.
- Card expansion: stiffness 800, critical damping, retaining its current velocity on reversal.
- Page travel: stiffness 380 and damping ratio 0.85. Back reverses the current two-page trajectory without resetting either page.
- Focus, switches and sliders: fast spatial response, stiffness 800 and damping ratio 0.76, with a small elastic settle.

These are positions and velocities, not frame counters. Reversing a target preserves both. Input stays active during motion; navigation does not wait for animations. Time steps use the same closed-form solution at 30, 60 and 120 Hz, including dropped frames.

Neighbouring pages travel through a shared surface instead of crossfading. Compact pause cards expand into full settings while their pages travel. No menu transparency animation substitutes for movement. Cache the last custom page and its controls until its closing surface has left the display.

About and the clock editor obey the same sheet rules. Their retained layers are cached draw primitives, not framebuffer copies. HUD and feedback pills also keep their ink opacity while moving; settings feedback retains its spring position and velocity when another notice rearms it or navigation dismisses it.
