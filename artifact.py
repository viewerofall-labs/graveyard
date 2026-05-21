import gi
import cairo
import signal
import random

gi.require_version('Gtk', '3.0')
gi.require_version('GtkLayerShell', '0.1')
from gi.repository import Gtk, GtkLayerShell, Gdk, GLib

def on_draw(widget, cr):
    # Wipe the frame clean (transparent)
    cr.set_source_rgba(0, 0, 0, 0)
    cr.set_operator(cairo.OPERATOR_SOURCE)
    cr.paint()
    
    cr.set_operator(cairo.OPERATOR_OVER)

    # Introduce a stutter (50% chance to draw nothing this frame)
    # This makes it feel like a struggling hardware clock, not a smooth animation.
    if random.random() > 0.5:
        return False

    width = widget.get_allocated_width()
    height = widget.get_allocated_height()

    # Randomly pick which type of hardware failure to simulate this frame
    glitch_mode = random.choice(["vram_dots", "tearing", "tearing", "minor_corruption"])

    if glitch_mode == "vram_dots":
        # Simulates the "Space Invaders" purple dot grid (Image 2)
        # We draw this in the top half of the screen usually, in a rigid grid.
        cr.set_source_rgba(0.9, 0.2, 0.8, 0.9)  # Hot Purple/Magenta
        
        spacing = 16 # Grid size
        start_y = random.choice([0, int(height/4)]) 
        end_y = start_y + int(height / 2)
        
        for x in range(0, width, spacing):
            for y in range(start_y, end_y, spacing):
                # 30% chance a dot appears in this grid cell
                if random.random() > 0.7:
                    cr.rectangle(x, y, 4, 4)
        cr.fill() # Batch fill for performance

    elif glitch_mode == "tearing":
        # Simulates the blocky horizontal digital tearing (Image 1)
        # Pick a random horizontal band on the screen
        band_y = random.randint(0, height - 200)
        band_h = random.randint(50, 200)
        
        # Draw clusters of corrupted macroblocks in this band
        for _ in range(random.randint(40, 100)):
            bx = random.randint(0, width)
            by = band_y + random.randint(0, band_h)
            
            # Glitch blocks are usually short and wide
            bw = random.randint(20, 150)
            bh = random.randint(2, 12)
            
            # Alternate between Lime Green, Magenta, and pure White/Grey noise
            color_choice = random.random()
            if color_choice > 0.6:
                cr.set_source_rgba(0.0, 1.0, 0.0, 0.8) # Lime Green
            elif color_choice > 0.2:
                cr.set_source_rgba(1.0, 0.0, 1.0, 0.8) # Magenta
            else:
                cr.set_source_rgba(0.8, 0.8, 0.8, 0.9) # White/Grey
                
            cr.rectangle(bx, by, bw, bh)
            cr.fill()

    elif glitch_mode == "minor_corruption":
        # Small scattered confetti glitches (just to keep things unpredictable)
        for _ in range(15):
            cr.set_source_rgba(random.random(), random.random(), random.random(), 0.9)
            cr.rectangle(random.randint(0, width), random.randint(0, height), random.randint(5, 20), random.randint(5, 20))
            cr.fill()

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
