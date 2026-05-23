N = int(input())
S = list(map(str,input().split()))

ans = []

for s in S:
    char = s[0]
    if char in ["a","b","c"]:
        ans.append("2")
    elif char in ["d","e","f"]:
        ans.append("3")
    elif char in ["g","h","i"]:
        ans.append("4")
    elif char in ["j","k","l"]:
        ans.append("5")
    elif char in ["m","n","o"]:
        ans.append("6")
    elif char in ["p","q","r","s"]:
        ans.append("7")
    elif char in ["t","u","v"]:
        ans.append("8")
    elif char in ["w","x","y","z"]:
        ans.append("9")

print("".join(ans))