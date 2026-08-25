#include <bits/stdc++.h>
using namespace std;

int main() {
    vector<int> data(5);
    for (int i = 0; i < 5; i++) {
        cin >> data.at(i);
    }

    // dataの中で隣り合う等しい要素が存在するなら"YES"を出力し、そうでなければ"NO"を出力する
    // ここにプログラムを追記
    string ans = "NO";
    for (int j = 0; j < 4; j++) {
        if (data.at(j) == data.at(j+1)) {
            ans = "YES";
            break;
        }
    }

    cout << ans << endl;
}
