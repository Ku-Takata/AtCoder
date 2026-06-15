N = int(input())

# abs(i-j)が小さい値が一番近いマス

num_list = [i for i in range(1,int(N**0.5)+1)]
ans = float("inf")

for i in num_list:
    if (N/i).is_integer():
        ans = min(ans,i-1 + (N//i)-1)

print(ans)