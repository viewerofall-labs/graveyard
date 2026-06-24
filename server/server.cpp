#include "httplib.h"
#include <iostream>
#include <string>
#include <ifaddrs.h>
#include <arpa/inet.h>
#include <netinet/in.h>

static const char* HTML = R"html(<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
  <title>you got here</title>
  <style>
    @import url('https://fonts.googleapis.com/css2?family=Share+Tech+Mono&family=Bebas+Neue&display=swap');

    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

    :root {
      --bg:     #0a0010;
      --purple: #7b2fff;
      --pink:   #ff2fd6;
      --cyan:   #00ffe0;
      --white:  #f0eaff;
      --glow-p: 0 0 12px #7b2fff, 0 0 40px #7b2fff88;
      --glow-c: 0 0 12px #00ffe0, 0 0 40px #00ffe088;
    }

    html, body {
      height: 100%;
      background: var(--bg);
      color: var(--white);
      font-family: 'Share Tech Mono', monospace;
      overflow: hidden;
    }

    /* scanline overlay */
    body::after {
      content: '';
      position: fixed; inset: 0;
      background: repeating-linear-gradient(
        0deg,
        transparent,
        transparent 2px,
        rgba(0,0,0,0.18) 2px,
        rgba(0,0,0,0.18) 4px
      );
      pointer-events: none;
      z-index: 100;
    }

    canvas#bg {
      position: fixed; inset: 0;
      width: 100%; height: 100%;
      z-index: 0;
    }

    .stage {
      position: relative;
      z-index: 10;
      height: 100vh;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 1.5rem;
    }

    .tag {
      font-size: 0.75rem;
      letter-spacing: 0.35em;
      text-transform: uppercase;
      color: var(--cyan);
      text-shadow: var(--glow-c);
      opacity: 0;
      animation: fadeup 0.6s 0.2s forwards;
    }

    h1 {
      font-family: 'Bebas Neue', sans-serif;
      font-size: clamp(4rem, 14vw, 11rem);
      line-height: 0.9;
      text-align: center;
      letter-spacing: 0.02em;
      background: linear-gradient(135deg, var(--purple) 0%, var(--pink) 50%, var(--cyan) 100%);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
      filter: drop-shadow(0 0 18px #7b2fff99);
      opacity: 0;
      animation: fadeup 0.7s 0.5s forwards, flicker 6s 1.5s infinite;
    }

    .sub {
      font-size: clamp(0.8rem, 2vw, 1.1rem);
      color: var(--white);
      opacity: 0;
      animation: fadeup 0.6s 0.9s forwards;
      letter-spacing: 0.15em;
    }

    .sub span { color: var(--pink); text-shadow: 0 0 8px var(--pink); }

    .divider {
      width: clamp(120px, 30vw, 300px);
      height: 1px;
      background: linear-gradient(90deg, transparent, var(--purple), var(--cyan), transparent);
      opacity: 0;
      animation: fadeup 0.5s 1.1s forwards;
    }

    .info {
      font-size: 0.72rem;
      letter-spacing: 0.2em;
      color: #7b5fa0;
      opacity: 0;
      animation: fadeup 0.5s 1.3s forwards;
    }

    .cursor {
      display: inline-block;
      width: 10px; height: 1.1em;
      background: var(--cyan);
      margin-left: 4px;
      vertical-align: middle;
      animation: blink 1s step-end infinite;
      box-shadow: var(--glow-c);
    }

    @keyframes fadeup {
      from { opacity: 0; transform: translateY(18px); }
      to   { opacity: 1; transform: translateY(0); }
    }
    @keyframes flicker {
      0%,95%,100% { filter: drop-shadow(0 0 18px #7b2fff99); }
      96%          { filter: drop-shadow(0 0 2px #7b2fff33); }
      97%          { filter: drop-shadow(0 0 22px #ff2fd6bb); }
      98%          { filter: drop-shadow(0 0 2px #7b2fff22); }
    }
    @keyframes blink {
      50% { opacity: 0; }
    }
  </style>
</head>
<body>
  <canvas id="bg"></canvas>
  <div class="stage">
    <div class="tag">// signal received</div>
    <h1>HELLO<br>YOU GOT<br>HERE</h1>
    <div class="divider"></div>
    <p class="sub">broadcast confirmed &mdash; <span>network is live</span></p>
    <p class="info">TWM &bull; LAN TEST &bull; C++/HTTPLIB<span class="cursor"></span></p>
  </div>

  <script>
    const canvas = document.getElementById('bg');
    const ctx    = canvas.getContext('2d');
    let W, H, particles = [];

    function resize() {
      W = canvas.width  = window.innerWidth;
      H = canvas.height = window.innerHeight;
    }
    window.addEventListener('resize', resize);
    resize();

    // spawn grid particles
    for (let i = 0; i < 80; i++) {
      particles.push({
        x: Math.random() * 1920,
        y: Math.random() * 1080,
        vx: (Math.random() - 0.5) * 0.4,
        vy: (Math.random() - 0.5) * 0.4,
        r: Math.random() * 1.5 + 0.5,
        color: Math.random() > 0.5 ? '#7b2fff' : '#00ffe0',
        alpha: Math.random() * 0.6 + 0.2,
      });
    }

    function draw() {
      ctx.clearRect(0, 0, W, H);

      // faint grid
      ctx.strokeStyle = 'rgba(123,47,255,0.06)';
      ctx.lineWidth = 1;
      const gs = 60;
      for (let x = 0; x < W; x += gs) { ctx.beginPath(); ctx.moveTo(x,0); ctx.lineTo(x,H); ctx.stroke(); }
      for (let y = 0; y < H; y += gs) { ctx.beginPath(); ctx.moveTo(0,y); ctx.lineTo(W,y); ctx.stroke(); }

      // particles + connections
      for (let i = 0; i < particles.length; i++) {
        const p = particles[i];
        p.x += p.vx; p.y += p.vy;
        if (p.x < 0) p.x = W; if (p.x > W) p.x = 0;
        if (p.y < 0) p.y = H; if (p.y > H) p.y = 0;

        ctx.beginPath();
        ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
        ctx.fillStyle = p.color;
        ctx.globalAlpha = p.alpha;
        ctx.fill();
        ctx.globalAlpha = 1;

        for (let j = i + 1; j < particles.length; j++) {
          const q = particles[j];
          const dx = p.x - q.x, dy = p.y - q.y;
          const dist = Math.sqrt(dx*dx + dy*dy);
          if (dist < 120) {
            ctx.beginPath();
            ctx.moveTo(p.x, p.y); ctx.lineTo(q.x, q.y);
            ctx.strokeStyle = p.color;
            ctx.globalAlpha = (1 - dist / 120) * 0.15;
            ctx.lineWidth = 0.5;
            ctx.stroke();
            ctx.globalAlpha = 1;
          }
        }
      }
      requestAnimationFrame(draw);
    }
    draw();
  </script>
</body>
</html>)html";

void print_lan_ips() {
    struct ifaddrs *ifap, *ifa;
    char buf[INET6_ADDRSTRLEN];

    if (getifaddrs(&ifap) != 0) {
        std::cerr << "  (could not enumerate interfaces)\n";
        return;
    }

    for (ifa = ifap; ifa; ifa = ifa->ifa_next) {
        if (!ifa->ifa_addr) continue;
        int family = ifa->ifa_addr->sa_family;

        if (family == AF_INET) {
            auto *sa = reinterpret_cast<struct sockaddr_in*>(ifa->ifa_addr);
            inet_ntop(AF_INET, &sa->sin_addr, buf, sizeof(buf));
            std::string ip(buf);
            if (ip != "127.0.0.1")
                std::cout << "  http://" << ip << ":8080  [" << ifa->ifa_name << "]\n";
        } else if (family == AF_INET6) {
            auto *sa6 = reinterpret_cast<struct sockaddr_in6*>(ifa->ifa_addr);
            inet_ntop(AF_INET6, &sa6->sin6_addr, buf, sizeof(buf));
            std::string ip(buf);
            if (ip != "::1" && ip.rfind("fe80", 0) != 0)
                std::cout << "  http://[" << ip << "]:8080  [" << ifa->ifa_name << "]\n";
        }
    }
    freeifaddrs(ifap);
}

int main() {
    httplib::Server svr;

    svr.Get("/", [](const httplib::Request&, httplib::Response& res) {
        res.set_content(HTML, "text/html");
    });

    std::cout << "\n\033[35m[TWM-TEST]\033[0m server starting on \033[36m0.0.0.0:8080\033[0m\n";
    std::cout << "\033[35m[TWM-TEST]\033[0m reachable at:\n";
    print_lan_ips();
    std::cout << "\033[35m[TWM-TEST]\033[0m ctrl+c to stop\n\n";

    svr.listen("0.0.0.0", 8080);
    return 0;
}
