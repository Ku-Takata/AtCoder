#include <bits/stdc++.h>
using namespace std;

int main() {
    cin.tie(0);
    ios::sync_with_stdio(0);

    int A,B,C;
    cin >> A >> B >> C;

    int ans = max({A,B,C}) - min({A,B,C});
    cout << ans << endl;
}