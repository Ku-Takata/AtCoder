import sys
sys.setrecursionlimit(10**7)

def check_multi_K(multi,A,B):
    if A > B:
        return "NG"
    if A % K == 0:
        return "OK"

    return check_multi_K(multi,A+1,B)

K = int(input())
A, B = map(int,input().split())

print(check_multi_K(K,A,B))