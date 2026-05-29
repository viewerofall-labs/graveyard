import gi
import cairo
import signal
import random
import os

gi.require_version('Gtk', '3.0')
gi.require_version('GtkLayerShell', '0.1')
from gi.repository import Gtk, GtkLayerShell, Gdk, GLib

def get_corruption_level():
    try:
        with open('/tmp/artifact.level', 'r') as f:
            return int(f.read().strip())
    except:
        return 0

def render_green_dots(cr, width, height):
    cr.set_source_rgba(0.0, 1.0, 0.0, 0.9)  # Lime green
    spacing = 16
    start_y = random.choice([0, int(height/4)])
    end_y = min(start_y + int(height / 2), height)

    for x in range(0, width, spacing):
        for y in range(start_y, end_y, spacing):
            if random.random() > 0.7:
                cr.rectangle(x, y, 4, 4)
    cr.fill()

def render_tearing(cr, width, height):
    band_y = random.randint(0, max(1, height - 200))
    band_h = random.randint(50, 200)

    for _ in range(random.randint(40, 100)):
        bx = random.randint(0, width)
        by = band_y + random.randint(0, band_h)
        bw = random.randint(20, 150)
        bh = random.randint(2, 12)

        color_choice = random.random()
        if color_choice > 0.6:
            cr.set_source_rgba(0.0, 1.0, 0.0, 0.8)  # Lime Green
        elif color_choice > 0.2:
            cr.set_source_rgba(1.0, 0.0, 1.0, 0.8)  # Magenta
        else:
            cr.set_source_rgba(0.8, 0.8, 0.8, 0.9)  # White/Grey

        cr.rectangle(bx, by, bw, bh)
        cr.fill()

def render_minor_corruption(cr, width, height):
    for _ in range(15):
        cr.set_source_rgba(random.random(), random.random(), random.random(), 0.9)
        cr.rectangle(random.randint(0, width), random.randint(0, height), random.randint(5, 20), random.randint(5, 20))
        cr.fill()

def render_screen_error(cr, width, height):
    cr.set_source_rgba(0.2, 0.2, 0.2, 0.85)
    cr.rectangle(0, 0, width, height)
    cr.fill()

    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0)
    cr.select_font_face("monospace", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
    cr.set_font_size(48)

    msg = random.choice(["Display not found", "GPU not found"])
    cr.move_to(width / 2 - len(msg) * 12, height / 2)
    cr.show_text(msg)

def on_draw(widget, cr):
    # Wipe the frame clean (transparent)
    cr.set_source_rgba(0, 0, 0, 0)
    cr.set_operator(cairo.OPERATOR_SOURCE)
    cr.paint()

    cr.set_operator(cairo.OPERATOR_OVER)

    # 50% chance to skip frame (stuttering)
    if random.random() > 0.5:
        return False

    width = widget.get_allocated_width()
    height = widget.get_allocated_height()

    corruption = get_corruption_level()
    stage = corruption // 20  # 0-5 stages

    # Render based on stage
    if stage >= 1:
        render_green_dots(cr, width, height)

    if stage >= 2:
        render_tearing(cr, width, height)

    if stage >= 3:
        render_tearing(cr, width, height)

    if stage >= 4:
        render_minor_corruption(cr, width, height)

    if stage >= 5:
        render_screen_error(cr, width, height)

    return False

def trigger_redraw(widget):
    widget.queue_draw()
    return True

def main():
    window = Gtk.Window()
    GtkLayerShell.init_for_window(window)
    GtkLayerShell.set_layer(window, GtkLayerShell.Layer.OVERLAY)
    GtkLayerShell.set_namespace(window, "gpu_artifact_sim")

    edges = [
        GtkLayerShell.Edge.TOP,
        GtkLayerShell.Edge.BOTTOM,
        GtkLayerShell.Edge.LEFT,
        GtkLayerShell.Edge.RIGHT
    ]
    for edge in edges:
        GtkLayerShell.set_anchor(window, edge, True)

    screen = window.get_screen()
    visual = screen.get_rgba_visual()
    if visual and screen.is_composited():
        window.set_visual(visual)
    window.override_background_color(Gtk.StateFlags.NORMAL, Gdk.RGBA(0, 0, 0, 0))

    region = cairo.Region() 
    window.input_shape_combine_region(region)
    
    drawing_area = Gtk.DrawingArea()
    window.add(drawing_area)
    drawing_area.connect("draw", on_draw)
    
    # 80ms is roughly 12 FPS. Perfect for a struggling GPU.
    GLib.timeout_add(80, trigger_redraw, drawing_area)

    window.show_all()
    signal.signal(signal.SIGINT, signal.SIG_DFL)
    Gtk.main()

if __name__ == '__main__':
    main()
