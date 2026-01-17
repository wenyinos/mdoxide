#!/usr/bin/env python3
"""
Ferrite Log Analyzer - Analyzes debug logs for performance issues
Specifically designed to identify continuous re-rendering loops and CPU spikes
"""

import re
import sys
from collections import Counter, defaultdict
from datetime import datetime

def parse_log_line(line):
    """Parse a log line and extract timestamp, level, module, and message."""
    # Pattern: [2026-01-17T05:59:26Z INFO  ferrite] Message
    pattern = r'\[(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z)\s+(\w+)\s+([^\]]+)\]\s*(.*)'
    match = re.match(pattern, line.strip())
    if match:
        return {
            'timestamp': match.group(1),
            'level': match.group(2),
            'module': match.group(3),
            'message': match.group(4)
        }
    return None

def normalize_message(msg):
    """Normalize a message by replacing variable parts with placeholders."""
    # Replace line numbers
    msg = re.sub(r'line \d+(-\d+)?', 'line N', msg)
    msg = re.sub(r'at line \d+', 'at line N', msg)
    # Replace counts
    msg = re.sub(r'children_count=\d+', 'children_count=N', msg)
    msg = re.sub(r'len\(\)=\d+', 'len()=N', msg)
    msg = re.sub(r'rendering \d+ children', 'rendering N children', msg)
    # Replace text content (single quoted strings)
    msg = re.sub(r"text_content='[^']*'", "text_content='...'", msg)
    # Replace arrays/lists
    msg = re.sub(r'\["[^"]*"(?:,\s*"[^"]*")*\]', '[...]', msg)
    # Replace hex addresses
    msg = re.sub(r'0x[0-9a-fA-F]+', '0x...', msg)
    # Replace file paths
    msg = re.sub(r'/Users/[^\s]+', '/path/...', msg)
    # Replace byte counts
    msg = re.sub(r'\d+ bytes', 'N bytes', msg)
    msg = re.sub(r'\d+ lines', 'N lines', msg)
    msg = re.sub(r'\d+ tabs', 'N tabs', msg)
    msg = re.sub(r'\d+ syntaxes', 'N syntaxes', msg)
    msg = re.sub(r'\d+ themes', 'N themes', msg)
    return msg

