q = int(input())
S = ["".join(input()) for i in range(q)]


from collections import Counter

for s in S:
    s = ["".join(s[_]) for _ in range(len(s))]
    sub_list = []
    for j in range(len(s)):
        for k in range(j+1, len(s)+1):
            sub = s[j:k]
            sorted_sub = ["".join(sorted(sub))]
            sub_list = sub_list.append(sorted_sub)

print(sorted_sub)