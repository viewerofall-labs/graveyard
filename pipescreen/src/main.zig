//! pipescreen — idle-triggered pipes screensaver + suspend for Niri/Wayland.
//!
//!   pipescreen                run the daemon
//!   pipescreen render         draw the pipes screensaver to this terminal
//!   pipescreen trigger        show the screensaver now
//!   pipescreen stop           hide the screensaver now
//!   pipescreen awake [on|off] keep the screen awake (no pipes, no suspend)
//!   pipescreen pipes  [on|off]toggle the screensaver
//!   pipescreen suspend[on|off]toggle auto-suspend
//!   pipescreen lock [on|off]  toggle lock (woven-lock) before suspend
//!   pipescreen status         print current state

const std = @import("std");
const linux = std.os.linux;
const idle = @import("idle.zig");
const ipc = @import("ipc.zig");
const pipes = @import("pipes.zig");
const dbus = @import("dbus.zig");

// --- knobs -------------------------------------------------------------
const PIPES_MS: u32 = 5 * 60 * 1000; // show pipes after 5 min idle
const SUSPEND_MS: u32 = 15 * 60 * 1000; // suspend after 15 min idle
const ACTIVITY_MS: u32 = 3 * 1000; // resume granularity for dismissing the saver
const SUSPEND_DEFAULT = true; // initial state of auto-suspend
const LOCK_DEFAULT = true; // initial state of lock-before-suspend
const LOCK_CMD = "woven-lock"; // global wayland locker (no args, releases on exit)
const LOCK_SETTLE_MS = 400; // let the locker map before we suspend
// -----------------------------------------------------------------------

