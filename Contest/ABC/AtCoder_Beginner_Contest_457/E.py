N, M = map(int,input().split())
LR = [list(map(int,input().split())) for i in range(M)]
Q = int(input())
ST = [list(map(int,input().split())) for i in range(Q)]

# これは初めに、2枚の風呂敷を選んだ時に、重なる風呂敷だけを選んで、その組み合わせのリストを作って、どこからどこまでのマスを覆えるかをリスト化して、
# そのリストを参照してYes,Noを答える問題と推測
# でもどうやって組み合わせリストを作る実装をしたらいいか分からないので解けません
# たぶん解けてる人そんないないから結構ハードな問題と予想

