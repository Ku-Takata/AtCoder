T = int(input())

# まずxの数は変えることができないからxの数は合っているかどうかチェック
# サンプル見た限りだと全部xの数は合ってそうだから、ここはさほど気にしなくてもよさそう
# 次にxxとつながっている場合にのみ変更ができる点に注意
# ()の数は2つずつ増加して、一緒になる数ならOK
# Tは最大3*10**5、文字列の総和は2*10**6以下
# 制約でかくないか？？
# AとBの両方、一旦できるところは全てxxに直してA==Bになるか判定してみる

def simple(s):
    stack = []
    for char in s:
        stack.append(char)
        # 末尾4つを見て、置換できるならxxに置換
        if len(stack) >= 4 and stack[-4:] == ["(", "x", "x", ")"]:
            for i in range(4):
                stack.pop()
            for i in range(2):
                stack.append("x")
    return "".join(stack)

for i in range(T):
    A = input()
    B = input()

    if simple(A) == simple(B):
        print("Yes")
    else:
        print("No")