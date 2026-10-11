import json
import os

brain_dir = r'C:\Users\kuon9\.gemini\antigravity\brain'
for root, dirs, files in os.walk(brain_dir):
    for f in files:
        if f == 'transcript_full.jsonl':
            cp = os.path.join(root, f)
            try:
                with open(cp, 'r', encoding='utf-8', errors='ignore') as fp:
                    for idx, line in enumerate(fp):
                        if 'A_44' in line or 'step44' in line:
                            obj = json.loads(line)
                            # check tool_calls
                            tcs = obj.get('tool_calls', [])
                            for tc in tcs:
                                fn = tc.get('function', {})
                                args = fn.get('arguments', {})
                                target = args.get('TargetFile', '')
                                cmd = args.get('CommandLine', '')
                                if 'A_44' in target or 'A_44' in cmd or 'step44' in cmd:
                                    print(f"[{cp}] line {idx}: func={fn.get('name')}, target={target}, cmd={cmd[:80]}")
            except Exception as e:
                pass
