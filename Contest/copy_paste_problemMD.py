import os
import re
import requests
from bs4 import BeautifulSoup

def clean_extra_newlines(text):
    """連続する無駄な改行を整理する"""
    text = re.sub(r'\n{3,}', '\n\n', text)
    return text.strip()

def parse_atcoder_html(html_content):
    """AtCoderの問題文HTMLから日本語文を抽出してMarkdownに変換する"""
    soup = BeautifulSoup(html_content, 'html.parser')
    
    # 日本語の問題文エリアを取得
    statement = soup.find('span', class_='lang-ja')
    if not statement:
        statement = soup.find('span', id='task-statement')
    
    if not statement:
        return "問題文のエリア（#task-statement または .lang-ja）が見つかりませんでした。"

    # ==========================================
    # 【前処理】Markdown変換前に、HTMLツリー上の数式・変数を綺麗にする
    # ==========================================

    # 1. KaTeX (AtCoderの最新の数式) を安全な文字列に置換
    for katex in statement.find_all('span', class_='katex-display'):
        annotation = katex.find('annotation')
        tex = annotation.get_text().strip() if annotation else ""
        if katex.parent:
            katex.replace_with(f"\n\n$${tex}$$\n\n")

    for katex in statement.find_all('span', class_='katex'):
        if katex.parent is None:
            continue
        annotation = katex.find('annotation')
        tex = annotation.get_text().strip() if annotation else ""
        katex.replace_with(f"${tex}$")

    # 2. MathJax (旧来の数式) の置換
    for script in statement.find_all('script'):
        if script.parent is None:
            continue
        script_type = script.get('type', '') or ''
        if 'math/tex' in script_type:
            tex = script.get_text().strip()
            if 'mode=display' in script_type:
                script.replace_with(f"\n\n$${tex}$$\n\n")
            else:
                script.replace_with(f"${tex}$")

    # 3. 重複出力の原因となる表示用スクリプトや残骸をDOMから完全削除
    for el in statement.find_all(class_=lambda c: c and any(sub in c for sub in ['MathJax_Preview', 'MathJax', 'katex-html', 'katex-mathml', 'div-btn-copy', 'btn-copy'])):
        if el.parent:
            el.decompose()
    for el in statement.find_all(['script', 'style', 'canvas']):
        if el.parent:
            el.decompose()

    # 4. AtCoder特有の <var> タグの処理
    for var in statement.find_all('var'):
        if var.parent is None:
            continue
        text = var.get_text().strip()
        # すでに前段の処理で $ が含まれている場合はそのまま、無い場合は $ で囲む
        if not text.startswith('$') and not text.endswith('$'):
            var.replace_with(f"${text}$")
        else:
            var.replace_with(text)

    # ==========================================
    # 【変換】綺麗になったDOMツリーをMarkdownにする
    # ==========================================
    def convert_element(element):
        md = ""
        for child in element.children:
            if child.name is None:
                text = child.string if child.string else ""
                text = text.replace('\r\n', '').replace('\n', '')
                md += text
            elif child.name in ['h3', 'h4', 'h5']:
                level = int(child.name[1])
                md += f"\n\n{'#' * level} {convert_element(child).strip()}\n\n"
            elif child.name == 'p':
                md += f"\n\n{convert_element(child).strip()}\n\n"
            elif child.name == 'strong':
                md += f"**{convert_element(child).strip()}**"
            elif child.name == 'em':
                md += f"*{convert_element(child).strip()}*"
            elif child.name == 'code':
                # 【修正点】数式($)を含む場合はコード(灰色の背景)化せず、そのまま出力する
                text = child.get_text()
                if '$' in text:
                    md += text
                else:
                    md += f"`{text}`"
            elif child.name == 'pre':
                code_text = child.get_text()
                code_text = code_text.replace('\r\n', '\n')
                code_text = re.sub(r'\n{3,}', '\n\n', code_text)
                
                # 【修正点】数式($)が含まれる場合はコードブロック(```)を避け、引用(>)で表現する
                if '$' in code_text:
                    lines = code_text.strip().split('\n')
                    # 各行の末尾に半角スペース2つを入れてソフト改行を維持
                    quoted = '\n'.join([f"> {line}  " for line in lines])
                    md += f"\n\n{quoted}\n\n"
                else:
                    md += f"\n\n```\n{code_text.strip()}\n```\n\n"
            elif child.name in ['ul', 'ol']:
                md += "\n\n"
                for i, li in enumerate(child.find_all('li', recursive=False), 1):
                    prefix = f"{i}. " if child.name == 'ol' else "* "
                    md += f"{prefix}{convert_element(li).strip()}\n"
                md += "\n\n"
            elif child.name == 'a':
                url = child.get('href', '')
                text = convert_element(child).strip()
                md += f"[{text}]({url})"
            elif child.name == 'img':
                src = child.get('src', '')
                alt = child.get('alt', 'image')
                md += f"![{alt}]({src})"
            elif child.name == 'br':
                md += "  \n"
            elif child.name == 'hr':
                md += "\n\n---\n\n"
            else:
                md += convert_element(child)
        return md

    markdown_text = convert_element(statement)
    
    markdown_text = markdown_text.replace(r'\(', '$').replace(r'\)', '$')
    markdown_text = markdown_text.replace(r'\[', '$$').replace(r'\]', '$$')
    
    return clean_extra_newlines(markdown_text)

def main():
    print("--- AtCoder 問題文 Markdown生成ツール (数式ブロック最適化版) ---")
    print("1: URLから直接取得する")
    print("2: 保存したHTMLファイルから変換する")
    
    mode = input("モードを選択してください (1/2): ").strip()
    
    html_content = ""
    
    if mode == '1':
        url = input("\n問題のURLを入力してください:\n> ").strip()
        print("\nリアルタイム参加中のコンテストなど、ログイン制限がかかっている問題ですか？")
        print("必要な場合はブラウザのCookieから 'REVEL_SESSION' の値を入力してください（空欄でEnter）")
        repsess = input("REVEL_SESSION: ").strip()
        
        headers = {
            'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'
        }
        cookies = {}
        if repsess:
            # クッキーの送信キー名を 'REVEL_SESSION' に修正
            cookies['REVEL_SESSION'] = repsess
            
        try:
            response = requests.get(url, headers=headers, cookies=cookies)
            response.raise_for_status()
            response.encoding = 'utf-8'
            html_content = response.text
        except Exception as e:
            print(f"URLからの取得に失敗しました: {e}")
            return
            
    elif mode == '2':
        file_name = input("\nHTMLファイル名を入力してください (デフォルト: source.html): ").strip()
        if not file_name:
            file_name = "source.html"
            
        if not os.path.exists(file_name):
            print(f"ファイルが見つかりません: {file_name}")
            return
            
        with open(file_name, "r", encoding="utf-8") as f:
            html_content = f.read()
    else:
        print("無効な選択です。")
        return

    print("\nHTMLを解析してMarkdownへ変換中...")
    markdown_result = parse_atcoder_html(html_content)
    
    output_file = "Problem.md"
    with open(output_file, "w", encoding="utf-8") as f:
        f.write(markdown_result)
        
    print(f"\n成功しました！ '{output_file}' に綺麗な問題文を保存しました。")

if __name__ == "__main__":
    main()