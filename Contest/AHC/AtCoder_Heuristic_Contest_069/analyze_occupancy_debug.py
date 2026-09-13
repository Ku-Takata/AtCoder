import subprocess
import re
import numpy as np

def run_diagnostics(exe_path):
    results = []
    tester = "AdcJXWH4_windows/tools_x86_64-pc-windows-gnu/tester.exe"
    
    for seed in range(100):
        seed_str = f"{seed:04d}"
        infile = f"AdcJXWH4_windows/tools_x86_64-pc-windows-gnu/in/{seed_str}.txt"
        
        with open(infile, "r") as f:
            p = subprocess.run([tester, exe_path], stdin=f, capture_output=True, text=True, timeout=15)
        
        stderr = p.stderr
        
        # Extract metrics
        score = 0
        accepted = 0
        rejected = 0
        filter_rej = 0
        noplace_rej = 0
        overall_occ = 0.0
        q1_occ = q2_occ = q3_occ = q4_occ = 0.0
        avg_compact = 0.0
        
        m_score = re.search(r"Score = (\d+)", stderr)
        if m_score: score = int(m_score.group(1))
        
        m_acc = re.search(r"Accepted=(\d+) / Rejected=(\d+) \(FilterRej=(\d+), NoPlaceRej=(\d+)\)", stderr)
        if m_acc:
            accepted = int(m_acc.group(1))
            rejected = int(m_acc.group(2))
            filter_rej = int(m_acc.group(3))
            noplace_rej = int(m_acc.group(4))
            
        m_occ = re.search(r"Occupancy: Overall=([\d\.]+)% \| Q1=([\d\.]+)% \| Q2=([\d\.]+)% \| Q3=([\d\.]+)% \| Q4=([\d\.]+)%", stderr)
        if m_occ:
            overall_occ = float(m_occ.group(1))
            q1_occ = float(m_occ.group(2))
            q2_occ = float(m_occ.group(3))
            q3_occ = float(m_occ.group(4))
            q4_occ = float(m_occ.group(5))
            
        m_comp = re.search(r"Average Compactness: ([\d\.]+)", stderr)
        if m_comp: avg_compact = float(m_comp.group(1))
        
        results.append({
            "seed": seed, "score": score, "accepted": accepted, "rejected": rejected,
            "filter_rej": filter_rej, "noplace_rej": noplace_rej,
            "overall_occ": overall_occ, "q1_occ": q1_occ, "q2_occ": q2_occ, "q3_occ": q3_occ, "q4_occ": q4_occ,
            "avg_compact": avg_compact
        })
    return results

print("Running 100 cases for main.exe...")
res_main = run_diagnostics("./main.exe")

print("Running 100 cases for main3.exe...")
res_main3 = run_diagnostics("./main3.exe")

def summarize(name, res):
    scores = [r["score"] for r in res]
    accs = [r["accepted"] for r in res]
    frejs = [r["filter_rej"] for r in res]
    nrejs = [r["noplace_rej"] for r in res]
    occs = [r["overall_occ"] for r in res]
    q1s = [r["q1_occ"] for r in res]
    q2s = [r["q2_occ"] for r in res]
    q3s = [r["q3_occ"] for r in res]
    q4s = [r["q4_occ"] for r in res]
    comps = [r["avg_compact"] for r in res]
    
    print(f"\n==============================================")
    print(f"   SUMMARY REPORT: {name}")
    print(f"==============================================")
    print(f"Average Score:     {np.mean(scores):,.0f} (Min: {np.min(scores):,}, Max: {np.max(scores):,})")
    print(f"Accepted Groups:   {np.mean(accs):.1f} / 1000")
    print(f"Filter Rejected:   {np.mean(frejs):.1f} (事前密度フィルタによる拒否)")
    print(f"NoPlace Rejected:  {np.mean(nrejs):.1f} (配置領域不成立による拒否)")
    print(f"Overall Occupancy: {np.mean(occs):.2f}%")
    print(f"Quarter Occupancy: Q1={np.mean(q1s):.1f}% | Q2={np.mean(q2s):.1f}% | Q3={np.mean(q3s):.1f}% | Q4={np.mean(q4s):.1f}%")
    print(print_avg_compact := f"Average Compact C: {np.mean(comps):.4f}")

summarize("main.cpp (旧ベスト 66.42M)", res_main)
summarize("main3.cpp (新蛇腹フィット 65.68M)", res_main3)

# Difference breakdown
print("\n==============================================")
print("   DIFFERENCE ANALYSIS (main3.cpp - main.cpp)")
print("==============================================")
diff_scores = [r3["score"] - r1["score"] for r1, r3 in zip(res_main, res_main3)]
diff_occs = [r3["overall_occ"] - r1["overall_occ"] for r1, r3 in zip(res_main, res_main3)]
diff_frej = [r3["filter_rej"] - r1["filter_rej"] for r1, r3 in zip(res_main, res_main3)]
diff_nrej = [r3["noplace_rej"] - r1["noplace_rej"] for r1, r3 in zip(res_main, res_main3)]

print(f"Avg Score Diff:     {np.mean(diff_scores):+,.0f}")
print(f"Avg Occupancy Diff: {np.mean(diff_occs):+.2f}%")
print(f"Avg FilterRej Diff: {np.mean(diff_frej):+.1f}")
print(f"Avg NoPlaceRej Diff:{np.mean(diff_nrej):+.1f}")
