/* Verlustfreie PNG-Optimierung ohne Fremdwerkzeug: Zeilenfilter neu wählen, Alphakanal entfernen, wenn er überall 255 ist,
   mit zlib Stufe 9 neu packen, danach Pixel gegen das Original prüfen. Aufruf: pngopt datei.png [...]  (überschreibt nur bei Verkleinerung)
   Übersetzen: cc -O2 -o /tmp/pngopt werkzeug/pngopt.c -lz */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <zlib.h>
typedef unsigned char u8;
static u8 *slurp(const char *p, size_t *n) { FILE *f = fopen(p, "rb"); if (!f) return 0; fseek(f, 0, SEEK_END); *n = ftell(f); rewind(f); u8 *b = malloc(*n); if (fread(b, 1, *n, f) != *n) { fclose(f); free(b); return 0; } fclose(f); return b; }
static unsigned be32(const u8 *p) { return (unsigned)p[0] << 24 | p[1] << 16 | p[2] << 8 | p[3]; }
static void put32(u8 *p, unsigned v) { p[0] = v >> 24; p[1] = v >> 16; p[2] = v >> 8; p[3] = v; }
static int paeth(int a, int b, int c) { int p = a + b - c, pa = abs(p - a), pb = abs(p - b), pc = abs(p - c); return pa <= pb && pa <= pc ? a : pb <= pc ? b : c; }
/* liest PNG (8 Bit RGB/RGBA, nicht verschachtelt) -> Pixel (ohne Filter) */
static u8 *decode(const u8 *f, size_t n, unsigned *w, unsigned *h, int *ct) {
  if (n < 8 || memcmp(f, "\x89PNG\r\n\x1a\n", 8)) return 0;
  size_t pos = 8, cap = 0, len = 0; u8 *idat = 0; int depth = 0, il = 0; *ct = -1;
  while (pos + 12 <= n) {
    unsigned l = be32(f + pos); const u8 *t = f + pos + 4, *d = f + pos + 8;
    if (!memcmp(t, "IHDR", 4)) { *w = be32(d); *h = be32(d + 4); depth = d[8]; *ct = d[9]; il = d[12]; }
    else if (!memcmp(t, "IDAT", 4)) { if (len + l > cap) { cap = (len + l) * 2; idat = realloc(idat, cap); } memcpy(idat + len, d, l); len += l; }
    else if (!memcmp(t, "IEND", 4)) break;
    pos += 12 + l;
  }
  if (depth != 8 || il || (*ct != 2 && *ct != 6)) { free(idat); return 0; }
  int bpp = *ct == 6 ? 4 : 3; size_t stride = (size_t)*w * bpp, rawn = (stride + 1) * *h;
  u8 *raw = malloc(rawn); uLongf rl = rawn;
  if (uncompress(raw, &rl, idat, len) != Z_OK || rl != rawn) { free(raw); free(idat); return 0; }
  free(idat);
  u8 *px = malloc(stride * *h);
  for (unsigned y = 0; y < *h; y++) {
    u8 ft = raw[y * (stride + 1)], *s = raw + y * (stride + 1) + 1, *o = px + y * stride, *up = y ? o - stride : 0;
    for (size_t x = 0; x < stride; x++) {
      int a = x >= (size_t)bpp ? o[x - bpp] : 0, b = up ? up[x] : 0, c = (up && x >= (size_t)bpp) ? up[x - bpp] : 0, v = s[x];
      switch (ft) { case 1: v += a; break; case 2: v += b; break; case 3: v += (a + b) / 2; break; case 4: v += paeth(a, b, c); break; }
      o[x] = (u8)v;
    }
  }
  free(raw); return px;
}
static void chunk(FILE *f, const char *t, const u8 *d, unsigned l) { u8 h[8]; put32(h, l); memcpy(h + 4, t, 4); fwrite(h, 1, 8, f); if (l) fwrite(d, 1, l, f);
  unsigned c = crc32(0, (const u8 *)t, 4); if (l) c = crc32(c, d, l); u8 e[4]; put32(e, c); fwrite(e, 1, 4, f); }
