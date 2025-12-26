N, K = map(int, input().split())
S = input().split()

word_list_filter = [w for w in S if len(w) >= K]
print(*word_list_filter)