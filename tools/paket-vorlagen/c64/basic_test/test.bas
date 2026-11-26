; Handpruefung BASIC (Rundgang 47 bis 49)
#define STERNE 5
#if SYSTEM_C128
print "c128"
#else
print "nicht c128"
#endif
@start:
for i = 1 to STERNE
print "*"
#asm "code.asm"
print "A"  ' Kommentar
end
