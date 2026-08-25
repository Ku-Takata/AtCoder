#include <bits/stdc++.h>
using namespace std;

int main() {
    int A, B;
    cin >> A >> B;

    // ここにプログラムを追記
    string ans_A = "", ans_B = "";

    int i = 0;
    while (i < A) {
        ans_A += "]";
        i++;
    }

    int j = 0;
    while (j < B) {
        ans_B += "]";
        j++;
    }

    cout << "A:" << ans_A << endl;
    cout << "B:" << ans_B << endl;
}
