import os

with open('A_series/A_44.rs', 'r', encoding='utf-8') as f:
    a44_code = f.read()

with open('B_series/B_01.rs', 'r', encoding='utf-8') as f:
    b01_code = f.read()

# Modify A_44 main to pub fn solve(input: &str)
# In A_44:
# fn main() {
#     let start_time = Instant::now();
#     ...
#     let mut input = String::new();
#     io::stdin().read_to_string(&mut input).unwrap();
#     let mut lines = input.lines();
a44_code_mod = a44_code.replace(
    "fn main() {",
    "pub fn solve(input_str: &str) {"
).replace(
    "let mut input = String::new();\n    io::stdin().read_to_string(&mut input).unwrap();\n    let mut lines = input.lines();",
    "let mut lines = input_str.lines();"
).replace(
    "let mut input = String::new();\r\n    io::stdin().read_to_string(&mut input).unwrap();\r\n    let mut lines = input.lines();",
    "let mut lines = input_str.lines();"
)

# Modify B_01 main to pub fn solve(input: &str)
# In B_01:
# fn main() {
#     let mut input = String::new();
#     io::stdin().read_to_string(&mut input).unwrap();
#     let mut lines = input.lines();
b01_code_mod = b01_code.replace(
    "fn main() {",
    "pub fn solve(input_str: &str) {"
).replace(
    "let mut input = String::new();\n    io::stdin().read_to_string(&mut input).unwrap();\n    let mut lines = input.lines().filter(|l| !l.trim().is_empty());",
    "let mut lines = input_str.lines().filter(|l| !l.trim().is_empty());"
).replace(
    "let mut input = String::new();\r\n    io::stdin().read_to_string(&mut input).unwrap();\r\n    let mut lines = input.lines().filter(|l| !l.trim().is_empty());",
    "let mut lines = input_str.lines().filter(|l| !l.trim().is_empty());"
)

hybrid_source = f"""// -------------------------------------------------------------
// 方針 A: 適応型ハイブリッドソルバー (Adaptive Hybrid Solver)
// 中・小盤面 (slimes <= 115): A_44 (タワー事前クラスタリング + マクロ集約 + SA)
// 超密集盤面 (slimes > 115):  B_01 (高速 Chokudai Search + 3手先静穏先読み + Pruning)
// -------------------------------------------------------------

use std::io::{{self, Read}};

mod a44 {{
{a44_code_mod}
}}

mod b01 {{
{b01_code_mod}
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

    if slimes <= 115 {{
        a44::solve(&input);
    }} else {{
        b01::solve(&input);
    }}
}}
"""

with open('hybrid_test.rs', 'w', encoding='utf-8') as f:
    f.write(hybrid_source)

print("Successfully generated hybrid_test.rs!")