int main(int argc, char **argv) {
  long before = 0, after = 0;
  for (int i = 1; i < argc; i++) {
    size_t n; u8 *f = slurp(argv[i], &n); unsigned w, h; int ct; u8 *px = f ? decode(f, n, &w, &h, &ct) : 0;
    if (!px) { fprintf(stderr, "%s: übersprungen\n", argv[i]); free(f); continue; }
    int bpp = ct == 6 ? 4 : 3; size_t stride = (size_t)w * bpp;
    int opaque = bpp == 4; if (opaque) for (size_t k = 3; k < stride * h; k += 4) if (px[k] != 255) { opaque = 0; break; }
    int nb = opaque ? 3 : bpp; size_t ns = (size_t)w * nb; u8 *q = px;
    if (opaque) { q = malloc(ns * h); for (size_t k = 0, m = 0; k < stride * h; k += 4) { q[m++] = px[k]; q[m++] = px[k + 1]; q[m++] = px[k + 2]; } }
    u8 *out = malloc((ns + 1) * h), *cand = malloc(ns * 5);
    for (unsigned y = 0; y < h; y++) {
      const u8 *r = q + y * ns, *up = y ? r - ns : 0; long best = -1; int bf = 0;
      for (int ft = 0; ft < 5; ft++) { long sum = 0; u8 *c = cand + ft * ns;
        for (size_t x = 0; x < ns; x++) { int a = x >= (size_t)nb ? r[x - nb] : 0, b = up ? up[x] : 0, cc = (up && x >= (size_t)nb) ? up[x - nb] : 0, p = 0;
          switch (ft) { case 0: p = 0; break; case 1: p = a; break; case 2: p = b; break; case 3: p = (a + b) / 2; break; case 4: p = paeth(a, b, cc); break; }
          c[x] = (u8)(r[x] - p); sum += c[x] < 128 ? c[x] : 256 - c[x]; }
        if (best < 0 || sum < best) { best = sum; bf = ft; } }
      out[y * (ns + 1)] = bf; memcpy(out + y * (ns + 1) + 1, cand + bf * ns, ns);
    }
    uLongf cl = compressBound((ns + 1) * h); u8 *z = malloc(cl); compress2(z, &cl, out, (ns + 1) * h, 9);
    char tmp[4096]; snprintf(tmp, sizeof tmp, "%s.opt", argv[i]); FILE *o = fopen(tmp, "wb");
    fwrite("\x89PNG\r\n\x1a\n", 1, 8, o); u8 ih[13]; put32(ih, w); put32(ih + 4, h); ih[8] = 8; ih[9] = nb == 4 ? 6 : 2; ih[10] = ih[11] = ih[12] = 0;
    chunk(o, "IHDR", ih, 13); chunk(o, "IDAT", z, (unsigned)cl); chunk(o, "IEND", 0, 0); fclose(o);
    /* Prüfung: Ausgabe lesen und mit den Pixeln des Originals vergleichen */
    size_t n2; u8 *f2 = slurp(tmp, &n2); unsigned w2, h2; int ct2; u8 *p2 = decode(f2, n2, &w2, &h2, &ct2); int ok = p2 && w2 == w && h2 == h && !memcmp(p2, q, ns * h);
    if (ok && (long)n2 < (long)n) { rename(tmp, argv[i]); before += n; after += n2; } else { remove(tmp); before += n; after += n; if (!ok) fprintf(stderr, "%s: Prüfung fehlgeschlagen, Original bleibt\n", argv[i]); }
    free(f); free(px); if (opaque) free(q); free(out); free(cand); free(z); free(f2); free(p2);
  }
  printf("vorher %ld KB, nachher %ld KB\n", before / 1024, after / 1024); return 0;
}
