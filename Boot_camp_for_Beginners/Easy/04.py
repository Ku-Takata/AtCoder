N = int(input())

ans = ":("

for i in range(1, N+1):
    tax_price = int(i*1.08)
    
    if tax_price == N:
        ans = i
        break

print(ans)