//! DBus watcher: spawn dbus-monitor to listen for PrepareForSleep signal from logind.
//! Emits a suspend-lock when the system is about to sleep (PrepareForSleep(true)).

const std = @import("std");
const linux = std.os.linux;

pub const Watcher = struct {
    fd: i32, // stdout from dbus-monitor
    buf: [4096]u8 = undefined,
    pos: usize = 0,

    pub fn start(io: std.Io) !Watcher {
        _ = io; // unused; available for future cleanup
        // Create a pipe to capture dbus-monitor's output
        var pipefd: [2]i32 = undefined;
        const flags = std.mem.zeroes(linux.O);
        if (linux.pipe2(&pipefd, flags) != 0) return error.PipeFailed;

        // Fork and exec dbus-monitor in the child
        const pid = linux.fork();
        if (@as(isize, @bitCast(pid)) < 0) {
            _ = linux.close(pipefd[0]);
            _ = linux.close(pipefd[1]);
            return error.ForkFailed;
        }

        if (pid == 0) {
            // Child: redirect stdout to pipe, exec dbus-monitor
            _ = linux.dup2(pipefd[1], 1);
            _ = linux.close(pipefd[0]);
            _ = linux.close(pipefd[1]);

            const argv = [_:null]?[*:0]const u8{
                "/usr/bin/dbus-monitor",
                "--system",
                "type='signal',interface='org.freedesktop.login1.Manager',member='PrepareForSleep'",
                null,
            };
            const envp = [_:null]?[*:0]const u8{null};
            _ = linux.execve("/usr/bin/dbus-monitor", argv[0..].ptr, envp[0..].ptr);
            // execve does not return on success
            std.debug.print("pipescreen: dbus-monitor exec failed\n", .{});
            linux.exit(127);
        }

        // Parent: close write end, return read end
        _ = linux.close(pipefd[1]);
        const fl = linux.fcntl(pipefd[0], linux.F.GETFL, 0);
        _ = linux.fcntl(pipefd[0], linux.F.SETFL, fl | 0o4000); // O_NONBLOCK

        return Watcher{ .fd = pipefd[0] };
    }

    pub fn drain(self: *Watcher, ctx: *anyopaque, cb: *const fn (*anyopaque, bool) void) !void {
        // Read available data from dbus-monitor output
        while (true) {
            const n = linux.read(self.fd, self.buf[self.pos..].ptr, self.buf.len - self.pos);
            if (@as(isize, @bitCast(n)) < 0) {
                const err = @as(isize, @bitCast(n));
                if (err == -11 or err == -35) break; // EAGAIN/EWOULDBLOCK
                return error.ReadFailed;
            }
            if (@as(isize, @bitCast(n)) == 0) break; // EOF or no data

            self.pos += @intCast(@as(isize, @bitCast(n)));

            // Parse lines looking for "member=PrepareForSleep" and "boolean 1"
            var line_start: usize = 0;
            while (line_start < self.pos) {
                if (std.mem.indexOf(u8, self.buf[line_start..self.pos], "\n")) |newline_off| {
                    const line_end = line_start + newline_off;
                    const line = self.buf[line_start..line_end];

                    // Detect the signal pattern across multiple lines
                    if (std.mem.indexOf(u8, line, "member=PrepareForSleep") != null) {
                        // Next line should contain "boolean true" or "boolean false"
                        const next_line_start = line_end + 1;
                        if (next_line_start < self.pos) {
                            const next_line_end = std.mem.indexOf(u8, self.buf[next_line_start..self.pos], "\n") orelse self.pos;
                            const next_line = self.buf[next_line_start..next_line_start + next_line_end];
                            if (std.mem.indexOf(u8, next_line, "boolean true")) |_| {
                                cb(ctx, true); // PrepareForSleep(true) = going to sleep
                            }
                        }
                    }

                    line_start = line_end + 1;
                } else {
                    break; // Incomplete line
                }
            }

            // Shift buffer
            if (line_start > 0) {
                const remaining = self.pos - line_start;
                if (remaining > 0) {
                    @memcpy(self.buf[0..remaining], self.buf[line_start..self.pos]);
                }
                self.pos = remaining;
            }
        }
    }
};
