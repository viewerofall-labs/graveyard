//! Tiny line-based control socket. The daemon listens; `pipescreen <cmd>`
//! connects, writes one command line, reads one response line. Raw linux
//! syscalls, same as idle.zig, to stay off std.Io.

const std = @import("std");
const linux = std.os.linux;

fn check(rc: usize) !usize {
    const s: isize = @bitCast(rc);
    if (s < 0) return error.Syscall;
    return rc;
}

fn fillAddr(addr: *linux.sockaddr.un, path: []const u8) !void {
    addr.family = linux.AF.UNIX;
    @memset(&addr.path, 0);
    if (path.len >= addr.path.len) return error.PathTooLong;
    @memcpy(addr.path[0..path.len], path);
}

/// Create + bind + listen on the control socket. Non-blocking.
pub fn listen(path: []const u8) !i32 {
    var zbuf: [linux.PATH_MAX]u8 = undefined;
    if (path.len + 1 > zbuf.len) return error.PathTooLong;
    @memcpy(zbuf[0..path.len], path);
    zbuf[path.len] = 0;
    _ = linux.unlink(@ptrCast(&zbuf)); // ignore ENOENT

    const fd: i32 = @intCast(try check(linux.socket(linux.AF.UNIX, linux.SOCK.STREAM, 0)));
    errdefer _ = linux.close(fd);
    var addr: linux.sockaddr.un = undefined;
    try fillAddr(&addr, path);
    _ = try check(linux.bind(fd, @ptrCast(&addr), @sizeOf(linux.sockaddr.un)));
    _ = try check(linux.listen(fd, 8));
    const fl = linux.fcntl(fd, linux.F.GETFL, 0);
    _ = linux.fcntl(fd, linux.F.SETFL, fl | 0o4000); // O_NONBLOCK
    return fd;
}

/// Connect to the daemon, send `cmd`, return the response (into `resp`).
pub fn send(path: []const u8, cmd: []const u8, resp: []u8) ![]u8 {
    const fd: i32 = @intCast(try check(linux.socket(linux.AF.UNIX, linux.SOCK.STREAM, 0)));
    defer _ = linux.close(fd);
    var addr: linux.sockaddr.un = undefined;
    try fillAddr(&addr, path);
    _ = try check(linux.connect(fd, @ptrCast(&addr), @sizeOf(linux.sockaddr.un)));

    var off: usize = 0;
    while (off < cmd.len) {
        off += try check(linux.write(fd, cmd[off..].ptr, cmd.len - off));
    }
    const n = try check(linux.read(fd, resp.ptr, resp.len));
    return resp[0..n];
}

/// Accept one pending connection (non-blocking). Returns the conn fd, or null
/// if nothing is pending.
pub fn accept(lfd: i32) ?i32 {
    const rc = linux.accept(lfd, null, null);
    const s: isize = @bitCast(rc);
    if (s < 0) return null; // EAGAIN / EWOULDBLOCK
    return @intCast(rc);
}

pub fn recvLine(cfd: i32, buf: []u8) []const u8 {
    const rc = linux.read(cfd, buf.ptr, buf.len);
    const s: isize = @bitCast(rc);
    if (s <= 0) return buf[0..0];
    return std.mem.trim(u8, buf[0..@intCast(rc)], " \t\r\n");
}

pub fn reply(cfd: i32, msg: []const u8) void {
    _ = linux.write(cfd, msg.ptr, msg.len);
}

pub fn close(fd: i32) void {
    _ = linux.close(fd);
}
