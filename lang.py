import os
import sys
from collections import defaultdict

def scan_codebase_weight(start_dir=".", show_files=False, github_mode=False):
    LANGUAGE_MAP = {
        # Systems & Low-Level
        ".rs": "Rust",
        ".c": "C",
        ".cpp": "C++",
        ".cc": "C++",
        ".cxx": "C++",
        ".h": "C/C++ Header",
        ".hpp": "C/C++ Header",
        ".asm": "Assembly",
        ".s": "Assembly",
        ".S": "Assembly",
        ".zig": "Zig",
        ".go": "Go",
        ".swift": "Swift",
        ".m": "Objective-C",
        ".mm": "Objective-C++",
        ".d": "D",
        ".pas": "Pascal",
        ".pp": "Pascal",
        
        # Scripting & Dynamic
        ".py": "Python",
        ".pyx": "Cython",
        ".lua": "Lua",
        ".js": "JavaScript",
        ".ts": "TypeScript",
        ".jsx": "JSX",
        ".tsx": "TSX",
        ".rb": "Ruby",
        ".php": "PHP",
        ".sh": "Shell",
        ".bash": "Shell",
        ".zsh": "Shell",
        ".fish": "Shell",
        ".ps1": "PowerShell",
        ".jl": "Julia",
        
        # Web & Frontend
        ".html": "HTML",
        ".htm": "HTML",
        ".css": "CSS",
        ".scss": "SCSS",
        ".sass": "SASS",
        ".less": "LESS",
        ".vue": "Vue",
        ".svelte": "Svelte",
        
        # JVM & Compiled
        ".java": "Java",
        ".kt": "Kotlin",
        ".scala": "Scala",
        ".groovy": "Groovy",
        ".gradle": "Gradle",
        
        # .NET
        ".cs": "C#",
        ".vb": "VB.NET",
        ".fsx": "F#",
        ".fs": "F#",
        
        # Build & Package Management
        ".toml": "TOML",
        ".lock": "Lock",
        ".cmake": "CMake",
        ".make": "Makefile",
        ".dockerfile": "Dockerfile",
        ".gemfile": "Gemfile",
        ".podspec": "Podspec",
        
        # Config & Data
        ".json": "JSON",
        ".jsonc": "JSON",
        ".yaml": "YAML",
        ".yml": "YAML",
        ".xml": "XML",
        ".ini": "INI",
        ".cfg": "Config",
        ".conf": "Config",
        ".env": "Env",
        ".properties": "Properties",
        
        # Database
        ".sql": "SQL",
        ".sqlite": "SQLite",
        ".db": "Database",
        ".pgsql": "PostgreSQL",
        
        # Markup & Docs
        ".md": "Markdown",
        ".markdown": "Markdown",
        ".rst": "reStructuredText",
        ".tex": "LaTeX",
        ".asciidoc": "AsciiDoc",
        ".adoc": "AsciiDoc",
        
        # Graphics & Shaders
        ".glsl": "GLSL",
        ".hlsl": "HLSL",
        ".wgsl": "GPU Graphics",
        ".spv": "SPIR-V",
        ".glb": "glTF Binary",
        ".gltf": "glTF",
        ".obj": "OBJ",
        ".mtl": "MTL",
        
        # Build & Linker
        ".ld": "Linker Script",
        ".lds": "Linker Script",
        ".x": "Linker Script",
        
        # Templates & Markup
        ".jinja": "Jinja",
        ".jinja2": "Jinja2",
        ".hbs": "Handlebars",
        ".ejs": "EJS",
        ".pug": "Pug",
        ".handlebars": "Handlebars",
        
        # Esoteric & Custom
        ".yourmom": ".yourmom",
        ".yourdad": ".yourdad",
        ".momjoke": ".momjoke",
        
        # Other
        ".qml": "QML",
        ".r": "R",
        ".R": "R",
        ".pl": "Perl",
        ".clj": "Clojure",
        ".cljs": "ClojureScript",
        ".ex": "Elixir",
        ".exs": "Elixir",
        ".erl": "Erlang",
        ".hrl": "Erlang Header",
        ".vim": "VimScript",
        ".nvim": "Neovim Config",
        ".lsp": "LSP",
        ".proto": "Protocol Buffers",
        ".thrift": "Thrift",
        ".gql": "GraphQL",
        ".graphql": "GraphQL",
    }

    language_sizes = defaultdict(int)
    language_files = defaultdict(list)
    total_size_bytes = 0
    ignored_dirs = {'.git', 'node_modules', 'target', 'venv', '.venv', 'build', 'dist', '__pycache__', '.env'}
    
    # GitHub-ignored extensions (config, docs, lock files, etc.)
    github_ignored = {
        '.lock', '.md', '.markdown', '.rst', '.tex', '.asciidoc', '.adoc',
        '.json', '.jsonc', '.yaml', '.yml', '.xml', '.ini', '.cfg', '.conf', '.env', '.properties',
        '.toml', '.gradle', '.cmake', '.make', '.dockerfile',
        '.gql', '.graphql', '.proto', '.thrift'
    }

    print(f"--- ⚖️  Calculating Codebase Weight: {os.path.abspath(start_dir)} ---")

    for root, dirs, files in os.walk(start_dir):
        dirs[:] = [d for d in dirs if d not in ignored_dirs and not d.startswith('.')]
        
        for file in files:
            ext = os.path.splitext(file)[1].lower()
            filename_lower = file.lower()
            lang = None
            
            # Handle exact filename matches first
            if filename_lower == "makefile":
                lang = "Makefile"
            elif filename_lower == "justfile":
                lang = "Just"
            elif filename_lower == "dockerfile":
                lang = "Dockerfile"
            elif ext in LANGUAGE_MAP:
                lang = LANGUAGE_MAP[ext]
            
            if lang is None:
                continue
            
            # Skip if GitHub mode and extension is ignored
            if github_mode and ext in github_ignored:
                continue
            
            file_path = os.path.join(root, file)
            
            try:
                file_size = os.path.getsize(file_path)
                language_sizes[lang] += file_size
                language_files[lang].append((file_path, file_size))
                total_size_bytes += file_size
            except (OSError, FileNotFoundError):
                continue

    if total_size_bytes == 0:
        print("❌ No recognized source files found or files are empty.")
        return

    sorted_stats = sorted(language_sizes.items(), key=lambda x: x[1], reverse=True)

    print(f"\n{'Language':<20} | {'Percentage':<12} | {'Size (KB)':<10}")
    print("-" * 55)
    
    for lang, size in sorted_stats:
        percentage = (size / total_size_bytes) * 100
        size_kb = size / 1024
        bar_len = int(percentage / 4)
        bar = "█" * bar_len
        
        print(f"{lang:<20} | {percentage:>6.1f}% | {size_kb:>8.1f} KB  {bar}")

    print(f"\n{'='*55}")
    print(f"TOTAL CODEBASE SIZE: {total_size_bytes / 1024:.2f} KB")
    print(f"{'='*55}\n")

    # Show files for top language
    if show_files:
        top_lang = sorted_stats[0][0]
        print(f"📋 Top Language: {top_lang}")
        print(f"{'File Path':<50} | {'Size (KB)':<10}")
        print("-" * 65)
        
        sorted_files = sorted(language_files[top_lang], key=lambda x: x[1], reverse=True)
        for file_path, size in sorted_files:
            print(f"{file_path:<50} | {size/1024:>8.1f} KB")
        print()

if __name__ == "__main__":
    show_files = "-f" in sys.argv or "--files" in sys.argv
    github_mode = "-g" in sys.argv or "--github" in sys.argv
    scan_codebase_weight(show_files=show_files, github_mode=github_mode)
