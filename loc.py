#!/usr/bin/env python3
import os
import sys
from pathlib import Path
from collections import defaultdict

EXTENSIONS = {
    # Rust, Slint, Markdown, Lua (your original)
    '.rs', '.slint', '.md', '.lua',
    # Python
    '.py', '.pyw',
    # JavaScript/TypeScript
    '.js', '.ts', '.jsx', '.tsx', '.mjs', '.cjs',
    # Java/JVM
    '.java', '.kt', '.scala', '.clj', '.groovy',
    # C/C++
    '.c', '.cpp', '.cc', '.cxx', '.h', '.hpp', '.hh', '.h++',
    # C#/.NET
    '.cs', '.fs', '.fsx', '.fsi', '.vb',
    # Go
    '.go',
    # Ruby
    '.rb', '.rbw',
    # PHP
    '.php', '.phtml', '.php3', '.php4', '.php5',
    # Swift
    '.swift',
    # R
    '.r', '.R',
    # Julia
    '.jl',
    # Perl
    '.pl', '.pm',
    # Shell
    '.sh', '.bash', '.zsh', '.fish',
    # SQL
    '.sql',
    # Web
    '.html', '.htm', '.css', '.scss', '.sass', '.less', '.xml', '.json', '.yaml', '.yml',
    # Lisp variants
    '.el', '.scm', '.rkt',
    # Other
    '.swift', '.go', '.nim', '.zig', '.dart', '.ex', '.exs', '.hs', '.ml', '.asm', '.s',
    '.m', '.mm', '.v', '.vhd', '.pas', '.pp', '.dpr', '.coffee', '.astro', '.vue'
}

def count_lines(file_path):
    """Count lines in a file, excluding empty lines and comments."""
    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
        
        total = len(lines)
        non_empty = sum(1 for line in lines if line.strip())
        
        return {'total': total, 'non_empty': non_empty}
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=sys.stderr)
        return {'total': 0, 'non_empty': 0}

def scan_directory(directory='.'):
    """Scan directory for target files and count lines."""
    results = defaultdict(lambda: {'total': 0, 'non_empty': 0, 'files': 0})
    
    for file_path in Path(directory).rglob('*'):
        if file_path.suffix in EXTENSIONS:
            counts = count_lines(file_path)
            ext = file_path.suffix
            
            results[ext]['total'] += counts['total']
            results[ext]['non_empty'] += counts['non_empty']
            results[ext]['files'] += 1
    
    return results

def print_results(results):
    """Print formatted results."""
    if not results:
        print("No matching files found.")
        return
    
    print(f"{'Extension':<15} {'Files':<8} {'Total':<12} {'Non-Empty':<12}")
    print("-" * 50)
    
    grand_total = 0
    grand_non_empty = 0
    
    for ext in sorted(results.keys()):
        data = results[ext]
        print(f"{ext:<15} {data['files']:<8} {data['total']:<12} {data['non_empty']:<12}")
        grand_total += data['total']
        grand_non_empty += data['non_empty']
    
    print("-" * 50)
    print(f"{'TOTAL':<15} {sum(r['files'] for r in results.values()):<8} {grand_total:<12} {grand_non_empty:<12}")

if __name__ == '__main__':
    target_dir = sys.argv[1] if len(sys.argv) > 1 else '.'
    results = scan_directory(target_dir)
    print_results(results)