def analyze_log(filepath):
    """Analyze a log file and produce statistics."""
    
    print(f"Analyzing: {filepath}\n")
    print("=" * 80)
    
    # Statistics
    total_lines = 0
    parsed_lines = 0
    module_counts = Counter()
    message_patterns = Counter()
    level_counts = Counter()
    timestamps = []
    messages_per_second = defaultdict(int)
    unique_messages = set()
    
    # Read and parse
    with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
        for line in f:
            total_lines += 1
            parsed = parse_log_line(line)
            if parsed:
                parsed_lines += 1
                module_counts[parsed['module']] += 1
                level_counts[parsed['level']] += 1
                
                # Normalize and count message patterns
                normalized = normalize_message(parsed['message'])
                message_patterns[(parsed['module'], normalized)] += 1
                unique_messages.add(line.strip())
                
                # Track timestamps
                timestamps.append(parsed['timestamp'])
                
                # Messages per second
                second_key = parsed['timestamp'][:19]  # YYYY-MM-DDTHH:MM:SS
                messages_per_second[second_key] += 1
    
    # Report
    print(f"\n[SUMMARY]")
    print(f"   Total lines: {total_lines:,}")
    print(f"   Parsed lines: {parsed_lines:,}")
    print(f"   Unique lines: {len(unique_messages):,}")
    print(f"   Duplication ratio: {total_lines / len(unique_messages):.1f}x" if unique_messages else "   Duplication ratio: N/A")
    
    if timestamps:
        print(f"\n[TIME SPAN]")
        print(f"   Start: {timestamps[0]}")
        print(f"   End: {timestamps[-1]}")
        
        # Calculate duration
        start = datetime.fromisoformat(timestamps[0].replace('Z', '+00:00'))
        end = datetime.fromisoformat(timestamps[-1].replace('Z', '+00:00'))
        duration = (end - start).total_seconds()
        print(f"   Duration: {duration:.1f} seconds")
        if duration > 0:
            print(f"   Avg lines/second: {total_lines / duration:,.0f}")
    
    print(f"\n[LOG LEVELS]")
    for level, count in level_counts.most_common():
        pct = count / parsed_lines * 100 if parsed_lines else 0
        print(f"   {level}: {count:,} ({pct:.1f}%)")
    
    print(f"\n[TOP MODULES] (by message count)")
    for module, count in module_counts.most_common(15):
        pct = count / parsed_lines * 100 if parsed_lines else 0
        bar = "#" * int(pct / 2)
        print(f"   {module}: {count:,} ({pct:.1f}%) {bar}")
    
    print(f"\n[TOP REPEATED MESSAGE PATTERNS]")
    for (module, pattern), count in message_patterns.most_common(20):
        if count > 10:  # Only show patterns that repeat significantly
            short_module = module.split('::')[-1] if '::' in module else module
            short_pattern = pattern[:70] + "..." if len(pattern) > 70 else pattern
            print(f"   [{count:,}x] {short_module}: {short_pattern}")
    
    print(f"\n[MESSAGES PER SECOND] (showing busiest seconds)")
    sorted_seconds = sorted(messages_per_second.items(), key=lambda x: -x[1])[:10]
    for second, count in sorted_seconds:
        bar = "#" * min(int(count / 100), 50)
        print(f"   {second}: {count:,} msgs {bar}")
    
    # Identify potential issues
    print(f"\n[POTENTIAL ISSUES DETECTED]")
    issues = []
    
    # Check for excessive LIST_ITEM_DEBUG messages
    list_debug_count = sum(c for (m, p), c in message_patterns.items() if 'LIST_ITEM_DEBUG' in p)
    if list_debug_count > 1000:
        issues.append(f"CRITICAL: {list_debug_count:,} LIST_ITEM_DEBUG messages - indicates continuous re-rendering of list items")
    
    # Check for rapid-fire logging (>1000 msgs/sec)
    rapid_seconds = [(s, c) for s, c in messages_per_second.items() if c > 1000]
    if rapid_seconds:
        issues.append(f"CRITICAL: {len(rapid_seconds)} seconds with >1000 log messages each - indicates tight loop")
    
    # Check for low unique-to-total ratio
    if unique_messages and total_lines / len(unique_messages) > 10:
        issues.append(f"WARNING: High duplication ratio ({total_lines / len(unique_messages):.1f}x) - same operations repeating")
    
    # Check for specific modules dominating
    if module_counts and parsed_lines > 0:
        top_module, top_count = module_counts.most_common(1)[0]
        if top_count / parsed_lines > 0.9:
            issues.append(f"WARNING: Module '{top_module}' generates {top_count/parsed_lines*100:.1f}% of all logs")
    
    if issues:
        for issue in issues:
            print(f"   - {issue}")
    else:
        print("   None detected")
    
    # Root cause analysis
    print(f"\n[ROOT CAUSE ANALYSIS]")
    
    # Check if it's the markdown editor re-rendering
    editor_count = module_counts.get('ferrite::markdown::editor', 0)
    if editor_count > 10000:
        print("""
   The markdown editor is continuously re-rendering list items.
   
   This is likely caused by:
   1. Missing repaint/needs_repaint check - the UI is requesting repaints
      every frame even when content hasn't changed
   2. Infinite layout loop - each render triggers another render
   3. Missing caching - list items being re-parsed/re-rendered unnecessarily
   
   Key evidence:
   - Same list items (line 6, 7, 8, 9, etc.) rendered repeatedly
   - ~2000+ messages per second
   - All from ferrite::markdown::editor module
   
   Suggested fixes:
   1. Add dirty flag to only re-render when content changes
   2. Cache rendered list items
   3. Check for conditions causing continuous ctx.request_repaint()
   4. Review the rendering loop in src/markdown/editor.rs
""")
    
    return {
        'total_lines': total_lines,
        'unique_lines': len(unique_messages),
        'module_counts': dict(module_counts),
        'message_patterns': dict(message_patterns.most_common(50))
    }

def main():
    if len(sys.argv) < 2:
        # Default to the Intel Mac log file
        filepath = "docs/ferrite_macos_intel_log.txt"
    else:
        filepath = sys.argv[1]
    
    try:
        analyze_log(filepath)
    except FileNotFoundError:
        print(f"Error: File not found: {filepath}")
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
