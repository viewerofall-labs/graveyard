#define _POSIX_C_SOURCE 200809L
#include <unistd.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <GL/glew.h>
#include <GL/glx.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdint.h>
#include <signal.h>
#include <time.h>

// X11 globals
static Display *display = NULL;
static Window window = 0;
static GLXContext glx_context = NULL;
static int width = 1920, height = 1080;
static volatile int running = 1;

// GL globals
static GLuint shader_program = 0;
static GLuint vao = 0, vbo = 0;

void sig_exit(int sig) { running = 0; }

// ============================================================================
// OpenGL Shader
// ============================================================================

static const char *vertex_shader =
"#version 330\n"
"layout(location = 0) in vec2 pos;\n"
"void main() { gl_Position = vec4(pos, 0.0, 1.0); }\n";

static const char *fragment_shader =
"#version 330\n"
"uniform float time;\n"
"uniform vec2 resolution;\n"
"out vec4 FragColor;\n"
"\n"
"uint xorshift32(inout uint state) {\n"
"    state ^= state << uint(13);\n"
"    state ^= state >> uint(17);\n"
"    state ^= state << uint(5);\n"
"    return state;\n"
"}\n"
"\n"
"void main() {\n"
"    vec2 uv = gl_FragCoord.xy / resolution;\n"
"    vec3 col = vec3(0.0);\n"
"    \n"
"    uint seed = uint(time * 1000.0) + uint(gl_FragCoord.x + gl_FragCoord.y * 1920.0);\n"
"    \n"
"    // Scanline glitch\n"
"    if (mod(time * 60.0, 8.0) < 1.0 && mod(gl_FragCoord.y, 3.0) < 1.0) {\n"
"        col = vec3(xorshift32(seed) % 256u) / 255.0;\n"
"    }\n"
"    \n"
"    // Chromatic aberration\n"
"    float shift = sin(time * 2.0 + uv.y * 10.0) * 0.02;\n"
"    col.r += sin(time + uv.x) * 0.3;\n"
"    col.g += sin(time + uv.x - 2.0) * 0.15;\n"
"    col.b += sin(time + uv.x - 4.0) * 0.3;\n"
"    \n"
"    // Horizontal tears\n"
"    float tear = sin(time * 2.0) * 0.2 + 0.5;\n"
"    if (abs(uv.y - tear) < 0.02) {\n"
"        col = vec3(1.0, 0.0, 1.0);\n"
"    }\n"
"    \n"
"    // Random bit corruption\n"
"    if (mod(xorshift32(seed), 50u) < 1u) {\n"
"        col = vec3(xorshift32(seed) % 256u) / 255.0;\n"
"    }\n"
"    \n"
"    FragColor = vec4(col, 1.0);\n"
"}\n";

static GLuint compile_shader(const char *src, GLenum type) {
    GLuint shader = glCreateShader(type);
    glShaderSource(shader, 1, &src, NULL);
    glCompileShader(shader);

    GLint success;
    glGetShaderiv(shader, GL_COMPILE_STATUS, &success);
    if (!success) {
        GLchar log[512];
        glGetShaderInfoLog(shader, 512, NULL, log);
        fprintf(stderr, "Shader compile error: %s\n", log);
        glDeleteShader(shader);
        return 0;
    }
    return shader;
}

static int gl_init() {
    GLuint vs = compile_shader(vertex_shader, GL_VERTEX_SHADER);
    GLuint fs = compile_shader(fragment_shader, GL_FRAGMENT_SHADER);
    if (!vs || !fs) return 0;

    shader_program = glCreateProgram();
    glAttachShader(shader_program, vs);
    glAttachShader(shader_program, fs);
    glLinkProgram(shader_program);

    GLint success;
    glGetProgramiv(shader_program, GL_LINK_STATUS, &success);
    if (!success) {
        GLchar log[512];
        glGetProgramInfoLog(shader_program, 512, NULL, log);
        fprintf(stderr, "Program link error: %s\n", log);
        return 0;
    }

    glDeleteShader(vs);
    glDeleteShader(fs);

    // Fullscreen quad
    float quad[] = {
        -1.0f,  1.0f,
        -1.0f, -1.0f,
        1.0f, -1.0f,
        1.0f,  1.0f,
    };

    glGenVertexArrays(1, &vao);
    glGenBuffers(1, &vbo);
    glBindVertexArray(vao);
    glBindBuffer(GL_ARRAY_BUFFER, vbo);
    glBufferData(GL_ARRAY_BUFFER, sizeof(quad), quad, GL_STATIC_DRAW);

    glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 8, (void *)0);
    glEnableVertexAttribArray(0);

    glBindBuffer(GL_ARRAY_BUFFER, 0);
    glBindVertexArray(0);

    return 1;
}

