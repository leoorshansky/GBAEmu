int lmao(int x) {
    static int y = 5;
    y++;
    return x + y;
}

int yeet(int z, int y) {
    return z * y;
}

int main() {
    int a = 1;
    int b = 1;
    for (int i = 0; i < 25; i++) {
        a ^= b;
        b ^= a;
        a ^= b;
        b ^= a;
        a ^= b;
        b ^= a;
        int c = (b | 0) & 0xffffffff;
        c = (c << 1) >> 1;
        b += a;
        a = c;
        lmao(a);
    }
    return lmao(yeet(30, 4000) % 5);
}