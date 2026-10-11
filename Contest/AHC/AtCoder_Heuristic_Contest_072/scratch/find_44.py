import json
import os

conversations = [
    r'C:\Users\kuon9\.gemini\antigravity\brain\0abee0bd-af55-42e3-b090-b682c4bae870\.system_generated\logs\transcript_full.jsonl',
    r'C:\Users\kuon9\.gemini\antigravity\brain\57a384d2-1479-481b-81e7-aa61bb8be318\.system_generated\logs\transcript_full.jsonl',
    r'C:\Users\kuon9\.gemini\antigravity\brain\7251b57a-f98e-434f-9d29-fe7831a8975b\.system_generated\logs\transcript_full.jsonl',
]

found = False
for cp in conversations:
    if not os.path.exists(cp):
        continue
    with open(cp, 'r', encoding='utf-8') as f:
        for idx, line in enumerate(f):
            obj = json.loads(line)
            tool_calls = obj.get('tool_calls', [])
            for tc in tool_calls:
                fn = tc.get('function', {})
                name = fn.get('name', '')
                args = fn.get('arguments', {})
                args_str = str(args)
                if '44' in args_str and ('write_to_file' in name or 'replace_file' in name):
                    target = args.get('TargetFile', '')
                    content = args.get('CodeContent', '')
                    if 'fn main' in content and ('44' in target or len(content) > 10000):
                        print(f"FOUND in {cp} line {idx}! Target: {target}, len: {len(content)}")
                        with open('A_44_extracted.rs', 'w', encoding='utf-8') as out_f:
                            out_f.write(content)
                        found = True
                        break
            if found:
                break
    if found:
        break

if not found:
    print("Searching for any rust file written in conversation 0abee0bd...")
    with open(conversations[0], 'r', encoding='utf-8') as f:
        for idx, line in enumerate(f):
            obj = json.loads(line)
            tool_calls = obj.get('tool_calls', [])
            for tc in tool_calls:
                fn = tc.get('function', {})
                name = fn.get('name', '')
                args = fn.get('arguments', {})
                if 'write_to_file' in name:
                    target = args.get('TargetFile', '')
                    content = args.get('CodeContent', '')
                    if target.endswith('.rs') and 'fn main' in content:
                        print(f"Found rs file: {target} (len={len(content)})")
                        with open('A_extracted_latest.rs', 'w', encoding='utf-8') as out_f:
                            out_f.write(content)
