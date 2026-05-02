use axum::{Router, response::Html, routing::get};
use std::net::SocketAddr;

const PAGE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>hi</title>
  <style>
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@100;900&display=swap');

    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

    body {
      background: #000;
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
      overflow: hidden;
      font-family: 'Inter', sans-serif;
    }

    /* ── starfield ── */
    .stars {
      position: fixed; inset: 0; z-index: 0;
      background: radial-gradient(ellipse at center, #0a0a1a 0%, #000 100%);
    }
    .star {
      position: absolute;
      border-radius: 50%;
      background: #fff;
      animation: twinkle var(--d) ease-in-out infinite alternate;
      opacity: 0;
    }
    @keyframes twinkle {
      from { opacity: 0; transform: scale(0.6); }
      to   { opacity: var(--max-op); transform: scale(1.2); }
    }

    /* ── main card ── */
    .card {
      position: relative; z-index: 10;
      text-align: center;
      opacity: 0;
      transform: scale(0.6) translateY(60px);
      animation: arrive 1.4s cubic-bezier(0.16, 1, 0.3, 1) 1.2s forwards;
    }
    @keyframes arrive {
      to { opacity: 1; transform: scale(1) translateY(0); }
    }

    /* ── glowing ring ── */
    .ring {
      position: absolute;
      inset: -60px;
      border-radius: 50%;
      border: 2px solid transparent;
      background: conic-gradient(from 0deg, #ff006e, #8338ec, #3a86ff, #06ffd0, #ff006e) border-box;
      -webkit-mask: linear-gradient(#fff 0 0) padding-box, linear-gradient(#fff 0 0);
      -webkit-mask-composite: destination-out;
      mask-composite: exclude;
      animation: spin 8s linear infinite, ring-fade-in 2s ease 0.5s forwards;
      opacity: 0;
    }
    @keyframes spin { to { transform: rotate(360deg); } }
    @keyframes ring-fade-in { to { opacity: 1; } }

    /* ── text ── */
    h1 {
      font-size: clamp(4rem, 15vw, 12rem);
      font-weight: 900;
      letter-spacing: -0.04em;
      line-height: 1;
      background: linear-gradient(135deg, #ff006e 0%, #8338ec 35%, #3a86ff 65%, #06ffd0 100%);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
      animation: shimmer 4s linear infinite;
      background-size: 300% 300%;
    }
    @keyframes shimmer {
      0%   { background-position: 0% 50%; }
      50%  { background-position: 100% 50%; }
      100% { background-position: 0% 50%; }
    }

    .sub {
      margin-top: 1.5rem;
      font-size: clamp(0.85rem, 2vw, 1.1rem);
      font-weight: 100;
      letter-spacing: 0.35em;
      text-transform: uppercase;
      color: #ffffff55;
      opacity: 0;
      animation: fade-up 1s ease 2.4s forwards;
    }
    @keyframes fade-up {
      from { opacity: 0; transform: translateY(16px); }
      to   { opacity: 1; transform: translateY(0); }
    }

    /* ── scanline overlay ── */
    body::after {
      content: '';
      position: fixed; inset: 0; z-index: 20;
      pointer-events: none;
      background: repeating-linear-gradient(
        0deg,
        transparent,
        transparent 2px,
        rgba(0,0,0,0.08) 2px,
        rgba(0,0,0,0.08) 4px
      );
    }

    /* ── ambient glow blobs ── */
    .blob {
      position: fixed; z-index: 1;
      border-radius: 50%;
      filter: blur(80px);
      opacity: 0;
      animation: blob-appear 3s ease var(--delay) forwards;
    }
    .blob-1 { width: 50vw; height: 50vw; background: #8338ec33; top: -10%; left: -10%; --delay: 0.2s; }
    .blob-2 { width: 40vw; height: 40vw; background: #3a86ff33; bottom: -10%; right: -5%;  --delay: 0.6s; }
    .blob-3 { width: 30vw; height: 30vw; background: #ff006e22; top: 30%; right: 10%;     --delay: 1s;   }
    @keyframes blob-appear {
      to { opacity: 1; }
    }
  </style>
</head>
<body>
  <div class="stars" id="stars"></div>
  <div class="blob blob-1"></div>
  <div class="blob blob-2"></div>
  <div class="blob blob-3"></div>

  <div class="card">
    <div class="ring"></div>
    <h1>hello</h1>
    <p class="sub">it&#39;s working &nbsp;·&nbsp; you&#39;re here &nbsp;·&nbsp; wow</p>
  </div>

  <script>
    // generate stars
    const container = document.getElementById('stars');
    for (let i = 0; i < 220; i++) {
      const s = document.createElement('div');
      s.className = 'star';
      const size = Math.random() * 2.5 + 0.5;
      s.style.cssText = [
        `width:${size}px`, `height:${size}px`,
        `top:${Math.random()*100}%`, `left:${Math.random()*100}%`,
        `--d:${(Math.random()*4+2).toFixed(1)}s`,
        `--max-op:${(Math.random()*0.7+0.1).toFixed(2)}`,
        `animation-delay:${(Math.random()*5).toFixed(1)}s`
      ].join(';');
      container.appendChild(s);
    }
  </script>
</body>
</html>"#;

async fn index() -> Html<&'static str> {
    Html(PAGE)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on http://0.0.0.0:3000");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