const Daemon = struct {
    io: std.Io,
    exe: []const u8,
    saver: ?std.process.Child = null,
    watcher: ?dbus.Watcher = null,

    inhibit: bool = false, // keep-awake: ignore all idle
    pipes_on: bool = true,
    suspend_on: bool = SUSPEND_DEFAULT,
    lock_on: bool = LOCK_DEFAULT,
    locker: ?std.process.Child = null,

    pipes_notif: u32 = 0,
    suspend_notif: u32 = 0,

    fn startSaver(self: *Daemon) void {
        if (self.saver != null) return;
        self.saver = std.process.spawn(self.io, .{
            .argv = &.{ "kitty", "--class", "pipescreen", "--start-as=fullscreen", "-e", self.exe, "render" },
        }) catch |e| {
            std.debug.print("pipescreen: failed to launch saver: {s}\n", .{@errorName(e)});
            return;
        };
    }

    fn stopSaver(self: *Daemon) void {
        if (self.saver) |*c| {
            c.kill(self.io); // force-terminate + reap, idempotent
            self.saver = null;
        }
    }

    fn suspendSystem(self: *Daemon) void {
        if (self.lock_on) {
            // Reap the previous locker (it exited when the user last unlocked),
            // then lock and give it a moment to map before we suspend.
            if (self.locker) |*l| {
                _ = l.wait(self.io) catch {};
                self.locker = null;
            }
            self.locker = std.process.spawn(self.io, .{ .argv = &.{LOCK_CMD} }) catch |e| blk: {
                std.debug.print("pipescreen: lock failed: {s}\n", .{@errorName(e)});
                break :blk null;
            };
            if (self.locker != null) settle(LOCK_SETTLE_MS);
        }
        var c = std.process.spawn(self.io, .{ .argv = &.{ "systemctl", "suspend" } }) catch |e| {
            std.debug.print("pipescreen: suspend failed: {s}\n", .{@errorName(e)});
            return;
        };
        _ = c.wait(self.io) catch {};
    }

    fn onEvent(ctx: *anyopaque, ev: idle.Event) void {
        const self: *Daemon = @ptrCast(@alignCast(ctx));
        switch (ev.kind) {
            // Any resume dismisses the saver. The short activity notification
            // guarantees a resume fires soon after the user returns.
            .resumed => self.stopSaver(),
            .idled => {
                if (self.inhibit) return;
                if (ev.notif == self.pipes_notif and self.pipes_on) self.startSaver();
                if (ev.notif == self.suspend_notif and self.suspend_on) self.suspendSystem();
            },
        }
    }

    fn onDbusSignal(ctx: *anyopaque, preparing: bool) void {
        const self: *Daemon = @ptrCast(@alignCast(ctx));
        if (preparing and self.lock_on) {
            // System is about to sleep; lock it (don't call suspend, logind will)
            if (self.locker) |*l| {
                _ = l.wait(self.io) catch {};
                self.locker = null;
            }
            self.locker = std.process.spawn(self.io, .{ .argv = &.{LOCK_CMD} }) catch |e| blk: {
                std.debug.print("pipescreen: lock failed: {s}\n", .{@errorName(e)});
                break :blk null;
            };
            if (self.locker != null) settle(LOCK_SETTLE_MS);
        }
    }

    fn handleIpc(self: *Daemon, lfd: i32) void {
        while (ipc.accept(lfd)) |cfd| {
            var inbuf: [256]u8 = undefined;
            const cmd = ipc.recvLine(cfd, &inbuf);
            var outbuf: [256]u8 = undefined;
            ipc.reply(cfd, self.applyCommand(cmd, &outbuf));
            ipc.close(cfd);
        }
    }

    fn applyCommand(self: *Daemon, cmd: []const u8, buf: []u8) []const u8 {
        var it = std.mem.tokenizeScalar(u8, cmd, ' ');
        const verb = it.next() orelse return "err: empty\n";
        const arg = it.next();

        if (std.mem.eql(u8, verb, "trigger")) {
            self.startSaver();
            return "ok: triggered\n";
        } else if (std.mem.eql(u8, verb, "stop")) {
            self.stopSaver();
            return "ok: stopped\n";
        } else if (std.mem.eql(u8, verb, "awake")) {
            self.inhibit = setFlag(self.inhibit, arg);
            if (self.inhibit) self.stopSaver();
            return onoff(buf, "awake", self.inhibit);
        } else if (std.mem.eql(u8, verb, "pipes")) {
            self.pipes_on = setFlag(self.pipes_on, arg);
            return onoff(buf, "pipes", self.pipes_on);
        } else if (std.mem.eql(u8, verb, "suspend")) {
            self.suspend_on = setFlag(self.suspend_on, arg);
            return onoff(buf, "suspend", self.suspend_on);
        } else if (std.mem.eql(u8, verb, "lock")) {
            self.lock_on = setFlag(self.lock_on, arg);
            return onoff(buf, "lock", self.lock_on);
        } else if (std.mem.eql(u8, verb, "status")) {
            return std.fmt.bufPrint(buf, "awake={} pipes={} suspend={} lock={} showing={}\n", .{
                self.inhibit, self.pipes_on, self.suspend_on, self.lock_on, self.saver != null,
            }) catch "err: fmt\n";
        }
        return "err: unknown command\n";
    }
};

/// Resolve "on"/"off"/null(toggle) into the next boolean value.
fn setFlag(current: bool, arg: ?[]const u8) bool {
    const a = arg orelse return !current;
    if (std.mem.eql(u8, a, "on")) return true;
    if (std.mem.eql(u8, a, "off")) return false;
    return !current;
}

fn onoff(buf: []u8, name: []const u8, on: bool) []const u8 {
    return std.fmt.bufPrint(buf, "{s}: {s}\n", .{ name, if (on) "on" else "off" }) catch "err: fmt\n";
}

fn settle(ms: u64) void {
    var req = linux.timespec{ .sec = @intCast(ms / 1000), .nsec = @intCast((ms % 1000) * 1_000_000) };
    _ = linux.nanosleep(&req, null);
}

