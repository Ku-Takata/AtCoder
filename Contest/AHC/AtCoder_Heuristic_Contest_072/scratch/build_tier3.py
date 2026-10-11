import os

with open('A_series/A_44.rs', 'r', encoding='utf-8') as f:
    a44_code = f.read()

with open('B_series/B_01.rs', 'r', encoding='utf-8') as f:
    b01_code = f.read()

with open('cluster_beam_test.rs', 'r', encoding='utf-8') as f:
    cb_code = f.read()

# a44 solve
a44_mod = a44_code.replace(
    "fn main() {",
    "pub fn solve(input_str: &str) {"
).replace(
    "let mut input = String::new();\n    io::stdin().read_to_string(&mut input).unwrap();\n    let mut lines = input.lines();",
    "let mut lines = input_str.lines();"
).replace(
    "let mut input = String::new();\r\n    io::stdin().read_to_string(&mut input).unwrap();\r\n    let mut lines = input.lines();",
    "let mut lines = input_str.lines();"
)

# b01 solve
b01_mod = b01_code.replace(
    "fn main() {",
    "pub fn solve(input_str: &str) {"
).replace(
    "let mut input = String::new();\n    io::stdin().read_to_string(&mut input).unwrap();\n    let mut lines = input.lines().filter(|l| !l.trim().is_empty());",
    "let mut lines = input_str.lines().filter(|l| !l.trim().is_empty());"
).replace(
    "let mut input = String::new();\r\n    io::stdin().read_to_string(&mut input).unwrap();\r\n    let mut lines = input.lines().filter(|l| !l.trim().is_empty());",
    "let mut lines = input_str.lines().filter(|l| !l.trim().is_empty());"
)

# cb solve
# In cluster_beam_test.rs, main() was defined at the bottom.
cb_mod = cb_code.replace(
    "fn main() {",
    "pub fn solve(input_str: &str) {"
).replace(
    "let mut input = String::new();\n    io::stdin().read_to_string(&mut input).unwrap();\n    let mut lines = input.lines().filter(|l| !l.trim().is_empty());",
    "let mut lines = input_str.lines().filter(|l| !l.trim().is_empty());"
).replace(
    "let mut input = String::new();\r\n    io::stdin().read_to_string(&mut input).unwrap();\r\n    let mut lines = input.lines().filter(|l| !l.trim().is_empty());",
    "let mut lines = input_str.lines().filter(|l| !l.trim().is_empty());"
)

tier3_source = f"""// -------------------------------------------------------------
// 方針 A + B 統合: 3 段適応型ハイブリッドソルバー (3-Tier Adaptive Solver)
// 1. 小規模盤面 (slimes <= 75):  A_44 (本格的事前クラスタリング + マクロ集約 + 1.85s 焼きなまし SA)
// 2. 中規模盤面 (75 < slimes <= 115): 方針 B ClusterBeam (事前クラスタリング + Chokudai Search)
// 3. 超密集盤面 (slimes > 115):  B_01 (純粋高速 Chokudai Search + 3手先静穏先読み + Pruning)
// -------------------------------------------------------------

use std::io::{{self, Read}};

mod a44 {{
{a44_mod}
}}

mod cb {{
{cb_mod}
}}

mod b01 {{
{b01_mod}
}}

fn main() {{
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut slimes = 0;
    for line in input.lines().skip(1) {{
        for ch in line.chars() {{
            if ch.is_ascii_lowercase() {{
                slimes += 1;
            }}
        }}
    }}

    if slimes <= 75 {{
        a44::solve(&input);
    }} else if slimes <= 115 {{
        cb::solve(&input);
    }} else {{
        b01::solve(&input);
    }}
}}
"""

with open('tier3_test.rs', 'w', encoding='utf-8') as f:
    f.write(tier3_source)

print("Successfully generated tier3_test.rs!")
