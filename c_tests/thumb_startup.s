.section startup
.arm
B resetHandler
NOP
NOP
NOP
NOP
NOP
resetHandler:
MOV SP, #0x20000
ADR LR, thumber + 1
BX LR
.thumb
thumber:
BL main
MOV R8, R0
MOV R8, R1
MOV R8, R2
MOV R8, R3
BL end