// ============================================================================
// X11 Setup
// ============================================================================

static int x11_init() {
    display = XOpenDisplay(NULL);
    if (!display) {
        fprintf(stderr, "Failed to open X11 display\n");
        return 0;
    }

    int screen = DefaultScreen(display);
    Window root = RootWindow(display, screen);
    int screen_width = DisplayWidth(display, screen);
    int screen_height = DisplayHeight(display, screen);

    width = screen_width;
    height = screen_height;

    // Find GLX visual
    int glx_attribs[] = {
        GLX_RGBA,
        GLX_DOUBLEBUFFER,
        GLX_RED_SIZE, 8,
        GLX_GREEN_SIZE, 8,
        GLX_BLUE_SIZE, 8,
        GLX_ALPHA_SIZE, 8,
        GLX_DEPTH_SIZE, 24,
        None
    };

    XVisualInfo *visual = glXChooseVisual(display, screen, glx_attribs);
    if (!visual) {
        fprintf(stderr, "Failed to find GLX visual\n");
        return 0;
    }

    // Create colormap
    Colormap colormap = XCreateColormap(display, root, visual->visual, AllocNone);

    // Create window
    XSetWindowAttributes attrs = {
        .colormap = colormap,
        .background_pixel = 0,
        .border_pixel = 0,
        .override_redirect = True,  // No window decorations, always on top
    };

    window = XCreateWindow(display, root, 0, 0, width, height, 0,
                           visual->depth, InputOutput, visual->visual,
                           CWColormap | CWBackPixel | CWBorderPixel | CWOverrideRedirect,
                           &attrs);

    if (!window) {
        fprintf(stderr, "Failed to create X11 window\n");
        return 0;
    }

    // Grab input (so we can detect ESC)
    XSelectInput(display, window, KeyPressMask);

    // Map window
    XMapWindow(display, window);
    XRaiseWindow(display, window);

    // Create GLX context
    glx_context = glXCreateContext(display, visual, NULL, GL_TRUE);
    if (!glx_context) {
        fprintf(stderr, "Failed to create GLX context\n");
        return 0;
    }

    if (!glXMakeCurrent(display, window, glx_context)) {
        fprintf(stderr, "Failed to make GLX context current\n");
        return 0;
    }

    // Init GLEW
    glewExperimental = GL_TRUE;
    GLenum glew_err = glewInit();
    if (glew_err != GLEW_OK) {
        fprintf(stderr, "GLEW init failed: %s\n", glewGetErrorString(glew_err));
        return 0;
    }

    // Vsync (optional, skip if unavailable)
    // glXSwapIntervalEXT(display, window, 1);

    XFree(visual);
    return 1;
}

static void x11_handle_events() {
    XEvent event;
    while (XPending(display)) {
        XNextEvent(display, &event);
        if (event.type == KeyPress) {
            KeySym key = XLookupKeysym(&event.xkey, 0);
            if (key == XK_Escape) {
                running = 0;
            }
        }
    }
}

// ============================================================================
// Main
// ============================================================================

int main() {
    signal(SIGINT, sig_exit);

    if (!x11_init()) {
        fprintf(stderr, "X11 init failed\n");
        return 1;
    }

    if (!gl_init()) {
        fprintf(stderr, "GL init failed\n");
        return 1;
    }

    printf("GPU Glitch X11 Overlay Running\n");
    printf("Press ESC to exit\n");

    float time = 0.0f;
    while (running) {
        glClearColor(0.0f, 0.0f, 0.0f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT);

        glUseProgram(shader_program);
        glUniform1f(glGetUniformLocation(shader_program, "time"), time);
        glUniform2f(glGetUniformLocation(shader_program, "resolution"), (float)width, (float)height);

        glBindVertexArray(vao);
        glDrawArrays(GL_TRIANGLE_STRIP, 0, 4);

        glXSwapBuffers(display, window);

        x11_handle_events();

        time += 0.016f;  // ~60fps
        struct timespec ts = {0, 16667000};
        nanosleep(&ts, NULL);
    }

    // Cleanup
    glDeleteProgram(shader_program);
    glDeleteBuffers(1, &vbo);
    glDeleteVertexArrays(1, &vao);
    glXDestroyContext(display, glx_context);
    XDestroyWindow(display, window);
    XCloseDisplay(display);

    return 0;
}
