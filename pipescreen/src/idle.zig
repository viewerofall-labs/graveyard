//! Minimal hand-rolled Wayland client for ext-idle-notify-v1.
//! No libwayland, no scanner — just the wire protocol over the unix socket,
//! driven with raw linux syscalls so it stays independent of std.Io.
//!
//! We only ever bind wl_seat + ext_idle_notifier_v1 and create idle
//! notifications, none of which transfer file descriptors, so a plain stream
//! socket is enough (no SCM_RIGHTS handling).

const std = @import("std");
const linux = std.os.linux;
const native = @import("builtin").cpu.arch.endian();

const display_id: u32 = 1;

pub const Event = struct {
    notif: u32,
    kind: enum { idled, resumed },
};

pub const Client = struct {
    fd: i32,
    buf: [4096]u8 = undefined,
    len: usize = 0,
    next_id: u32 = 2,

    registry_id: u32 = 0,
    sync_id: u32 = 0,
    sync_done: bool = false,

    seat_name: u32 = 0,
    seat_ver: u32 = 0,
    notifier_name: u32 = 0,
    notifier_ver: u32 = 0,

    seat_id: u32 = 0,
    notifier_id: u32 = 0,

    notifs: [8]u32 = .{0} ** 8,
    notif_count: usize = 0,

    fn alloc(self: *Client) u32 {
        const id = self.next_id;
        self.next_id += 1;
        return id;
    }

    pub fn connect(socket_path: []const u8) !Client {
        const fd: i32 = @intCast(try check(linux.socket(linux.AF.UNIX, linux.SOCK.STREAM, 0)));
        errdefer _ = linux.close(fd);
        var addr: linux.sockaddr.un = undefined;
        addr.family = linux.AF.UNIX;
        @memset(&addr.path, 0);
        if (socket_path.len >= addr.path.len) return error.PathTooLong;
        @memcpy(addr.path[0..socket_path.len], socket_path);
        _ = try check(linux.connect(fd, @ptrCast(&addr), @sizeOf(linux.sockaddr.un)));

        var self = Client{ .fd = fd };

        // get_registry(new_id) -> registry
        self.registry_id = self.alloc();
        var a = ArgBuf{};
        a.u32v(self.registry_id);
        try self.send(display_id, 1, a.slice());

        // roundtrip so every global advertisement has arrived
        try self.roundtrip();

        if (self.notifier_name == 0) return error.NoIdleNotifier;
        if (self.seat_name == 0) return error.NoSeat;

        self.seat_id = self.alloc();
        try self.bind(self.seat_name, "wl_seat", self.seat_ver, self.seat_id);

        self.notifier_id = self.alloc();
        try self.bind(self.notifier_name, "ext_idle_notifier_v1", self.notifier_ver, self.notifier_id);

        return self;
    }

    /// Register an idle timeout (ms). Returns the notification object id,
    /// which is what `Event.notif` will carry.
    pub fn addNotification(self: *Client, timeout_ms: u32) !u32 {
        const id = self.alloc();
        var a = ArgBuf{};
        a.u32v(id); // new_id
        a.u32v(timeout_ms); // timeout
        a.u32v(self.seat_id); // seat
        try self.send(self.notifier_id, 1, a.slice()); // get_idle_notification (opcode 1; 0 is destroy)
        self.notifs[self.notif_count] = id;
        self.notif_count += 1;
        return id;
    }

    /// Block until the next idle/resume event for one of our notifications.
    pub fn next(self: *Client) !Event {
        while (true) {
            if (try self.takeMessage()) |ev| {
                if (ev) |e| return e;
            } else {
                try self.fill();
            }
        }
    }

    /// Switch the socket to non-blocking so the daemon can poll it alongside
    /// the IPC socket. Call after `connect` (which needs blocking reads).
    pub fn setNonblock(self: *Client) void {
        const fl = linux.fcntl(self.fd, linux.F.GETFL, 0);
        _ = linux.fcntl(self.fd, linux.F.SETFL, fl | 0o4000); // O_NONBLOCK
    }

    /// Drain every buffered/ready message (non-blocking), invoking `cb` for
    /// each idle/resume event. Call when poll() reports the fd readable.
    pub fn drain(self: *Client, ctx: *anyopaque, cb: *const fn (*anyopaque, Event) void) !void {
        while (true) {
            while (try self.takeMessage()) |maybe| {
                if (maybe) |ev| cb(ctx, ev);
            }
            if (!try self.fillNb()) return;
        }
    }

    fn fillNb(self: *Client) !bool {
        if (self.len == self.buf.len) return error.BufferOverflow;
        const rc = linux.read(self.fd, self.buf[self.len..].ptr, self.buf.len - self.len);
        const s: isize = @bitCast(rc);
        if (s < 0) {
            if (s == -11) return false; // EAGAIN
            return error.Syscall;
        }
        if (rc == 0) return error.Disconnected;
        self.len += rc;
        return true;
    }

    // ---- internals -------------------------------------------------------

    fn roundtrip(self: *Client) !void {
        self.sync_id = self.alloc();
        self.sync_done = false;
        var a = ArgBuf{};
        a.u32v(self.sync_id);
        try self.send(display_id, 0, a.slice()); // wl_display.sync

        while (!self.sync_done) {
            if (try self.takeMessage()) |_| {} else try self.fill();
        }
    }

    fn bind(self: *Client, name: u32, iface: []const u8, version: u32, new_id: u32) !void {
        var a = ArgBuf{};
        a.u32v(name);
        a.str(iface);
        a.u32v(version);
        a.u32v(new_id);
        try self.send(self.registry_id, 0, a.slice()); // wl_registry.bind
    }

    fn fill(self: *Client) !void {
        if (self.len == self.buf.len) return error.BufferOverflow;
        const n = try check(linux.read(self.fd, self.buf[self.len..].ptr, self.buf.len - self.len));
        if (n == 0) return error.Disconnected;
        self.len += n;
    }

    /// Returns:
    ///   null            -> no complete message buffered, caller should fill()
    ///   ?Event == null  -> a message was consumed but produced no idle event
    ///   ?Event == ev    -> an idle/resume event
    fn takeMessage(self: *Client) !?(?Event) {
        if (self.len < 8) return null;
        const obj = rd(self.buf[0..4]);
        const word = rd(self.buf[4..8]);
        const size: usize = word >> 16;
        const op: u16 = @truncate(word & 0xffff);
        if (size < 8) return error.BadMessage;
        if (self.len < size) return null;

        const body = self.buf[8..size];
        const ev = try self.dispatch(obj, op, body);

        // slide the rest of the buffer down
        std.mem.copyForwards(u8, self.buf[0 .. self.len - size], self.buf[size..self.len]);
        self.len -= size;
        return ev;
    }

    fn dispatch(self: *Client, obj: u32, op: u16, body: []const u8) !?Event {
        if (obj == display_id) {
            if (op == 0) { // wl_display.error(object, code, message)
                const code = rd(body[4..8]);
                const slen = rd(body[8..12]);
                const msg = body[12 .. 12 + slen - 1];
                std.debug.print("pipescreen: wl_display error code={d}: {s}\n", .{ code, msg });
                return error.Protocol;
            }
            return null; // delete_id, ignore
        }
        if (obj == self.registry_id and op == 0) { // wl_registry.global
            const name = rd(body[0..4]);
            const slen = rd(body[4..8]);
            const iface = body[8 .. 8 + slen - 1];
            const ver = rd(body[8 + align4(slen) ..][0..4]);
            if (std.mem.eql(u8, iface, "wl_seat")) {
                self.seat_name = name;
                self.seat_ver = @min(ver, 5);
            } else if (std.mem.eql(u8, iface, "ext_idle_notifier_v1")) {
                self.notifier_name = name;
                self.notifier_ver = @min(ver, 1);
            }
            return null;
        }
        if (obj == self.sync_id and op == 0) { // wl_callback.done
            self.sync_done = true;
            return null;
        }
        // Only treat op 0/1 as idle/resume for objects we know are notifications,
        // otherwise wl_seat events (capabilities=0, name=1) would be misread.
        for (self.notifs[0..self.notif_count]) |n| {
            if (n == obj) {
                return switch (op) {
                    0 => Event{ .notif = obj, .kind = .idled },
                    1 => Event{ .notif = obj, .kind = .resumed },
                    else => null,
                };
            }
        }
        return null;
    }

    fn send(self: *Client, obj: u32, op: u16, args: []const u8) !void {
        var out: [512]u8 = undefined;
        const total = 8 + args.len;
        const size: u32 = @intCast(total);
        wr(out[0..4], obj);
        wr(out[4..8], (size << 16) | op);
        @memcpy(out[8..total], args);
        var off: usize = 0;
        while (off < total) {
            off += try check(linux.write(self.fd, out[off..total].ptr, total - off));
        }
    }
};

const ArgBuf = struct {
    data: [256]u8 = undefined,
    len: usize = 0,

    fn u32v(self: *ArgBuf, v: u32) void {
        wr(self.data[self.len..][0..4], v);
        self.len += 4;
    }
    fn str(self: *ArgBuf, s: []const u8) void {
        self.u32v(@intCast(s.len + 1)); // length includes the NUL
        @memcpy(self.data[self.len..][0..s.len], s);
        self.len += s.len;
        self.data[self.len] = 0;
        self.len += 1;
        while (self.len % 4 != 0) {
            self.data[self.len] = 0;
            self.len += 1;
        }
    }
    fn slice(self: *const ArgBuf) []const u8 {
        return self.data[0..self.len];
    }
};

fn check(rc: usize) !usize {
    const signed: isize = @bitCast(rc);
    if (signed < 0) return error.Syscall;
    return rc;
}
fn rd(b: *const [4]u8) u32 {
    return std.mem.readInt(u32, b, native);
}
fn wr(b: *[4]u8, v: u32) void {
    std.mem.writeInt(u32, b, v, native);
}
fn align4(n: u32) u32 {
    return (n + 3) & ~@as(u32, 3);
}