fn connectWayland(rt: []const u8, display: ?[]const u8) !idle.Client {
    var buf: [linux.PATH_MAX]u8 = undefined;
    if (display) |wl| {
        const p = if (wl.len > 0 and wl[0] == '/')
            wl
        else
            try std.fmt.bufPrint(&buf, "{s}/{s}", .{ rt, wl });
        if (idle.Client.connect(p)) |c| return c else |_| {}
    }
    // Fall back to scanning the runtime dir — robust under systemd where
    // WAYLAND_DISPLAY may not be exported into the user manager.
    var i: u32 = 0;
    while (i < 33) : (i += 1) {
        const p = std.fmt.bufPrint(&buf, "{s}/wayland-{d}", .{ rt, i }) catch continue;
        if (idle.Client.connect(p)) |c| return c else |_| {}
    }
    return error.NoWayland;
}

pub fn main(init: std.process.Init) !void {
    const gpa = init.gpa;
    const io = init.io;

    const rt = init.environ_map.get("XDG_RUNTIME_DIR") orelse return error.NoRuntimeDir;
    var ctl_buf: [linux.PATH_MAX]u8 = undefined;
    const ctl = try std.fmt.bufPrint(&ctl_buf, "{s}/pipescreen.sock", .{rt});

    // --- argument dispatch ---
    var it = init.minimal.args.iterate();
    _ = it.next(); // argv[0]
    if (it.next()) |first| {
        if (std.mem.eql(u8, first, "render")) {
            pipes.run();
            return;
        }
        // anything else is a control command: join argv[1..] and send it
        var cmd_buf: [256]u8 = undefined;
        var n: usize = 0;
        @memcpy(cmd_buf[0..first.len], first);
        n = first.len;
        while (it.next()) |a| {
            if (n + 1 + a.len > cmd_buf.len) break;
            cmd_buf[n] = ' ';
            n += 1;
            @memcpy(cmd_buf[n..][0..a.len], a);
            n += a.len;
        }
        var resp_buf: [256]u8 = undefined;
        const resp = ipc.send(ctl, cmd_buf[0..n], &resp_buf) catch {
            std.debug.print("pipescreen: daemon not running?\n", .{});
            return;
        };
        _ = linux.write(1, resp.ptr, resp.len);
        return;
    }

    // --- daemon ---
    var client = try connectWayland(rt, init.environ_map.get("WAYLAND_DISPLAY"));
    const watcher_opt = dbus.Watcher.start(io) catch |e| blk: {
        std.debug.print("pipescreen: dbus watcher failed: {s}\n", .{@errorName(e)});
        break :blk null;
    };

    var dmn = Daemon{
        .io = io,
        .exe = try std.process.executablePathAlloc(io, gpa),
        .watcher = watcher_opt,
    };
    defer gpa.free(dmn.exe);

    dmn.pipes_notif = try client.addNotification(PIPES_MS);
    dmn.suspend_notif = try client.addNotification(SUSPEND_MS);
    // _ = try client.addNotification(ACTIVITY_MS); // resume detector; idled ignored — DISABLED FOR DEBUG
    client.setNonblock();

    const lfd = try ipc.listen(ctl);

    var pfds = [_]linux.pollfd{
        .{ .fd = client.fd, .events = linux.POLL.IN, .revents = 0 },
        .{ .fd = lfd, .events = linux.POLL.IN, .revents = 0 },
        .{ .fd = if (dmn.watcher != null) dmn.watcher.?.fd else -1, .events = linux.POLL.IN, .revents = 0 },
    };
    while (true) {
        _ = linux.poll(&pfds, pfds.len, -1);
        if (pfds[0].revents != 0) try client.drain(@ptrCast(&dmn), Daemon.onEvent);
        if (pfds[1].revents != 0) dmn.handleIpc(lfd);
        if (pfds[2].fd != -1 and pfds[2].revents != 0) {
            if (dmn.watcher) |*w| {
                w.drain(
                    @ptrCast(&dmn),
                    Daemon.onDbusSignal,
                ) catch {};
            }
        }
    }
}
