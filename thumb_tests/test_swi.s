B start
NOP
B handler
.align 4
start:
MOV R0, #1
ADR LR, main + 1
BX LR
.THUMB
.align 4
main:
SUB R0, #1
BNE l
SWI 69
l:
BX LR
.align 4
.ARM
handler:
ADR R7, t + 1
MOV LR, R7
BX LR
.THUMB
t:
BL main
MOV R0, #69
