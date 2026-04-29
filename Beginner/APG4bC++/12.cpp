#include <bits/stdc++.h>
using namespace std;

int main() {
    string S;
    cin >> S;

    // ここにプログラムを追記
    int ans = 1;
    for (size_t i = 0; i < S.size(); i++)
    {
        if (S.at(i) == '+') {
            ans += 1;
        }
        else if (S.at(i) == '-') {
            ans -= 1;
        }
    }
    cout << ans << endl;
}
