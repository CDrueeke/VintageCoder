10 border 6:background 0:foreground 1
20 print "hello mega65"
30 for i=1 to 10:poke 53280,i:next i
40 if a=1 then goto 100 else print "no"
50 dopen#1,"daten",w:print#1,"x":dclose#1
60 sprite 1,1,2:movspr 1,100,100
70 x=pot(1)+bump(1)+joy(1)
80 do:a=a+1:loop until a>9
90 dma 0,1,2,3:vsync 100
100 box 10,10,50,50:circle 20,20,5,5:line 0,0,10,10
110 print "x";hex$(255);err$(1):data 1,2,3
120 sys 8210:bank 128:window 0,0,79,24
130 rem comment
140 a$="x":b=len(a$)+val("5"):c=fre(0)
150 mount "x.d81":cursor 1,2:screen set 640,400,2:palette 0,1,2,3,4
