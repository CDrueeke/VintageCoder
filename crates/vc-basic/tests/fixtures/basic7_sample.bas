10 graphic 1,1:color 0,2:scnclr
20 do while a<10:a=a+1:loop
30 if a>5 then print "fertig":else print "nein"
40 sprite 1,1,3:movspr 1,100,100
50 sound 1,5000,60:vol 8:play "cde"
60 trap 100:window 0,0,20,10,1
70 print using "###.##";3.14159:dopen#1,"x",w:dclose
80 collision 1,200:scale 1:box 1,10,10,50,50
